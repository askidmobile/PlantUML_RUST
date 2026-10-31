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
