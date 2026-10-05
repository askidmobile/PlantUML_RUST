//! Визуальные тесты для class diagrams
//!
//! Используем insta для snapshot тестирования SVG вывода.

use plantuml_core::{render, RenderOptions};

/// Тест простой class diagram
#[test]
fn test_simple_class_svg() {
    let source = r#"@startuml
class User {
    -id: Long
    -name: String
    +getId(): Long
    +getName(): String
}
@enduml"#;

    let svg = render(source, &RenderOptions::default()).unwrap();

    // Проверяем что SVG валидный
    assert!(svg.contains("<?xml"));
    assert!(svg.contains("<svg"));
    assert!(svg.contains("User"));

    // Snapshot тест
    insta::assert_snapshot!("simple_class", svg);
}

/// Тест наследования
#[test]
fn test_inheritance_svg() {
    let source = r#"@startuml
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

    let svg = render(source, &RenderOptions::default()).unwrap();

    assert!(svg.contains("Animal"));
    assert!(svg.contains("Dog"));
    assert!(svg.contains("Cat"));

    insta::assert_snapshot!("inheritance", svg);
}

/// Тест интерфейса и реализации
#[test]
fn test_interface_svg() {
    let source = r#"@startuml
interface Serializable {
    +serialize(): String
}

class User {
    -name: String
    +serialize(): String
}

User ..|> Serializable
@enduml"#;

    let svg = render(source, &RenderOptions::default()).unwrap();

    assert!(svg.contains("Serializable"));
    assert!(svg.contains("User"));

    insta::assert_snapshot!("interface", svg);
}

/// Тест композиции и агрегации
#[test]
fn test_composition_aggregation_svg() {
    let source = r#"@startuml
class Car {
    -engine: Engine
    -wheels: List<Wheel>
}

class Engine {
    -power: int
}

class Wheel {
    -size: int
}

Car *-- Engine : contains
Car o-- Wheel : has
@enduml"#;

    let svg = render(source, &RenderOptions::default()).unwrap();

    assert!(svg.contains("Car"));
    assert!(svg.contains("Engine"));
    assert!(svg.contains("Wheel"));

    insta::assert_snapshot!("composition_aggregation", svg);
}

/// Тест сложной иерархии
#[test]
fn test_complex_hierarchy_svg() {
    let source = r#"@startuml
interface Repository<T> {
    +findById(id): T
    +save(entity): T
}

abstract class AbstractRepository<T> {
    #entityClass: Class
    +findById(id): T
}

class UserRepository {
    +findByName(name): User
}

class ProductRepository {
    +findByCategory(cat): List<Product>
}

AbstractRepository ..|> Repository
UserRepository --|> AbstractRepository
ProductRepository --|> AbstractRepository
@enduml"#;

    let svg = render(source, &RenderOptions::default()).unwrap();

    assert!(svg.contains("Repository"));
    assert!(svg.contains("AbstractRepository"));
    assert!(svg.contains("UserRepository"));
    assert!(svg.contains("ProductRepository"));

    insta::assert_snapshot!("complex_hierarchy", svg);
}

/// `left to right direction` меняет оси раскладки.
///
/// Проверено на сервере: для `class A; class B; A --> B` эталон даёт
/// 63x178 без директивы (рамки друг под другом) и 165x70 с ней (рамки
/// рядом). Направление сохранялось в AST, но раскладка его не читала,
/// поэтому вывод был одинаковым в обоих случаях.
#[test]
fn test_left_to_right_direction_transposes_layout() {
    let vertical = "@startuml\nclass A\nclass B\nA --> B\n@enduml";
    let horizontal = "@startuml\nleft to right direction\nclass A\nclass B\nA --> B\n@enduml";

    let size_of = |source: &str| -> (f64, f64) {
        let svg = render(source, &RenderOptions::default()).expect("диаграмма должна рисоваться");
        let view_box: Vec<f64> = svg
            .split("viewBox=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .map(|value| {
                value
                    .split_whitespace()
                    .filter_map(|part| part.parse().ok())
                    .collect()
            })
            .unwrap_or_default();
        (view_box[2], view_box[3])
    };

    let (vertical_width, vertical_height) = size_of(vertical);
    let (horizontal_width, horizontal_height) = size_of(horizontal);

    assert!(
        vertical_width < horizontal_width,
        "без директивы диаграмма должна быть УЗКОЙ: {vertical_width} против {horizontal_width}"
    );
    assert!(
        vertical_height > horizontal_height,
        "без директивы диаграмма должна быть ВЫСОКОЙ: {vertical_height} против {horizontal_height}"
    );
}
