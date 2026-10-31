//! Гейт полноты: схема из корпуса либо отрисована со всем содержимым, либо не отрисована.
//!
//! Корпус `tests/golden/corpus/` — примеры из справочников (`ref_*`), отдельные
//! возможности синтаксиса (`feat_*`) и реалистичные схемы (`real_*`). Эталоны
//! `corpus_reference/` сняты с plantuml.com и закоммичены, сеть тест не использует.
//!
//! Сравниваем слова подписей, а не геометрию: геометрию проверяет `golden_tests`.
//! Нарушения считаются так:
//!
//! - `ERR` — эталон отрисован, у нас ошибка;
//! - `LOST` — слова эталона нет в нашем SVG, то есть молчаливая потеря;
//! - `RAW` — в нашем SVG слово, которого нет в эталоне, обычно сырая разметка;
//! - `ACCEPTED` — эталон сообщает об ошибке синтаксиса, а мы рисуем.
//!
//! Исключения с причиной — `tests/golden/exceptions.txt`. Список нарушений
//! зафиксирован храповиком в `gate_baseline.txt`: новое нарушение валит тест,
//! исправленное требует убрать строку (`UPDATE_GATE=1`).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use plantuml_core::{render, RenderOptions};

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden")
}

/// Текст элементов `<text>` с раскрытыми сущностями.
fn svg_text(svg: &str) -> String {
    text_of(svg, false)
}

/// Текст эталона без предупреждений сервера об устаревшем синтаксисе.
///
/// Сервер печатает их кеглем 10 над диаграммой. Содержимым схемы они не являются.
fn reference_text(svg: &str) -> String {
    // В эталоне между словами стоят неразрывные пробелы.
    let flat = svg.replace("&#160;", " ").replace('\u{a0}', " ");
    let has_warning = flat.contains("is deprecated") || flat.contains("is ignored");
    text_of(svg, has_warning)
}

fn text_of(svg: &str, skip_small: bool) -> String {
    let mut out = String::new();
    let mut rest = svg;
    while let Some(start) = rest.find("<text") {
        let tail = &rest[start..];
        let Some(gt) = tail.find('>') else { break };
        let content = &tail[gt + 1..];
        let Some(end) = content.find("</text>") else {
            break;
        };
        // Буква значка класса: контур в эталоне, не слово.
        if tail[..gt].contains("data-glyph") {
            rest = &content[end..];
            continue;
        }
        if skip_small && tail[..gt].contains("font-size=\"10\"") {
            rest = &content[end..];
            continue;
        }
        out.push_str(&content[..end]);
        out.push(' ');
        rest = &content[end..];
    }
    strip_tags(&decode_entities(&out))
}

fn strip_tags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut depth = false;
    for c in text.chars() {
        match c {
            '\u{1}' => depth = true,
            '\u{2}' => {
                depth = false;
                out.push(' ');
            }
            _ if !depth => out.push(c),
            _ => {}
        }
    }
    out
}

/// Раскрывает сущности XML. Разметка уже закрыта своими символами, чтобы `&lt;b&gt;`
/// из текста не спутать с настоящим вложенным элементом.
fn decode_entities(raw: &str) -> String {
    let marked: String = raw
        .chars()
        .map(|c| match c {
            '<' => '\u{1}',
            '>' => '\u{2}',
            other => other,
        })
        .collect();
    let mut out = String::with_capacity(marked.len());
    let mut rest = marked.as_str();
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let tail = &rest[amp..];
        let Some(semi) = tail.find(';').filter(|i| *i <= 10) else {
            out.push('&');
            rest = &tail[1..];
            continue;
        };
        let entity = &tail[1..semi];
        match entity {
            "lt" => out.push('<'),
            "gt" => out.push('>'),
            "amp" => out.push('&'),
            "quot" => out.push('"'),
            "apos" => out.push('\''),
            _ => {
                let code = entity
                    .strip_prefix("#x")
                    .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                    .or_else(|| entity.strip_prefix('#').and_then(|d| d.parse().ok()));
                if let Some(c) = code.and_then(char::from_u32) {
                    out.push(c);
                }
            }
        }
        rest = &tail[semi + 1..];
    }
    out.push_str(rest);
    out
}

/// Слова текста: буквы и цифры подряд.
fn words(text: &str) -> BTreeSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_string)
        .collect()
}

/// Эталон сообщает об ошибке или сервер отдал только растр (тип вне объёма).
fn reference_is_error(svg: &str) -> bool {
    svg.contains("Syntax Error")
        || svg.contains("[From string")
        || svg.contains("data-reference=\"bitmap\"")
}

/// Нарушение одного кейса в виде строки храповика; `None` — кейс чистый.
fn check(name: &str, source: &str, reference: &str) -> Option<String> {
    // Сервер не разобрал пример и показал страницу приветствия: сверять не с чем.
    if reference.contains("Welcome to PlantUML") {
        return None;
    }
    let ours = render(source, &RenderOptions::default());
    if reference_is_error(reference) {
        return ours.is_ok().then(|| format!("{name} ACCEPTED"));
    }
    let want = words(&reference_text(reference));
    // Схема без содержимого: сервер рисует пустоту, у нас это ошибка «нет элементов».
    let has_nothing = want.is_empty();
    let svg = match ours {
        Ok(svg) => svg,
        Err(_) if has_nothing => return None,
        Err(_) => return Some(format!("{name} ERR")),
    };
    let have = words(&svg_text(&svg));
    let lost: Vec<&str> = want.difference(&have).map(String::as_str).collect();
    // Предупреждения сервера — не потеря, но и не сырая разметка, если мы их тоже вывели.
    let full = words(&svg_text(reference));
    let raw: Vec<&str> = have.difference(&full).map(String::as_str).collect();
    if !lost.is_empty() {
        return Some(format!("{name} LOST {}", lost.join(" ")));
    }
    if !raw.is_empty() {
        return Some(format!("{name} RAW {}", raw.join(" ")));
    }
    None
}

/// `имя: причина` из файла исключений.
fn exceptions(dir: &Path) -> BTreeMap<String, String> {
    let raw = std::fs::read_to_string(dir.join("exceptions.txt")).unwrap_or_default();
    raw.lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .filter_map(|l| l.split_once(':'))
        .map(|(n, r)| (n.trim().to_string(), r.trim().to_string()))
        .collect()
}

fn corpus(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir.join("corpus"))
        .expect("есть корпус")
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let path = e.path();
            (path.extension()? == "puml").then(|| path.file_stem()?.to_str().map(String::from))?
        })
        .collect();
    names.sort();
    names
}

fn number(tag: &str, name: &str) -> Option<f64> {
    let needle = format!(" {name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let rest = &tag[start..];
    rest[..rest.find('"')?].trim_end_matches("px").parse().ok()
}

/// Подпись вышла за холст: координаты текста вне `viewBox`.
fn clipped_text(svg: &str) -> Option<String> {
    let root_start = svg.find("<svg")?;
    let root = &svg[root_start..root_start + svg[root_start..].find('>')?];
    let view = {
        let start = root.find("viewBox=\"")? + "viewBox=\"".len();
        let rest = &root[start..];
        let raw = &rest[..rest.find('"')?];
        raw.split_whitespace()
            .filter_map(|v| v.parse::<f64>().ok())
            .collect::<Vec<f64>>()
    };
    let [vx, vy, vw, vh] = view[..] else {
        return None;
    };
    // Украшения лежат снаружи, сама диаграмма внутри `translate`.
    let (mut dx, mut dy) = (0.0, 0.0);
    if let Some(at) = svg.find("<g transform=\"translate(") {
        let rest = &svg[at + "<g transform=\"translate(".len()..];
        let raw = &rest[..rest.find(')')?];
        let values: Vec<f64> = raw
            .split_whitespace()
            .filter_map(|v| v.parse().ok())
            .collect();
        if let [x, y] = values[..] {
            dx = x;
            dy = y;
        }
    }
    let mut rest = svg;
    while let Some(start) = rest.find("<text") {
        let tail = &rest[start..];
        let gt = tail.find('>')?;
        let tag = &tail[..gt];
        let (Some(x), Some(y)) = (number(tag, "x"), number(tag, "y")) else {
            rest = &tail[gt..];
            continue;
        };
        let length = number(tag, "textLength").unwrap_or(0.0);
        let (left, right) = if tag.contains("text-anchor=\"middle\"") {
            (x - length / 2.0, x + length / 2.0)
        } else {
            (x, x + length)
        };
        // Допуск на несовпадение метрик шрифта.
        let slack = 6.0;
        let inside = |ox: f64, oy: f64| {
            left + ox >= vx - slack
                && right + ox <= vx + vw + slack
                && y + oy >= vy - slack
                && y + oy <= vy + vh + slack
        };
        if !inside(dx, dy) && !inside(0.0, 0.0) {
            let close = tail.find("</text>")?;
            return Some(tail[gt + 1..close].to_string());
        }
        rest = &tail[gt..];
    }
    None
}

/// Первое поле строки: имя кейса.
fn case_of(line: &str) -> &str {
    line.split(' ').next().unwrap_or(line)
}

/// Ни одна подпись корпуса не вылезает за пределы холста.
#[test]
fn corpus_text_stays_on_canvas() {
    let dir = golden_dir();
    let mut failures = Vec::new();
    for name in corpus(&dir) {
        let source = std::fs::read_to_string(dir.join("corpus").join(format!("{name}.puml")))
            .expect("читается исходник");
        if let Ok(svg) = render(&source, &RenderOptions::default()) {
            if let Some(text) = clipped_text(&svg) {
                failures.push(format!("{name}: {text}"));
            }
        }
    }
    assert!(failures.is_empty(), "подписи за холстом:\n{failures:#?}");
}

#[test]
fn corpus_gate() {
    let dir = golden_dir();
    let listed = exceptions(&dir);
    let excepted: BTreeMap<String, String> = listed
        .iter()
        .filter(|(n, _)| !n.starts_with('~'))
        .map(|(n, r)| (n.clone(), r.clone()))
        .collect();
    let excepted_raw: BTreeMap<String, String> = listed
        .iter()
        .filter_map(|(n, r)| Some((n.strip_prefix('~')?.to_string(), r.clone())))
        .collect();
    let names = corpus(&dir);
    assert!(names.len() >= 200, "корпус неполон: {}", names.len());

    let mut violations = Vec::new();
    let mut clean = 0;
    for name in &names {
        let source = std::fs::read_to_string(dir.join("corpus").join(format!("{name}.puml")))
            .expect("читается исходник");
        let reference =
            std::fs::read_to_string(dir.join("corpus_reference").join(format!("{name}.svg")))
                .unwrap_or_else(|_| panic!("нет эталона {name}"));
        let found = check(name, &source, &reference);

        if excepted.contains_key(name) {
            assert!(
                render(&source, &RenderOptions::default()).is_err(),
                "{name} в исключениях, но рисуется: молчаливая потеря недопустима"
            );
            continue;
        }
        // Осознанно рисуем больше эталона: `~имя` в файле исключений.
        if let Some(v) = &found {
            if excepted_raw.contains_key(name) && v.split(' ').nth(1) == Some("RAW") {
                clean += 1;
                continue;
            }
        }
        match found {
            Some(v) => violations.push(v),
            None => clean += 1,
        }
    }

    let count = |kind: &str| {
        violations
            .iter()
            .filter(|v| v.split(' ').nth(1) == Some(kind))
            .count()
    };
    println!(
        "корпус: {} кейсов, чисто {clean}, исключений {}; ERR {}, LOST {}, RAW {}, ACCEPTED {}",
        names.len(),
        excepted.len(),
        count("ERR"),
        count("LOST"),
        count("RAW"),
        count("ACCEPTED"),
    );

    // Подробности для разбора: текст ошибки по каждому кейсу.
    if let Ok(path) = std::env::var("GATE_REPORT") {
        let lines: Vec<String> = violations
            .iter()
            .map(|v| {
                let name = case_of(v);
                let source =
                    std::fs::read_to_string(dir.join("corpus").join(format!("{name}.puml")))
                        .unwrap_or_default();
                match render(&source, &RenderOptions::default()) {
                    Err(e) => format!("{v} :: {e}"),
                    Ok(_) => v.clone(),
                }
            })
            .collect();
        std::fs::write(path, lines.join("\n") + "\n").expect("пишется отчёт");
    }

    let baseline_path = dir.join("gate_baseline.txt");
    if std::env::var("UPDATE_GATE").is_ok_and(|v| v == "1") {
        std::fs::write(&baseline_path, violations.join("\n") + "\n").expect("пишется храповик");
        return;
    }
    let baseline = std::fs::read_to_string(&baseline_path).unwrap_or_default();
    let known: BTreeSet<&str> = baseline
        .lines()
        .filter(|l| !l.is_empty())
        .map(case_of)
        .collect();
    let current: BTreeSet<&str> = violations.iter().map(|v| case_of(v)).collect();

    let new: Vec<&String> = violations
        .iter()
        .filter(|v| !known.contains(case_of(v)))
        .collect();
    assert!(new.is_empty(), "новые нарушения гейта:\n{new:#?}");

    let fixed: Vec<&&str> = known.difference(&current).collect();
    assert!(
        fixed.is_empty(),
        "исправлено, уберите из храповика (UPDATE_GATE=1): {fixed:?}"
    );
}

/// Разбор слов не должен терять кириллицу и экранирование.
#[test]
fn words_are_extracted_from_svg() {
    let svg = r#"<svg><text x="1">Привет&#160;мир</text><text><tspan>a&lt;b</tspan></text></svg>"#;
    let got = words(&svg_text(svg));
    for w in ["Привет", "мир", "a", "b"] {
        assert!(got.contains(w), "нет {w} в {got:?}");
    }
    assert!(!got.contains("tspan"), "разметка попала в текст: {got:?}");
}
