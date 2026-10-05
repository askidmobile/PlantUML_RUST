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

/// Спрайт превращается в набор прямоугольников.
///
/// Регрессия: грамматика `sprite_stmt` принимала определение, но парсер
/// его не обрабатывал — спрайт терялся ещё до AST. Затем спрайты не
/// пробрасывались в раскладку и не рисовались. Теперь рисуются: палитра
/// PlantUML известна (16 оттенков серого), поэтому PNG-кодер не нужен.
#[test]
fn sprite_renders_as_rectangles() {
    let source = "@startuml\n\
        sprite $s [4x4/16] {\n\
        0123\n\
        1230\n\
        2301\n\
        3012\n\
        }\n\
        class A\n\
        class \"X <$s>\" as x\n\
        @enduml";

    let svg = render(source, &RenderOptions::default()).expect("диаграмма должна рисоваться");

    // Цифра `0` в палитре полностью прозрачна, поэтому её цвета в выводе
    // нет, а остальные обязаны присутствовать.
    for color in ["#E2E2E2", "#D3D3D3", "#C5C5C5"] {
        assert!(svg.contains(color), "в выводе нет цвета спрайта {color}");
    }
}

/// Спрайт рисуется без потери пикселей.
///
/// Одинаковые пиксели объединяются в блоки: сначала по строке, затем по
/// вертикали. Без объединения спрайт 64x63 давал 10.5 МБ вывода на одну
/// диаграмму; после объединения — меньше мегабайта. Проверяем, что
/// суммарная площадь закраски равна числу непрозрачных пикселей.
#[test]
fn sprite_pixels_are_not_lost_when_merged() {
    // Спрайт 4x4, где цифра `0` прозрачна: непрозрачных 15.
    let source = "@startuml\n\
        sprite $s [4x4/16] {\n\
        1234\n\
        5678\n\
        9abc\n\
        def0\n\
        }\n\
        class A\n\
        class \"X <$s>\" as x\n\
        @enduml";

    let svg = render(source, &RenderOptions::default()).expect("диаграмма должна рисоваться");

    let mut area = 0.0_f64;
    for part in svg.split("<rect").skip(1) {
        let text = part.split("/>").next().unwrap_or("");
        // Прямоугольники спрайта отличаются наличием fill-opacity.
        if !text.contains("fill-opacity") {
            continue;
        }
        let number = |attribute: &str| -> f64 {
            text.find(&format!("{attribute}=\""))
                .and_then(|index| text[index + attribute.len() + 2..].split('"').next())
                .and_then(|value| value.parse().ok())
                .unwrap_or(0.0)
        };
        area += number("width") * number("height");
    }

    assert_eq!(area, 15.0, "площадь закраски не совпала с числом пикселей");
}

/// Векторный спрайт (`sprite имя <svg ...>...</svg>`) разбирается и рисуется.
///
/// Формат из официальной документации PlantUML: имя БЕЗ ведущего `$`,
/// тело — встроенный SVG. Библиотека Archimate использует именно его.
#[test]
fn vector_sprite_is_parsed_and_rendered() {
    let source = "@startuml\n\
        sprite foo1 <svg width=\"8\" height=\"8\" viewBox=\"0 0 8 8\">\n\
        <path d=\"M1 0l-1 1 1.5 1.5-1.5 1.5h4v-4l-1.5 1.5-1.5-1.5z\" />\n\
        </svg>\n\
        class A\n\
        class \"X <$foo1>\" as x\n\
        @enduml";

    let svg = render(source, &RenderOptions::default()).expect("диаграмма должна рисоваться");

    assert!(
        svg.contains("M1 0l-1 1"),
        "тело векторного спрайта не перенесено в вывод"
    );
    assert!(
        svg.contains("scale("),
        "не найдено преобразование системы координат спрайта"
    );
}

/// Размер векторного спрайта берётся из `viewBox`, а не из `width`.
///
/// Библиотека Archimate задаёт `width`/`height` в МИЛЛИМЕТРАХ
/// (`width="19.995mm"`), а систему координат тела — в `viewBox`.
/// Если брать миллиметры, спрайт получит неверный масштаб.
#[test]
fn vector_sprite_size_comes_from_view_box() {
    let source = "@startuml\n\
        sprite react <svg width=\"19.995mm\" height=\"19.928mm\" viewBox=\"0 0 230 230\">\n\
        <circle cx=\"115\" cy=\"115\" r=\"20.5\" fill=\"#61dafb\"/>\n\
        </svg>\n\
        class A\n\
        @enduml";

    let diagram = plantuml_parser::parse(source).expect("диаграмма должна разбираться");
    let plantuml_ast::Diagram::Class(class) = diagram else {
        panic!("ожидалась class-диаграмма");
    };

    let sprite = class.sprites.first().expect("спрайт не найден");
    assert_eq!(sprite.name, "react");
    assert_eq!(
        (sprite.width, sprite.height),
        (230, 230),
        "размер должен быть взят из viewBox, а не из миллиметров"
    );
    assert!(sprite.svg.is_some(), "векторное тело потеряно");
}
