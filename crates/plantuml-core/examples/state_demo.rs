//! Демонстрация рендеринга State Diagrams
//!
//! Запуск: cargo run -p plantuml-core --example state_demo

use plantuml_core::{render, RenderOptions};
use std::fs;
use std::path::Path;

fn main() {
    // Создаём директорию для выходных файлов
    let output_dir = Path::new("target/state_examples");
    fs::create_dir_all(output_dir).expect("Не удалось создать директорию");

    // Пример 1: Простая диаграмма состояний
    let simple = r#"
@startuml
[*] --> Active
Active --> Inactive : timeout
Inactive --> Active : resume
Active --> [*] : close
@enduml
"#;

    // Пример 2: С описаниями переходов
    let with_labels = r#"
@startuml
[*] --> Ready
Ready --> Processing : start [valid] / init()
Processing --> Done : complete
Processing --> Error : fail / log()
Done --> [*]
Error --> Ready : retry
@enduml
"#;

    // Пример 3: С определениями состояний
    let with_definitions = r#"
@startuml
state "Ожидание" as Waiting
state "Обработка" as Processing
state "Завершено" as Done

[*] --> Waiting
Waiting --> Processing : request
Processing --> Done : success
Done --> [*]
@enduml
"#;

    // Пример 4: С choice state
    let with_choice = r#"
@startuml
state choice1 <<choice>>

[*] --> Request
Request --> choice1
choice1 --> Approved : [valid]
choice1 --> Rejected : [invalid]
Approved --> [*]
Rejected --> [*]
@enduml
"#;

    // Пример 5: С fork/join
    let with_fork = r#"
@startuml
state fork1 <<fork>>
state join1 <<join>>

[*] --> fork1
fork1 --> Task1
fork1 --> Task2
Task1 --> join1
Task2 --> join1
join1 --> [*]
@enduml
"#;

    // Пример 6: Составное состояние
    let composite = r#"
@startuml
state Composite {
    [*] --> Inner1
    Inner1 --> Inner2
    Inner2 --> [*]
}

[*] --> Composite
Composite --> [*]
@enduml
"#;

    // Пример 7: Пример из задачи - с retry и abort
    let issue_example = r#"
@startuml
[*] --> Idle

Idle --> Processing : start
Processing --> Completed : success
Processing --> Failed : error
Completed --> [*]
Failed --> Idle : retry
Failed --> [*] : abort
@enduml
"#;

    // Рендерим все примеры
    let examples = [
        ("simple", simple),
        ("with_labels", with_labels),
        ("with_definitions", with_definitions),
        ("with_choice", with_choice),
        ("with_fork", with_fork),
        ("composite", composite),
        ("issue_example", issue_example),
    ];

    let options = RenderOptions::default();

    for (name, source) in examples.iter() {
        match render(source, &options) {
            Ok(svg) => {
                let path = output_dir.join(format!("{}.svg", name));
                fs::write(&path, &svg).expect("Не удалось записать файл");
                println!("✓ Сохранено: {}", path.display());
            }
            Err(e) => {
                eprintln!("✗ Ошибка в примере {}: {:?}", name, e);
            }
        }
    }

    println!("\nГотово! Проверьте файлы в {}", output_dir.display());
}
