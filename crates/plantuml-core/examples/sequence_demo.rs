//! Демо: рендеринг sequence diagram
//!
//! Запуск: cargo run --example sequence_demo

use plantuml_core::{render, RenderOptions};
use std::fs;

fn main() {
    // Простая sequence diagram (как в документации PlantUML)
    let simple_source = r#"@startuml
Alice -> Bob: Authentication Request
Bob --> Alice: Authentication Response

Alice -> Bob: Another authentication Request
Alice <-- Bob: Another authentication Response
@enduml"#;

    // Диаграмма с фрагментами
    let fragment_source = r#"@startuml
participant User
participant Server
participant Database

User -> Server: Запрос авторизации
activate Server

alt Успешная авторизация
    Server -> Database: Проверить пользователя
    Database --> Server: Найден
    Server --> User: Токен
else Ошибка авторизации
    Server --> User: Ошибка 401
end

deactivate Server
@enduml"#;

    // Self-message диаграмма с длинным текстом
    let self_message_source = r#"@startuml
participant "Сервис Обработки" as Processor

Processor -> Processor: Инициализация системы
Processor -> Processor: Валидация входных данных
Processor -> Processor: Обработка запроса
@enduml"#;

    // Диаграмма с boxes (группировка участников)
    let box_source = r#"@startuml
box "Frontend" #LightBlue
participant App
participant Store
end box

box "Backend" #LightGreen
participant API
participant DB
end box

App -> Store: dispatch(login)
Store -> API: POST /auth/login
API -> DB: SELECT user
DB --> API: user data
API --> Store: token
Store --> App: logged in
@enduml"#;

    // Диаграмма с autonumber и return
    let autonumber_source = r#"@startuml
autonumber "[00]"

participant "Потребитель API" as apiConsumer
participant "IGA.SSO" as iam
participant "Integration Platform" as ip
participant "NBA Widget BFF" as bff

apiConsumer->iam++: Атентификация - Client Credentials flow
autonumber stop
return Технический токен
autonumber resume

apiConsumer->ip++: Запрос к защищенному ресурсу через API + Технический токен
autonumber stop
ip->ip: Валидировать Технический токен и подписку на API
autonumber resume
ip->bff++: Запрос к защищенному ресурсу через API\n+ Данные подписки (витрины)
bff->bff: Определение витрины
bff->bff: Найти по clientId и issuer\nзапись о файле скрипта для витрины
bff->bff: Обработать запрос и вернуть запрашиваемый ресурс\n(скрипт NBA Widget FE для определенной витрины)
autonumber stop
return Запрашиваемый ресурс
autonumber resume
autonumber stop
return Запрашиваемый ресурс
autonumber resume
@enduml"#;

    // Рендерим все примеры
    let options = RenderOptions::default();

    println!("Рендеринг sequence diagrams...\n");

    // 1. Простая диаграмма
    match render(simple_source, &options) {
        Ok(svg) => {
            fs::write("output_simple.svg", &svg).expect("Не удалось записать файл");
            println!(
                "✓ Простая диаграмма: output_simple.svg ({} байт)",
                svg.len()
            );
        }
        Err(e) => println!("✗ Ошибка простой диаграммы: {}", e),
    }

    // 2. Диаграмма с фрагментами
    match render(fragment_source, &options) {
        Ok(svg) => {
            fs::write("output_fragments.svg", &svg).expect("Не удалось записать файл");
            println!(
                "✓ Диаграмма с фрагментами: output_fragments.svg ({} байт)",
                svg.len()
            );
        }
        Err(e) => println!("✗ Ошибка диаграммы с фрагментами: {}", e),
    }

    // 3. Self-message
    match render(self_message_source, &options) {
        Ok(svg) => {
            fs::write("output_self_message.svg", &svg).expect("Не удалось записать файл");
            println!(
                "✓ Self-message диаграмма: output_self_message.svg ({} байт)",
                svg.len()
            );
        }
        Err(e) => println!("✗ Ошибка self-message: {}", e),
    }

    // 4. Диаграмма с boxes
    match render(box_source, &options) {
        Ok(svg) => {
            fs::write("output_boxes.svg", &svg).expect("Не удалось записать файл");
            println!(
                "✓ Диаграмма с boxes: output_boxes.svg ({} байт)",
                svg.len()
            );
        }
        Err(e) => println!("✗ Ошибка boxes: {}", e),
    }

    // 5. Диаграмма с autonumber и return
    match render(autonumber_source, &options) {
        Ok(svg) => {
            fs::write("output_autonumber.svg", &svg).expect("Не удалось записать файл");
            println!(
                "✓ Диаграмма с autonumber: output_autonumber.svg ({} байт)",
                svg.len()
            );
        }
        Err(e) => println!("✗ Ошибка autonumber: {}", e),
    }

    println!("\nГотово! Откройте SVG файлы в браузере для просмотра.");
}
