//! Дорожки и вилка activity выглядят как у PlantUML: без заливки и плашек.

use plantuml_core::{render, RenderOptions};

const SOURCE: &str = "@startuml\n|Первая|\nstart\n:a;\nfork\n:b;\nfork again\n:c;\nend fork\n|Вторая|\n:d;\nstop\n@enduml";

fn svg() -> String {
    render(SOURCE, &RenderOptions::default()).expect("диаграмма должна рисоваться")
}

#[test]
fn swimlanes_are_separated_by_lines_without_fill() {
    let out = svg();
    assert!(!out.contains("#FEFECE") && !out.contains("#E2E2F0"));
    // Две дорожки дают три вертикальные линии толщиной 1.5
    assert_eq!(out.matches("stroke-width=\"1.5\"").count(), 3);
    assert!(out.contains("font-size=\"18\""));
}

#[test]
fn fork_bar_is_dark_grey_and_rounded() {
    let out = svg();
    assert!(out.contains("fill=\"#555\"") && out.contains("rx=\"2.5\""));
}

#[test]
fn lane_is_wide_enough_for_its_fork() {
    let out = svg();
    // Вилка из двух веток шире 180 — дорожка растёт вместе с ней
    let width: f64 = out
        .split("width=\"")
        .nth(1)
        .and_then(|rest| rest.split("px").next())
        .and_then(|w| w.parse().ok())
        .expect("ширина холста");
    assert!(width > 400.0, "холст {width} слишком узок для двух дорожек");
}
