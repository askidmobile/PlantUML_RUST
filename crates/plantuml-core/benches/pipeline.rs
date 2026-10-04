//! Замеры времени работы pipeline.
//!
//! Бенчмарки написаны без `criterion`: он тянет за собой десятки
//! зависимостей, часть из которых плохо собирается под `wasm32-unknown-unknown`,
//! а сам проект WASM-ориентирован. Здесь достаточно `std::time::Instant`.
//!
//! Запуск:
//! ```text
//! cargo bench -p plantuml-core
//! ```
//!
//! Замеряется полный путь Source → Preprocessor → Parser → Layout → Renderer
//! на диаграммах разных типов: узкие места у них разные (парсинг против
//! раскладки против сериализации SVG).

use std::hint::black_box;
use std::time::Instant;

use plantuml_core::{render, RenderOptions};

/// Сколько раз повторять замер для одной диаграммы.
const ITERATIONS: u32 = 200;

/// Диаграмма для замера: имя и исходник.
struct Case {
    name: &'static str,
    source: &'static str,
}

/// Набор диаграмм — по одной на каждый тяжёлый тип.
const CASES: &[Case] = &[
    Case {
        name: "sequence (10 сообщений)",
        source: "@startuml\nparticipant A\nparticipant B\nparticipant C\nA -> B: запрос\nB -> C: запрос\nC -> B: ответ\nB -> A: ответ\nA -> C: напрямую\nC -> A: напрямую\nB -> A: снова\n@enduml",
    },
    Case {
        name: "sequence с фрагментами",
        source: "@startuml\nAlice -> Bob: запрос\nalt успех\n  Bob -> Alice: ответ\nelse ошибка\n  Bob -> Alice: отказ\nend\nloop 3 раза\n  Alice -> Bob: повтор\nend\n@enduml",
    },
    Case {
        name: "class с иерархией",
        source: "@startuml\nclass Animal {\n  +eat()\n}\nclass Dog {\n  +bark()\n}\nclass Cat {\n  +meow()\n}\nclass Puppy {\n  +play()\n}\nAnimal <|-- Dog\nAnimal <|-- Cat\nDog <|-- Puppy\n@enduml",
    },
    Case {
        name: "activity с ветвлением",
        source: "@startuml\nstart\n:Первый шаг;\nif (Условие) then (да)\n  :Ветка да;\nelseif (Второе) then (может)\n  :Ветка может;\nelse (нет)\n  :Ветка нет;\nendif\n:Последний шаг;\nstop\n@enduml",
    },
    Case {
        name: "state с переходами",
        source: "@startuml\n[*] --> Active\nActive --> Inactive : timeout\nInactive --> Active : resume\nActive --> [*] : close\n@enduml",
    },
    Case {
        name: "gantt с зависимостями",
        source: "@startgantt\nProject starts 2024-01-01\n[T1] lasts 10 days\n[T2] lasts 20 days\n[T2] starts at [T1]'s end\n[T3] lasts 5 days\n[T3] starts at [T2]'s end\n@endgantt",
    },
    Case {
        name: "json таблица",
        source: "@startjson\n{\"name\": \"Проект\", \"version\": 1, \"tags\": [\"a\", \"b\"], \"active\": true}\n@endjson",
    },
    Case {
        name: "mindmap",
        source: "@startmindmap\n* Проект\n** Планирование\n*** Сроки\n*** Бюджет\n** Реализация\n*** Код\n*** Тесты\n@endmindmap",
    },
];

fn main() {
    let options = RenderOptions::default();

    println!("Замеры pipeline: {ITERATIONS} итераций на диаграмму\n");
    println!(
        "{:<26} {:>12} {:>12}",
        "диаграмма", "среднее, мкс", "размер, байт"
    );
    println!("{}", "-".repeat(52));

    for case in CASES {
        // Прогрев: первый прогон включает ленивую инициализацию и
        // заполнение кэшей, его нельзя включать в замер.
        let warmup = render(case.source, &options).expect("диаграмма должна рендериться");
        black_box(&warmup);

        let started = Instant::now();
        for _ in 0..ITERATIONS {
            let svg = render(black_box(case.source), &options).expect("рендер не должен падать");
            black_box(&svg);
        }
        let elapsed = started.elapsed();

        let per_iteration_us = elapsed.as_secs_f64() * 1_000_000.0 / f64::from(ITERATIONS);
        println!(
            "{:<26} {:>12.1} {:>12}",
            case.name,
            per_iteration_us,
            warmup.len()
        );
    }
}
