//! Демонстрация рендеринга Activity Diagrams
//!
//! Запуск: cargo run -p plantuml-core --example activity_demo

use plantuml_core::{render, RenderOptions};
use std::fs;

fn main() {
    // Пример 1: Простая activity диаграмма
    let simple = r#"
@startuml
start
:Hello world;
:This is defined on
temporary lines;
stop
@enduml
"#;

    // Пример 2: Условие if/else
    let condition = r#"
@startuml
start
if (Graphviz installed?) then (yes)
  :process all diagrams;
else (no)
  :process only sequence diagrams;
endif
stop
@enduml
"#;

    // Пример 3: While цикл
    let while_loop = r#"
@startuml
start
while (data available?)
  :read data;
  :generate diagram;
endwhile
stop
@enduml
"#;

    // Пример 4: Fork/Join
    let fork = r#"
@startuml
start
fork
  :action 1;
fork again
  :action 2;
end fork
stop
@enduml
"#;

    // Пример 5: Swimlanes
    let swimlanes = r#"
@startuml
|Swimlane1|
start
:foo1;
|#AntiqueWhite|Swimlane2|
:foo2;
:foo3;
|Swimlane1|
:foo4;
stop
@enduml
"#;

    // Создаём директорию для вывода
    let output_dir = "target/activity_examples";
    fs::create_dir_all(output_dir).unwrap();

    let examples = [
        ("simple", simple),
        ("condition", condition),
        ("while_loop", while_loop),
        ("fork", fork),
        ("swimlanes", swimlanes),
    ];

    let options = RenderOptions::default();

    for (name, source) in examples {
        println!("Рендеринг: {}", name);

        match render(source, &options) {
            Ok(svg) => {
                let path = format!("{}/{}.svg", output_dir, name);
                fs::write(&path, &svg).unwrap();
                println!("  ✓ Сохранено: {}", path);
            }
            Err(e) => {
                println!("  ✗ Ошибка: {}", e);
            }
        }
    }

    println!("\nВсе примеры сохранены в {}/", output_dir);
}
