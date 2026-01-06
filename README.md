# plantuml-rs

**Pure Rust библиотека для рендеринга UML диаграмм, полностью совместимая с PlantUML**

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![CI](https://github.com/askidmobile/PlantUML_RUST/workflows/CI/badge.svg)](https://github.com/askidmobile/PlantUML_RUST/actions)

## 🎮 Попробовать онлайн

**[▶️ Открыть Playground](https://askidmobile.github.io/PlantUML_RUST/playground/)** — интерактивный редактор для тестирования диаграмм прямо в браузере!

---

## Особенности

- **100% совместимость с PlantUML** — поддержка полного синтаксиса PlantUML
- **Pure Rust** — без зависимостей от C/C++ библиотек
- **WASM поддержка** — работает в браузере через WebAssembly
- **SVG вывод** — векторная графика высокого качества
- **PNG вывод** — растеризация через resvg/tiny-skia
- **Все типы диаграмм** — UML и non-UML диаграммы

## Поддерживаемые диаграммы

### UML диаграммы
- Sequence Diagram
- Class Diagram
- Activity Diagram
- State Diagram
- Component Diagram
- Deployment Diagram
- Use Case Diagram
- Object Diagram
- Timing Diagram

### Non-UML диаграммы
- Gantt Chart
- MindMap
- WBS (Work Breakdown Structure)
- JSON/YAML визуализация
- Network Diagram (nwdiag)
- Salt (Wireframe)
- ER Diagram
- Archimate

---

## Установка

Добавьте в `Cargo.toml`:

```toml
[dependencies]
plantuml-rs = "0.1"
```

## Использование

### Базовый пример

```rust
use plantuml_rs::render;

fn main() {
    let source = r#"
@startuml
Alice -> Bob: Привет!
Bob --> Alice: Привет!
@enduml
"#;

    let svg = render(source).unwrap();
    println!("{}", svg);
}
```

### Sequence Diagram

```rust
use plantuml_rs::render;

let source = r#"
@startuml
participant Alice
participant Bob
participant Charlie

Alice -> Bob: Запрос авторизации
activate Bob

Bob -> Charlie: Проверка токена
activate Charlie
Charlie --> Bob: Токен валиден
deactivate Charlie

Bob --> Alice: Авторизация успешна
deactivate Bob

alt Успех
    Alice -> Bob: Получить данные
    Bob --> Alice: Данные
else Ошибка
    Alice -> Bob: Повторить запрос
end
@enduml
"#;

let svg = render(source).unwrap();
```

### Class Diagram

```rust
use plantuml_rs::render;

let source = r#"
@startuml
abstract class Animal {
    + name: String
    + age: int
    + {abstract} speak(): void
}

class Dog extends Animal {
    + breed: String
    + speak(): void
}

class Cat extends Animal {
    + indoor: bool
    + speak(): void
}

interface Trainable {
    + train(): void
}

Dog ..|> Trainable
@enduml
"#;

let svg = render(source).unwrap();
```

### WASM (в браузере)

```javascript
import init, { render } from 'plantuml-rs';

async function main() {
    await init();
    
    const source = `
@startuml
Alice -> Bob: Hello
@enduml
`;
    
    const svg = render(source);
    document.getElementById('diagram').innerHTML = svg;
}

main();
```

---

## Архитектура

```
┌─────────────┐    ┌──────────────┐    ┌────────┐    ┌──────────┐
│   Source    │───▶│ Preprocessor │───▶│ Parser │───▶│   AST    │
│   Text      │    │              │    │        │    │          │
└─────────────┘    └──────────────┘    └────────┘    └────┬─────┘
                                                          │
                                                          ▼
┌─────────────┐    ┌──────────────┐    ┌────────┐    ┌──────────┐
│    SVG      │◀───│   Renderer   │◀───│ Layout │◀───│  Model   │
│   Output    │    │              │    │        │    │          │
└─────────────┘    └──────────────┘    └────────┘    └──────────┘
```

## Производительность

| Операция | plantuml-rs | PlantUML (Java) |
|----------|-------------|-----------------|
| Простая sequence | ~5ms | ~500ms |
| Сложная class | ~20ms | ~1000ms |
| WASM загрузка | ~50ms | N/A |

*Бенчмарки проводились на M1 MacBook Pro*

---

## Разработка

### Требования

- Rust 1.75+
- wasm-pack (для WASM сборки)

### Быстрый старт

```bash
# Интерактивное меню со всеми командами
./run.sh

# Или выполнить конкретную команду:
./run.sh build      # Сборка проекта
./run.sh test       # Запуск тестов
./run.sh wasm       # Сборка WASM
./run.sh server     # Локальный сервер
./run.sh help       # Справка по командам
```

### Скрипты

Проект содержит набор скриптов в папке `scripts/` для автоматизации рабочих процессов:

| Скрипт | Описание |
|--------|----------|
| `run.sh` | Главное меню (интерактивный выбор действий) |
| `scripts/build.sh` | Полная сборка проекта (clippy + fmt + build + wasm + docs) |
| `scripts/test.sh` | Запуск тестов (all/unit/integration/quick) |
| `scripts/wasm.sh` | Сборка WASM модуля через wasm-pack |
| `scripts/server.sh` | Локальный HTTP-сервер для тестирования |
| `scripts/clean.sh` | Очистка временных файлов и артефактов |
| `scripts/docs.sh` | Генерация документации |
| `scripts/examples.sh` | Запуск примеров диаграмм |
| `scripts/release.sh` | Создание нового релиза |

### Примеры использования скриптов

```bash
# Полная сборка с проверками
./run.sh build

# Только проверка кода (без сборки)
./run.sh check

# Запуск конкретных тестов
./run.sh test plantuml-parser

# WASM сборка и локальный сервер
./run.sh wasm && ./run.sh server 3000

# Создание релиза
./run.sh release 0.3.0

# Очистка всех артефактов
./run.sh clean all
```

### Ручные команды

```bash
# Сборка библиотеки
cargo build --workspace

# Запуск тестов
cargo test --workspace

# Сборка WASM
cargo build --target wasm32-unknown-unknown -p plantuml-wasm

# Документация
cargo doc --workspace --open
```

### Структура проекта

```
crates/
├── plantuml-core/       # Главный фасад
├── plantuml-parser/     # Лексер + парсер
├── plantuml-ast/        # AST типы
├── plantuml-preprocessor/ # Препроцессор
├── plantuml-model/      # Модели диаграмм
├── plantuml-layout/     # Layout engines
├── plantuml-renderer/   # SVG/PNG рендеринг
├── plantuml-themes/     # Темы
├── plantuml-stdlib/     # Стандартная библиотека
└── plantuml-wasm/       # WASM биндинги
```

---

## Roadmap

- [x] Фаза 0: Инфраструктура
- [x] Фаза 1: Sequence + Class Diagrams
- [x] Фаза 2: Activity + State + Component
- [x] Фаза 3: Остальные UML диаграммы
- [x] Фаза 4: Non-UML диаграммы
- [x] Фаза 5: WASM биндинги
- [x] Playground с GitHub Pages
- [ ] Визуальная верификация с оригинальным PlantUML
- [ ] Публикация на crates.io

Подробный план: [docs/PLAN.md](docs/PLAN.md)

### Текущий статус (v0.2.0)

| Компонент | Статус |
|-----------|--------|
| Парсинг (18 типов диаграмм) | ✅ |
| Layout engines | ✅ |
| SVG рендеринг | ✅ |
| PNG рендеринг | ✅ |
| WASM сборка | ✅ |
| Темы (6 тем) | ✅ |
| Препроцессор | ✅ |
| Визуальная сверка | 🔄 В процессе |

---

## Лицензия

Проект доступен под двойной лицензией:

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

Выберите любую на ваше усмотрение.

---

## Благодарности

- [PlantUML](https://plantuml.com/) — за создание отличного инструмента и синтаксиса
- [pest](https://pest.rs/) — за мощный PEG парсер
- [resvg](https://github.com/RazrFalcon/resvg) — за качественный SVG рендеринг

## Вклад в проект

Приветствуются любые вклады! Пожалуйста, ознакомьтесь с [CONTRIBUTING.md](CONTRIBUTING.md) перед отправкой pull request.
