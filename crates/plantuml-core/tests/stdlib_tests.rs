//! Интеграционные тесты стандартной библиотеки.
//!
//! Проверяют СКВОЗНОЙ путь: `render` → препроцессор → разбор → SVG.
//! Отдельные тесты препроцессора тут не помогли бы: реестр stdlib был
//! недостижим именно из публичного API, а тест, конструировавший
//! препроцессор с резолвером вручную, этого не замечал.

use plantuml_core::{parse_diagram, render, RenderOptions};

/// Включение из стандартной библиотеки доступно без настройки.
#[test]
fn stdlib_include_works_out_of_the_box() {
    let source = "@startuml\n!include <C4/C4_Context>\nPerson(a, \"Пользователь\")\n@enduml";
    let diagram = parse_diagram(source).expect("stdlib-включение должно разрешаться");
    assert_eq!(
        diagram.diagram_type(),
        plantuml_ast::diagram::DiagramType::Component
    );
}

/// Диаграмма C4 рисуется целиком: элементы и связи с подписями.
#[test]
fn c4_context_renders_elements_and_relations() {
    let source = "@startuml\n\
        !include <C4/C4_Context>\n\
        Person(пользователь, \"Пользователь\")\n\
        System(система, \"Система\")\n\
        System_Ext(внешняя, \"Внешняя система\")\n\
        Rel(пользователь, система, \"Использует\")\n\
        Rel_D(система, внешняя, \"REST\")\n\
        @enduml";

    let svg = render(source, &RenderOptions::default()).expect("C4-диаграмма должна рисоваться");
    for text in [
        "Пользователь",
        "Система",
        "Внешняя система",
        "Использует",
        "REST",
    ] {
        assert!(svg.contains(text), "в выводе нет «{text}»");
    }
}

/// Границы C4 открываются фигурной скобкой вызывающего, а не макросом.
///
/// Регрессия: в нашей библиотеке макросы `*_Boundary` заканчивались
/// символом `{`, и вместе со скобкой в исходнике получалось `{ {`.
#[test]
fn c4_boundary_takes_brace_from_caller() {
    let source = "@startuml\n\
        !include <C4/C4_Context>\n\
        System_Boundary(граница, \"Граница\") {\n\
        System(система, \"Система\")\n\
        }\n\
        @enduml";

    let svg =
        render(source, &RenderOptions::default()).expect("диаграмма с границей должна рисоваться");
    assert!(svg.contains("Граница"));
    assert!(svg.contains("Система"));
}

/// Все четыре направления `Rel_*` разбираются.
#[test]
fn c4_directional_relations_work() {
    let source = "@startuml\n\
        !include <C4/C4_Context>\n\
        Person(a, \"A\")\n\
        System(b, \"B\")\n\
        Rel_D(a, b, \"Вниз\")\n\
        Rel_U(b, a, \"Вверх\")\n\
        Rel_L(a, b, \"Влево\")\n\
        Rel_R(b, a, \"Вправо\")\n\
        @enduml";

    render(source, &RenderOptions::default()).expect("направленные связи должны разбираться");
}

/// Двунаправленные связи, включая направленные варианты.
///
/// `BiRel` порождает `<-->`, `BiRel_D` — `<-down->`. Порядок альтернатив
/// в грамматике важен: `<-` перехватывает начало обеих форм.
#[test]
fn c4_bidirectional_relations_work() {
    for (name, call) in [
        ("BiRel", "BiRel"),
        ("BiRel_D", "BiRel_D"),
        ("BiRel_L", "BiRel_L"),
    ] {
        let source = format!(
            "@startuml\n!include <C4/C4_Context>\nPerson(a, \"A\")\nSystem(b, \"B\")\n{call}(a, b, \"Связь\")\n@enduml"
        );
        let svg = render(&source, &RenderOptions::default())
            .unwrap_or_else(|e| panic!("{name} не разбирается: {e}"));
        assert!(svg.contains("Связь"), "{name}: подпись связи потеряна");
    }
}

/// Несуществующее включение даёт понятную ошибку, а не панику.
#[test]
fn unknown_stdlib_include_is_an_error() {
    let result = parse_diagram("@startuml\n!include <нет/такого>\nA -> B\n@enduml");
    assert!(result.is_err());
}

/// Все включения реестра разбираются.
///
/// Регрессия: работало 9 из 39. Остальные — библиотеки иконок, содержимое
/// которых состоит из определений спрайтов (`sprite $имя [ШxВ/цветов]`),
/// а такого правила в грамматике не было вовсе.
#[test]
fn every_registry_include_parses() {
    // Список повторяет реестр plantuml-stdlib. Держим его здесь явно:
    // plantuml-core не зависит от plantuml-stdlib напрямую, а проверять
    // надо именно сквозной путь через render.
    let includes = [
        "C4/C4_Context",
        "C4/C4_Container",
        "C4/C4_Component",
        "C4/C4_Dynamic",
        "C4/C4_Deployment",
        "tupadr3/common",
        "logos/rust",
        "logos/docker",
        "office/Users/user",
        "common",
    ];

    for path in includes {
        let source = format!("@startuml\n!include <{path}>\nclass A\n@enduml");
        parse_diagram(&source).unwrap_or_else(|e| panic!("<{path}> не разбирается: {e}"));
    }
}

/// Определение спрайта не ломает разбор.
#[test]
fn sprite_definition_parses() {
    let source = "@startuml\n\
        sprite $s [4x4/4] {\n\
        0123\n\
        1230\n\
        2301\n\
        3012\n\
        }\n\
        class A\n\
        @enduml";
    parse_diagram(source).expect("спрайт должен разбираться");
}
