//! Библиотека не навязывает приложению глобальный аллокатор.
//!
//! Проверка на этапе сборки: если `plantuml-core` без `memory-limit` объявит
//! `#[global_allocator]`, этот файл перестанет компилироваться, потому что
//! двух глобальных аллокаторов в одной программе быть не может.

#![cfg(not(feature = "memory-limit"))]

use std::alloc::System;

#[global_allocator]
static APP_ALLOCATOR: System = System;

#[test]
fn test_library_works_with_application_allocator() {
    let svg = plantuml_core::render(
        "@startuml\nAlice -> Bob\n@enduml",
        &plantuml_core::RenderOptions::default(),
    )
    .expect("простая схема рисуется");
    assert!(svg.contains("<svg"));
}

#[test]
fn test_unknown_chart_command_is_an_error() {
    let result = plantuml_core::render(
        "@startchart\nh-axis [a, b]\nbar [1, 2]\nannotation \"важно\"\n@endchart",
        &plantuml_core::RenderOptions::default(),
    );
    assert!(result.is_err(), "молчаливая потеря строки графика");
}

#[test]
fn test_transparent_background_leaves_only_authors_fill() {
    let options = plantuml_core::RenderOptions::default().with_transparent_background();
    let plain = plantuml_core::render("@startuml\nA -> B\n@enduml", &options).expect("схема");
    let root = &plain[plain.find("<svg").expect("корень")..];
    let root = &root[..root.find('>').expect("конец тега")];
    assert!(!root.contains("background:"), "{root}");

    let source = "@startuml\nskinparam backgroundColor #FF0000\nA -> B\n@enduml";
    let authored = plantuml_core::render(source, &options).expect("схема");
    assert!(
        authored.to_lowercase().contains("#ff0000"),
        "фон автора потерян"
    );
}

#[test]
fn test_dark_theme_inverts_named_fill() {
    let options = plantuml_core::RenderOptions::default().with_dark(true);
    let source = "@startuml\nbox \"B\" #LightBlue\nparticipant A\nend box\nA -> A\n@enduml";
    let svg = plantuml_core::render(source, &options).expect("схема");
    assert!(
        !svg.contains("LightBlue"),
        "светлая заливка под светлым текстом"
    );
}
