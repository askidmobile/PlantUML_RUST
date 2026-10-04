//! Тесты на фикстурах из `tests/fixtures/`.
//!
//! Эти `.puml`-файлы лежали в репозитории, но не использовались ни одним
//! тестом: регрессия в них не отслеживалась. Теперь каждый файл
//! прогоняется через полный pipeline.
//!
//! Проверяется не «похожесть» вывода, а два свойства:
//! 1. pipeline не падает и не возвращает пустой результат;
//! 2. все подписи, встречающиеся в исходнике, попадают в SVG.

use std::path::{Path, PathBuf};

use plantuml_core::{render, RenderOptions};

/// Путь к директории фикстур.
fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}

/// Собирает все `.puml`-файлы фикстур.
fn all_fixtures() -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_puml(&fixtures_dir(), &mut out);
    out.sort();
    out
}

fn collect_puml(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_puml(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("puml") {
            out.push(path);
        }
    }
}

/// Имя фикстуры для сообщений об ошибках.
fn fixture_name(path: &Path) -> String {
    path.strip_prefix(fixtures_dir())
        .unwrap_or(path)
        .display()
        .to_string()
}

/// Извлекает подписи связей вида `A -> B: текст`.
///
/// Возвращает тексты, которые обязаны попасть в SVG.
///
/// Важна именно проверка на стрелку: в class-диаграммах двоеточие
/// встречается и внутри членов класса (`+ setName(name: String): void`),
/// но это не подпись связи, а часть содержимого класса.
fn expected_labels(source: &str) -> Vec<String> {
    // Разделители связей всех поддерживаемых типов диаграмм
    const ARROWS: &[&str] = &["-->", "<--", "->>", "<<-", "->", "<-", "--", "..", "-["];

    let mut out = Vec::new();
    for line in source.lines() {
        let line = line.trim();
        if line.starts_with('\'') || line.starts_with('@') {
            continue;
        }

        let Some(colon) = line.find(": ") else {
            continue;
        };

        // Стрелка должна быть ДО двоеточия — иначе это не связь
        let before = &line[..colon];
        let has_arrow = ARROWS.iter().any(|a| before.contains(a));
        if !has_arrow {
            continue;
        }

        let text = line[colon + 2..].trim();
        if !text.is_empty() && !text.contains('"') {
            out.push(text.to_string());
        }
    }
    out
}

#[test]
fn fixtures_exist() {
    let fixtures = all_fixtures();
    assert!(
        !fixtures.is_empty(),
        "не найдено ни одного .puml в {}",
        fixtures_dir().display()
    );
}

/// Каждая фикстура рендерится без ошибок и даёт непустой SVG.
#[test]
fn all_fixtures_render() {
    let fixtures = all_fixtures();
    assert!(!fixtures.is_empty(), "нет фикстур");

    let mut failures = Vec::new();

    for path in &fixtures {
        let name = fixture_name(path);
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!("{name}: не читается файл ({e})"));
                continue;
            }
        };

        match render(&source, &RenderOptions::default()) {
            Ok(svg) => {
                if !svg.contains("<svg") {
                    failures.push(format!("{name}: вывод не содержит <svg"));
                }
                if svg.len() < 100 {
                    failures.push(format!(
                        "{name}: подозрительно короткий SVG ({} байт)",
                        svg.len()
                    ));
                }
            }
            Err(e) => failures.push(format!("{name}: ошибка рендера: {e}")),
        }
    }

    assert!(
        failures.is_empty(),
        "фикстуры не отрендерились ({} из {}):\n  - {}",
        failures.len(),
        fixtures.len(),
        failures.join("\n  - ")
    );
}

/// Экранирует текст так, как это делает SVG-сериализатор.
///
/// В XML символы `&`, `<`, `>` обязаны быть экранированы, поэтому подпись
/// `places >` попадает в разметку как `places &gt;`. Проверять наличие
/// подписи нужно с учётом этого.
fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Подписи из исходников фикстур попадают в SVG.
///
/// Это проверка на молчаливую потерю данных: если парсер отбросил часть
/// сообщений или связей, текст исчезнет из вывода, и тест это заметит.
#[test]
fn fixture_labels_are_preserved() {
    let mut failures = Vec::new();
    let mut checked = 0;

    for path in &all_fixtures() {
        let name = fixture_name(path);
        let Ok(source) = std::fs::read_to_string(path) else {
            continue;
        };
        let Ok(svg) = render(&source, &RenderOptions::default()) else {
            continue; // отдельный тест сообщит об ошибке рендера
        };

        for label in expected_labels(&source) {
            checked += 1;
            // Подпись может попасть в разметку как есть или в экранированном виде
            if !svg.contains(&label) && !svg.contains(&xml_escape(&label)) {
                failures.push(format!("{name}: подпись «{label}» потеряна"));
            }
        }
    }

    assert!(checked > 0, "не нашлось ни одной подписи для проверки");
    assert!(
        failures.is_empty(),
        "потеряны подписи ({} из {checked}):\n  - {}",
        failures.len(),
        failures.join("\n  - ")
    );
}
