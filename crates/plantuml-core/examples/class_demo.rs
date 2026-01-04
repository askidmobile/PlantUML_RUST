//! Демо: рендеринг class diagram
//!
//! Запуск: cargo run --example class_demo

use plantuml_core::{render, RenderOptions};
use std::fs;

fn main() {
    // Простое наследование (Animal -> Dog, Cat)
    let inheritance_source = r#"@startuml
class Animal {
    +eat()
}

class Dog {
    +bark()
}

class Cat {
    +meow()
}

Dog --|> Animal
Cat --|> Animal
@enduml"#;

    // Композиция и агрегация
    let composition_source = r#"@startuml
class Car {
    -engine
    -wheels
}

class Engine {
    -power
}

class Wheel {
    -size
}

Car *-- Engine : contains
Car o-- Wheel : has
@enduml"#;

    // Интерфейсы и реализация
    let interface_source = r#"@startuml
interface Repository {
    +save()
    +find()
}

abstract class AbstractRepository {
    #connection
}

class UserRepository {
    +findByName()
}

Repository <|.. AbstractRepository
AbstractRepository <|-- UserRepository
@enduml"#;

    // Кардинальности (мультипликативность)
    let cardinality_source = r#"@startuml
class User {
    +id: int
    +name: string
}

class Order {
    +id: int
    +total: decimal
}

User "1" -- "*" Order : places
@enduml"#;

    // Рендерим все примеры
    let options = RenderOptions::default();

    println!("Рендеринг class diagrams...\n");

    // 1. Наследование
    match render(inheritance_source, &options) {
        Ok(svg) => {
            fs::write("output_class_inheritance.svg", &svg).expect("Не удалось записать файл");
            println!(
                "✓ Наследование: output_class_inheritance.svg ({} байт)",
                svg.len()
            );
        }
        Err(e) => println!("✗ Ошибка наследования: {}", e),
    }

    // 2. Композиция/агрегация
    match render(composition_source, &options) {
        Ok(svg) => {
            fs::write("output_class_composition.svg", &svg).expect("Не удалось записать файл");
            println!(
                "✓ Композиция: output_class_composition.svg ({} байт)",
                svg.len()
            );
        }
        Err(e) => println!("✗ Ошибка композиции: {}", e),
    }

    // 3. Интерфейсы
    match render(interface_source, &options) {
        Ok(svg) => {
            fs::write("output_class_interface.svg", &svg).expect("Не удалось записать файл");
            println!(
                "✓ Интерфейсы: output_class_interface.svg ({} байт)",
                svg.len()
            );
        }
        Err(e) => println!("✗ Ошибка интерфейсов: {}", e),
    }

    // 4. Кардинальности
    match render(cardinality_source, &options) {
        Ok(svg) => {
            fs::write("output_class_cardinality.svg", &svg).expect("Не удалось записать файл");
            println!(
                "✓ Кардинальности: output_class_cardinality.svg ({} байт)",
                svg.len()
            );
        }
        Err(e) => println!("✗ Ошибка кардинальностей: {}", e),
    }

    println!("\nГотово! Откройте SVG файлы в браузере для просмотра.");
}
