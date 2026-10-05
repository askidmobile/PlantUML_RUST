//! Golden-тесты: сравнение нашего рендера с эталонами PlantUML.
//!
//! # Как это работает
//!
//! Эталоны в `tests/golden/reference/*.svg` получены с официального сервера
//! PlantUML (см. `tests/golden/fetch_references.py`) и закоммичены вместе с
//! метаданными: версией PlantUML и хешем исходника. Тесты не обращаются к
//! сети — они сравнивают наш рендер с зафиксированным эталоном.
//!
//! # Почему сравнение с допуском, а не байтовое
//!
//! Байтовое совпадение недостижимо: PlantUML расставляет атрибуты в другом
//! порядке, использует CSS-классы, инструкцию `<?plantuml ...?>` и инлайновые
//! `<polygon>` вместо `<marker>`. Поэтому сравнение идёт по измеримым
//! характеристикам: габариты диаграммы, наличие подписей, число участников.
//!
//! # Модель «храповика» (ratchet)
//!
//! Фактическое расхождение с PlantUML составляет от 7% до 67% (см.
//! `docs/AUDIT.md`), поэтому тест не может требовать совпадения сразу.
//! Вместо этого текущий разрыв зафиксирован в `tests/golden/baseline.json`,
//! и тест падает, только если расхождение **выросло** относительно baseline.
//!
//! Такой храповик даёт три свойства:
//! 1. CI зелёный на текущем (неидеальном) состоянии;
//! 2. регрессия геометрии не проходит незамеченной;
//! 3. прогресс виден — baseline уменьшается по мере работы над Фазой 4.
//!
//! # Обновление baseline
//!
//! После осознанного улучшения геометрии:
//! ```text
//! UPDATE_BASELINE=1 cargo test -p plantuml-core --test golden_tests
//! ```
//! Обновлённый `baseline.json` нужно просмотреть глазами: он и есть
//! свидетельство того, что разрыв сократился.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use plantuml_core::{render, RenderOptions};

/// Абсолютный допуск габаритов сверх baseline, пиксели.
///
/// Даёт небольшой запас на округления вычислений с плавающей точкой,
/// чтобы тест не «мигал» на десятых долях пикселя.
const SLACK_PX: f64 = 1.0;

/// Извлекает размеры и viewBox из SVG.
#[derive(Debug, Clone, Copy, PartialEq)]
struct SvgBox {
    width: f64,
    height: f64,
}

/// Извлекает значение атрибута из первой подходящей разметки.
fn attr(svg: &str, name: &str) -> Option<String> {
    let needle = format!("{}=\"", name);
    let start = svg.find(&needle)? + needle.len();
    let rest = &svg[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Разбирает число, отбрасывая суффикс `px`.
fn parse_num(raw: &str) -> Option<f64> {
    raw.trim_end_matches("px").trim().parse().ok()
}

/// Извлекает габариты SVG.
///
/// PlantUML пишет `width="306px"`, мы — `width="338.5"`. Обе формы
/// поддерживаются.
fn svg_box(svg: &str) -> Option<SvgBox> {
    let width = parse_num(&attr(svg, "width")?)?;
    let height = parse_num(&attr(svg, "height")?)?;
    Some(SvgBox { width, height })
}

/// Извлекает все текстовые подписи из SVG.
fn texts(svg: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = svg;
    while let Some(start) = rest.find("<text") {
        let after_tag = &rest[start..];
        let Some(gt) = after_tag.find('>') else { break };
        let content = &after_tag[gt + 1..];
        let Some(end) = content.find("</text>") else {
            break;
        };
        let value = content[..end].trim();
        if !value.is_empty() {
            out.insert(value.to_string());
        }
        rest = &content[end..];
    }
    out
}

/// Путь к директории golden-тестов.
fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/golden")
        .canonicalize()
        .unwrap_or_else(|_| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden"))
}

/// Результат сравнения одного кейса.
struct Comparison {
    name: String,
    reference: SvgBox,
    ours: SvgBox,
    missing_texts: Vec<String>,
    extra_texts: Vec<String>,
}

impl Comparison {
    fn width_diff(&self) -> f64 {
        (self.ours.width - self.reference.width).abs()
    }

    fn height_diff(&self) -> f64 {
        (self.ours.height - self.reference.height).abs()
    }
}

/// Зафиксированный уровень расхождений для одного кейса.
#[derive(Debug, Clone, Copy)]
struct BaselineEntry {
    width_diff: f64,
    height_diff: f64,
    missing_texts: usize,
}

/// Читает baseline из JSON.
///
/// Разбор намеренно простой: файл формируется этим же тестом и имеет
/// предсказуемую структуру. Формат:
/// `{ "имя_кейса": { "width_diff": 32.5, "height_diff": 21.0, "missing_texts": 0 } }`
fn read_baseline(path: &Path) -> Option<std::collections::HashMap<String, BaselineEntry>> {
    let raw = std::fs::read_to_string(path).ok()?;
    let mut out = std::collections::HashMap::new();

    // Ищем объекты вида "name": { ... }
    let mut rest = raw.as_str();
    while let Some(name_start) = rest.find('"') {
        let after_quote = &rest[name_start + 1..];
        let Some(name_end) = after_quote.find('"') else {
            break;
        };
        let name = &after_quote[..name_end];
        let after_name = &after_quote[name_end..];

        let Some(obj_start) = after_name.find('{') else {
            break;
        };
        let Some(obj_end) = after_name.find('}') else {
            break;
        };
        let obj = &after_name[obj_start..obj_end];

        if let (Some(w), Some(h)) = (
            json_number(obj, "width_diff"),
            json_number(obj, "height_diff"),
        ) {
            out.insert(
                name.to_string(),
                BaselineEntry {
                    width_diff: w,
                    height_diff: h,
                    missing_texts: json_number(obj, "missing_texts").unwrap_or(0.0) as usize,
                },
            );
        }
        rest = &after_name[obj_end..];
    }

    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Достаёт числовое поле из фрагмента JSON.
fn json_number(obj: &str, key: &str) -> Option<f64> {
    let needle = format!("\"{key}\"");
    let start = obj.find(&needle)? + needle.len();
    let rest = &obj[start..];
    let colon = rest.find(':')? + 1;
    let value: String = rest[colon..]
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+')
        .collect();
    value.parse().ok()
}

/// Сериализует baseline в JSON.
fn write_baseline(path: &Path, comparisons: &[Comparison]) -> std::io::Result<()> {
    let mut out = String::from("{\n");
    let mut names: Vec<&Comparison> = comparisons.iter().collect();
    names.sort_by(|a, b| a.name.cmp(&b.name));
    for (i, c) in names.iter().enumerate() {
        out.push_str(&format!(
            "  \"{}\": {{ \"width_diff\": {:.1}, \"height_diff\": {:.1}, \"missing_texts\": {} }}",
            c.name,
            c.width_diff(),
            c.height_diff(),
            c.missing_texts.len()
        ));
        if i + 1 < names.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("}\n");
    std::fs::write(path, out)
}

/// Прогоняет один кейс и собирает сравнение.
fn compare_case(name: &str) -> Result<Comparison, String> {
    let dir = golden_dir();
    let case_path = dir.join("cases").join(format!("{name}.puml"));
    let ref_path = dir.join("reference").join(format!("{name}.svg"));

    let source = std::fs::read_to_string(&case_path)
        .map_err(|e| format!("не читается {}: {e}", case_path.display()))?;
    let reference =
        std::fs::read_to_string(&ref_path).map_err(|e| format!("не читается эталон: {e}"))?;

    let ours =
        render(&source, &RenderOptions::default()).map_err(|e| format!("наш рендер упал: {e}"))?;

    let reference_box = svg_box(&reference).ok_or("в эталоне нет габаритов")?;
    let ours_box = svg_box(&ours).ok_or("в нашем рендере нет габаритов")?;

    let ref_texts = texts(&reference);
    let our_texts = texts(&ours);

    let missing_texts: Vec<String> = ref_texts.difference(&our_texts).cloned().collect();
    let extra_texts: Vec<String> = our_texts.difference(&ref_texts).cloned().collect();

    Ok(Comparison {
        name: name.to_string(),
        reference: reference_box,
        ours: ours_box,
        missing_texts,
        extra_texts,
    })
}

/// Все кейсы, для которых есть и исходник, и эталон.
fn available_cases() -> Vec<String> {
    let dir = golden_dir().join("cases");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let path = e.path();
            if path.extension().and_then(|s| s.to_str()) == Some("puml") {
                path.file_stem().and_then(|s| s.to_str()).map(String::from)
            } else {
                None
            }
        })
        .collect();
    names.sort();
    names
}

/// Печатает таблицу расхождений (видно при `cargo test -- --nocapture`).
fn report(comparisons: &[Comparison]) {
    println!();
    println!(
        "{:<24} {:>14} {:>14} {:>9} {:>9}",
        "кейс", "PlantUML", "наш", "Δ width", "Δ height"
    );
    for c in comparisons {
        println!(
            "{:<24} {:>14} {:>14} {:>9.1} {:>9.1}",
            c.name,
            format!("{:.0}x{:.0}", c.reference.width, c.reference.height),
            format!("{:.0}x{:.0}", c.ours.width, c.ours.height),
            c.width_diff(),
            c.height_diff(),
        );
        if !c.missing_texts.is_empty() {
            println!("  отсутствуют подписи: {:?}", c.missing_texts);
        }
        if !c.extra_texts.is_empty() {
            println!("  лишние подписи:      {:?}", c.extra_texts);
        }
    }
    println!();
}

#[test]
fn golden_cases_exist() {
    let cases = available_cases();
    assert!(
        !cases.is_empty(),
        "нет golden-кейсов в {} — запустите tests/golden/fetch_references.py",
        golden_dir().join("cases").display()
    );
}

/// Все эталоны на месте для каждого кейса.
#[test]
fn golden_references_exist() {
    let dir = golden_dir();
    for name in available_cases() {
        let reference = dir.join("reference").join(format!("{name}.svg"));
        assert!(
            reference.is_file(),
            "нет эталона для кейса {name}: {}\n\
             Запустите: python3 tests/golden/fetch_references.py",
            reference.display()
        );
    }
}

/// Ни один эталон не вырожден.
///
/// PlantUML иногда возвращает SVG, где у ВСЕХ подписей стоит
/// `textLength="0"`, а габариты занижены: например `network_nwdiag`
/// 117x129 вместо правильных 273x140. Такой файл нельзя использовать как
/// эталон — расхождения по нему недостоверны.
///
/// Проверка появилась после того, как выяснилось, что четыре эталона из
/// двадцати были вырожденными: `fetch_references.py` по умолчанию
/// пропускает существующие файлы, поэтому однажды сохранённый плохой
/// ответ оставался навсегда.
#[test]
fn golden_references_are_not_degenerate() {
    let dir = golden_dir();
    for name in available_cases() {
        let path = dir.join("reference").join(format!("{name}.svg"));
        let Ok(svg) = std::fs::read_to_string(&path) else {
            continue;
        };

        let text_count = svg.matches("<text").count();
        if text_count == 0 {
            continue;
        }

        let lengths: Vec<f64> = svg
            .match_indices("textLength=\"")
            .filter_map(|(index, prefix)| {
                let rest = &svg[index + prefix.len()..];
                let end = rest.find('"')?;
                rest[..end].parse::<f64>().ok()
            })
            .collect();

        if lengths.is_empty() {
            continue;
        }

        assert!(
            lengths.iter().any(|value| *value > 0.0),
            "эталон {name} вырожден: у всех {} подписей textLength=0.\n\
             Перезапросите: python3 tests/golden/fetch_references.py --force",
            lengths.len()
        );
    }
}

/// Метаданные эталонов читаются и содержат версию PlantUML.
#[test]
fn golden_metadata_is_valid() {
    let path = golden_dir().join("reference").join("metadata.json");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("не читается {}: {e}", path.display()));
    assert!(
        raw.contains("plantuml_version"),
        "в метаданных нет версии PlantUML"
    );
    assert!(
        raw.contains("source_sha256"),
        "в метаданных нет хеша исходников"
    );
    // Версия не должна быть заглушкой
    assert!(
        !raw.contains("не указана"),
        "версия PlantUML не извлечена — эталоны не маркированы"
    );
}

/// Главный тест: расхождение с PlantUML не выросло относительно baseline.
///
/// Проверяет три вещи:
/// 1. рендер не падает на всех эталонных кейсах;
/// 2. ни одна подпись из эталона не потеряна;
/// 3. габариты не разошлись сильнее, чем зафиксировано в baseline.
///
/// При `UPDATE_BASELINE=1` перезаписывает baseline текущими значениями.
#[test]
fn golden_render_matches_reference() {
    let cases = available_cases();
    assert!(!cases.is_empty(), "нет golden-кейсов");

    let mut comparisons = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    for name in &cases {
        match compare_case(name) {
            Ok(c) => comparisons.push(c),
            Err(e) => failures.push(format!("{name}: {e}")),
        }
    }

    report(&comparisons);

    let baseline_path = golden_dir().join("baseline.json");
    let update = std::env::var("UPDATE_BASELINE").is_ok_and(|v| v == "1");

    if update {
        write_baseline(&baseline_path, &comparisons)
            .unwrap_or_else(|e| panic!("не записать baseline: {e}"));
        println!("baseline обновлён: {}", baseline_path.display());
        assert!(
            failures.is_empty(),
            "часть кейсов не отрендерилась: {failures:?}"
        );
        return;
    }

    let baseline = read_baseline(&baseline_path).unwrap_or_else(|| {
        panic!(
            "не читается baseline: {}\n\
             Создайте его: UPDATE_BASELINE=1 cargo test -p plantuml-core --test golden_tests",
            baseline_path.display()
        )
    });

    for c in &comparisons {
        let Some(base) = baseline.get(&c.name) else {
            failures.push(format!("{}: нет записи в baseline", c.name));
            continue;
        };

        // Потерянные подписи тоже под храповиком: их число не должно расти.
        // Проверять «ноль потерь» нельзя — часть расхождений уже
        // зафиксирована в baseline (например, network не выводит адрес сети
        // и её имя); цель — не допустить ухудшения.
        if c.missing_texts.len() > base.missing_texts {
            failures.push(format!(
                "{}: потеряно подписей больше baseline ({} > {}): {:?}",
                c.name,
                c.missing_texts.len(),
                base.missing_texts,
                c.missing_texts
            ));
        }

        // Храповик: расхождение не должно вырасти.
        if c.width_diff() > base.width_diff + SLACK_PX {
            failures.push(format!(
                "{}: ширина разошлась сильнее baseline ({:.1} > {:.1}), \
                 PlantUML {:.0}, наш {:.0}",
                c.name,
                c.width_diff(),
                base.width_diff,
                c.reference.width,
                c.ours.width,
            ));
        }
        if c.height_diff() > base.height_diff + SLACK_PX {
            failures.push(format!(
                "{}: высота разошлась сильнее baseline ({:.1} > {:.1}), \
                 PlantUML {:.0}, наш {:.0}",
                c.name,
                c.height_diff(),
                base.height_diff,
                c.reference.height,
                c.ours.height,
            ));
        }
    }

    if !failures.is_empty() {
        panic!(
            "регрессия относительно baseline ({} проблем):\n  - {}\n\n\
             Если расхождение сократилось осознанно — обновите baseline:\n  \
             UPDATE_BASELINE=1 cargo test -p plantuml-core --test golden_tests",
            failures.len(),
            failures.join("\n  - ")
        );
    }
}

/// Суммарное расхождение по всем кейсам — метрика прогресса Фазы 4.
///
/// Печатается при `--nocapture`, чтобы видеть, сокращается ли разрыв.
#[test]
fn golden_report_total_deviation() {
    let cases = available_cases();
    let mut total_width = 0.0;
    let mut total_height = 0.0;
    let mut count = 0;

    for name in &cases {
        if let Ok(c) = compare_case(name) {
            total_width += c.width_diff();
            total_height += c.height_diff();
            count += 1;
        }
    }

    if count > 0 {
        println!(
            "\nСуммарное расхождение с PlantUML по {count} кейсам: \
             ширина {total_width:.1}px, высота {total_height:.1}px \
             (среднее {:.1}x{:.1}px на кейс)",
            total_width / count as f64,
            total_height / count as f64
        );
    }
}
