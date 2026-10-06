//! Отступы в activity-диаграммах не должны ломать разбор.
//!
//! # Что было сломано
//!
//! Тело блока в грамматике — повторение `(ws* ~ statement ~ NEWLINE*)*`.
//! Когда последний оператор разобран, повторе откатывается **перед
//! пробелами** следующей строки: неудачная итерация съедает свой `ws*`
//! целиком. Поэтому закрывающее ключевое слово проверялось на пробелах:
//!
//! ```text
//! if (x?) then (да)
//!   :a;
//! endif            <- ждали `endif`, а стояли пробелы
//! ```
//!
//! До правки грамматики НЕ разбирались `if`, `while`, `repeat`, `fork`,
//! `split` и `switch`, если перед закрывающим словом стоял отступ.
//! Отступ в activity-диаграммах пишут постоянно, то есть это ломало
//! обычный синтаксис PlantUML.
//!
//! # Как проверяется
//!
//! Не «разбирается вообще», а даёт **побайтово тот же SVG**, что и текст
//! без отступов. Иначе правка могла бы незаметно изменить раскладку.

use plantuml_core::{render, RenderOptions};

/// Пара «плоский текст — тот же текст с отступом» для каждой конструкции.
const CASES: &[(&str, &str, &str)] = &[
    (
        "if/else",
        "@startuml\nstart\nif (x?) then (да)\n:a;\nelse\n:b;\nendif\nstop\n@enduml",
        "@startuml\nstart\n  if (x?) then (да)\n    :a;\n  else\n    :b;\n  endif\nstop\n@enduml",
    ),
    (
        "elseif",
        "@startuml\nstart\nif (a?) then (да)\n:a;\nelseif (b?) then (нет)\n:b;\nelse\n:c;\nendif\nstop\n@enduml",
        "@startuml\nstart\n  if (a?) then (да)\n    :a;\n  elseif (b?) then (нет)\n    :b;\n  else\n    :c;\n  endif\nstop\n@enduml",
    ),
    (
        "while",
        "@startuml\nstart\nwhile (y?) is (да)\n:a;\nendwhile (нет)\nstop\n@enduml",
        "@startuml\nstart\n  while (y?) is (да)\n    :a;\n  endwhile (нет)\nstop\n@enduml",
    ),
    (
        "repeat",
        "@startuml\nstart\nrepeat\n:a;\nrepeat while (x?) is (нет)\nstop\n@enduml",
        "@startuml\nstart\n  repeat\n    :a;\n  repeat while (x?) is (нет)\nstop\n@enduml",
    ),
    (
        "repeat backward",
        "@startuml\nstart\nrepeat\n:a;\nbackward :a;\nrepeat while (x?)\nstop\n@enduml",
        "@startuml\nstart\n  repeat\n    :a;\n    backward :a;\n  repeat while (x?)\nstop\n@enduml",
    ),
    (
        "fork",
        "@startuml\nstart\nfork\n:a;\nfork again\n:b;\nend fork\nstop\n@enduml",
        "@startuml\nstart\n  fork\n    :a;\n  fork again\n    :b;\n  end fork\nstop\n@enduml",
    ),
    (
        "end merge",
        "@startuml\nstart\nfork\n:a;\nfork again\n:b;\nend merge\nstop\n@enduml",
        "@startuml\nstart\n  fork\n    :a;\n  fork again\n    :b;\n  end merge\nstop\n@enduml",
    ),
    (
        "split",
        "@startuml\nstart\nsplit\n:a;\nsplit again\n:b;\nend split\nstop\n@enduml",
        "@startuml\nstart\n  split\n    :a;\n  split again\n    :b;\n  end split\nstop\n@enduml",
    ),
    (
        "switch",
        "@startuml\nstart\nswitch (q?)\ncase (a)\n:x;\ncase (b)\n:y;\nendswitch\nstop\n@enduml",
        "@startuml\nstart\n  switch (q?)\n    case (a)\n      :x;\n    case (b)\n      :y;\n  endswitch\nstop\n@enduml",
    ),
];

/// Отступ не меняет результат: SVG обязан совпасть побайтово.
#[test]
fn test_indentation_does_not_change_output() {
    for (name, flat, indented) in CASES {
        let without = render(flat, &RenderOptions::default())
            .unwrap_or_else(|e| panic!("{name}: текст без отступа не разобран: {e}"));
        let with = render(indented, &RenderOptions::default())
            .unwrap_or_else(|e| panic!("{name}: текст с отступом не разобран: {e}"));

        assert_eq!(
            with, without,
            "{name}: отступ изменил результат — раскладка разошлась"
        );
    }
}

/// Каждая конструкция с отступом даёт непустой SVG.
#[test]
fn test_indented_constructs_render_non_empty() {
    for (name, _flat, indented) in CASES {
        let svg = render(indented, &RenderOptions::default())
            .unwrap_or_else(|e| panic!("{name}: не разобран: {e}"));
        assert!(svg.contains("<svg"), "{name}: пустой SVG");
        assert!(svg.len() > 500, "{name}: подозрительно короткий SVG");
    }
}

/// Вложенный `while` внутри `if` — обычная практика, раньше не
/// разбирался. Именно на этом падал эталонный кейс activity_branch.
#[test]
fn test_while_nested_in_if() {
    let source = "@startuml\nstart\nif (файл?) then (да)\n  while (записи?) is (да)\n    :обработать;\n  endwhile (нет)\nelse\n  :создать;\nendif\nstop\n@enduml";
    let svg = render(source, &RenderOptions::default())
        .expect("вложенный while внутри if обязан разбираться");
    assert!(svg.contains("<svg"));
    for word in ["файл", "записи", "обработать", "создать"] {
        assert!(svg.contains(word), "потеряно слово {word}");
    }
}

/// Отступ в несколько уровней — обычная практика вложенности.
#[test]
fn test_deeply_indented_block() {
    let source = "@startuml\nstart\nif (a?) then (да)\n  if (b?) then (да)\n    if (c?) then (нет)\n      :глубоко;\n    endif\n  endif\nendif\nstop\n@enduml";
    let svg = render(source, &RenderOptions::default())
        .expect("тройная вложенность с отступами обязана разбираться");
    assert!(svg.contains("глубоко"));
}

/// Управляющая конструкция без пары: `start` на месте, закрывать нечем.
#[test]
fn test_indented_partition_still_works() {
    let source = "@startuml\nstart\n  partition Задача {\n  :шаг;\n  }\nstop\n@enduml";
    let svg =
        render(source, &RenderOptions::default()).expect("partition с отступом обязан работать");
    assert!(svg.contains("Задача"));
}
