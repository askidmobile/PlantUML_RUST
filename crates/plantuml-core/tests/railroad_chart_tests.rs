//! Сквозные проверки `@startregex`, `@startebnf`, `@startchart`: из текста в SVG.

use plantuml_core::{render, RenderOptions};

fn svg(source: &str) -> String {
    render(source, &RenderOptions::default()).expect("диаграмма должна рисоваться")
}

/// Есть ли подпись `word`: svg-рендерер пишет текст отдельной строкой.
fn has_text(out: &str, word: &str) -> bool {
    out.contains(&format!(">\n{word}\n</text>"))
}

#[test]
fn regex_draws_boxes_with_text_inside_canvas() {
    let out = svg("@startregex\n(a|b)+c\\d\n@endregex");
    assert!(out.contains("data-diagram-type=\"REGEX\""));
    for word in ["a", "b", "c", "\\d"] {
        assert!(has_text(&out, word), "нет подписи {word}");
    }
    // Повтор рисуется обратной дорожкой со стрелкой
    assert!(out.matches("<path").count() >= 4);
}

#[test]
fn ebnf_draws_rule_name_terminals_and_end_markers() {
    let out = svg("@startebnf\nrule = \"x\", [ other ];\n@endebnf");
    assert!(out.contains("data-diagram-type=\"EBNF\""));
    assert!(["rule", "x", "other"].iter().all(|w| has_text(&out, w)));
    // Начальный (пустой) и конечный (залитый) кружки
    assert_eq!(out.matches("<ellipse").count(), 2);
}

#[test]
fn chart_draws_axes_bars_and_legend_from_data() {
    let out = svg(
        "@startchart\nh-axis [Q1, Q2]\nbar \"Sales\" [30, 60]\nline \"Plan\" [40, 50]\nlegend right\n@endchart",
    );
    assert!(out.contains("data-diagram-type=\"CHART\""));
    for word in ["Q1", "Q2", "100", "Sales", "Plan"] {
        assert!(has_text(&out, word), "нет подписи {word}");
    }
    // Столбец 60 из 100 на шкале высотой 300 — 180
    assert!(out.contains("height=\"180\""));
}
