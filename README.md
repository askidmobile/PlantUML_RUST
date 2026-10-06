# plantuml-rs

**Pure Rust библиотека для рендеринга UML диаграмм, полностью совместимая с PlantUML**

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![CI](https://github.com/askidmobile/PlantUML_RUST/workflows/CI/badge.svg)](https://github.com/askidmobile/PlantUML_RUST/actions)

## 🎮 Попробовать онлайн

**[▶️ Открыть Playground](https://askidmobile.github.io/PlantUML_RUST/playground/)** — интерактивный редактор для тестирования диаграмм прямо в браузере!

---

## Особенности

- **Совместимость с PlantUML** — 16 типов диаграмм, сверка с эталоном
  по 20 контрольным кейсам; полный синтаксис PlantUML не покрыт
- **Pure Rust** — без зависимостей от C/C++ библиотек
- **WASM поддержка** — работает в браузере через WebAssembly
- **SVG вывод** — векторная графика высокого качества
- **PNG вывод** — растеризация через resvg/tiny-skia
- **Все типы диаграмм** — UML и non-UML диаграммы
- **Стандартная библиотека** — 576 встроенных включений: aws, azure, C4,
  kubernetes, archimate, tupadr3, office, logos
- **Современный Rust** — типизированные ошибки, MSRV 1.83

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
plantuml-core = "0.2"
```

Для PNG-вывода включите feature `png`:

```toml
[dependencies]
plantuml-core = { version = "0.2", features = ["png"] }
```

## Использование

### Базовый пример

```rust
use plantuml_core::{render, RenderOptions};

fn main() {
    let source = r#"
@startuml
Alice -> Bob: Привет!
Bob --> Alice: Привет!
@enduml
"#;

    let svg = render(source, &RenderOptions::default()).unwrap();
    println!("{}", svg);
}
```

### Sequence Diagram

```rust
use plantuml_core::{render, RenderOptions};

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

let svg = render(source, &RenderOptions::default()).unwrap();
```

### Class Diagram

```rust
use plantuml_core::{render, RenderOptions};

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

let svg = render(source, &RenderOptions::default()).unwrap();
```

### WASM (в браузере)

```javascript
import init, { render } from './pkg/plantuml_wasm.js';

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

Замеры лежат в `crates/plantuml-core/benches/pipeline.rs` и запускаются
командой:

```bash
cargo bench -p plantuml-core
```

Бенчмарк написан без `criterion`: он тянет десятки зависимостей, часть
из которых плохо собирается под `wasm32-unknown-unknown`, а проект
WASM-ориентирован. Замеряется полный путь Source → Preprocessor → Parser →
Layout → Renderer, по 200 итераций на диаграмму после прогрева.

Порядок величин на машине разработчика (release, Apple Silicon):

| Диаграмма | Среднее время |
|---|---|
| sequence, 10 сообщений | ~110 мкс |
| sequence с фрагментами | ~125 мкс |
| class с иерархией | ~120 мкс |
| activity с ветвлением | ~100 мкс |
| state с переходами | ~60 мкс |
| gantt с зависимостями | ~230 мкс |
| json таблица | ~75 мкс |
| mindmap | ~75 мкс |

Это ориентир, а не гарантия: цифры зависят от машины и версии Rust.
Абсолютные значения не публикуются как обещание — воспроизводите замер
на своей конфигурации.

---

## Разработка

### Требования

- Rust 1.83+ (требование `pest 2.8.4`)
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
├── plantuml-parser/     # Грамматики pest и разбор
├── plantuml-ast/        # AST типы
├── plantuml-preprocessor/ # Препроцессор
├── plantuml-model/      # Геометрические примитивы (Point, Rect, Size)
├── plantuml-layout/     # Layout engines
├── plantuml-renderer/   # SVG/PNG рендеринг
├── plantuml-themes/     # Темы
├── plantuml-stdlib/     # Стандартная библиотека
└── plantuml-wasm/       # WASM биндинги
```

---

## Верификация совместимости

Заявленная цель проекта — **100% визуальная идентичность оригинальному
PlantUML**. Это цель, а не текущее состояние, и она измеряется.

Каталог `tests/golden/` содержит golden-харнесс: эталоны, снятые с
официального сервера PlantUML, и тесты сравнения.

```bash
# Снять/обновить эталоны (нужен доступ к plantuml.com)
python3 tests/golden/fetch_references.py

# Сравнить наш рендер с эталонами
cargo test -p plantuml-core --test golden_tests

# Метрика прогресса — суммарное расхождение
cargo test -p plantuml-core --test golden_tests golden_report -- --nocapture
```

Сравнение идёт по измеримым характеристикам (габариты, сохранность
подписей), а не байтово: PlantUML иначе расставляет атрибуты, использует
CSS-классы и инлайновые `<polygon>` вместо `<marker>`.

Текущий уровень расхождений зафиксирован в `tests/golden/baseline.json`
по модели «храповика»: тест падает, если расхождение **выросло**. Так
регрессия не проходит незамеченной, а прогресс виден по уменьшению
baseline. Подробности — в [tests/golden/README.md](tests/golden/README.md).

Актуальный разбор расхождений и план их устранения: [docs/AUDIT.md](docs/AUDIT.md).

---

## Roadmap

- [x] Фаза 0: Инфраструктура
- [x] Фаза 1: Sequence + Class Diagrams
- [x] Фаза 2: Activity + State + Component
- [x] Фаза 3: Остальные UML диаграммы
- [x] Фаза 4: Non-UML диаграммы
- [x] Фаза 5: WASM биндинги
- [x] Playground с GitHub Pages
- [x] Инфраструктура верификации (golden-харнесс с эталонами PlantUML)
- [ ] Достижение визуальной идентичности с PlantUML (идёт работа)
- [ ] Публикация на crates.io

Подробный план: [docs/PLAN.md](docs/PLAN.md)

### Текущий статус (v0.2.0)

| Компонент | Статус |
|-----------|--------|
| Парсинг (16 типов диаграмм) | ✅ |
| Layout engines (16) | ✅ |
| SVG рендеринг | ✅ |
| PNG рендеринг | ⚠️ Требует feature `png` |
| WASM сборка | ✅ |
| Препроцессор | ✅ |
| Стандартная библиотека (576 включений) | ✅ |
| Инфраструктура верификации | ✅ |
| Визуальная идентичность PlantUML | 🔄 В работе, 10 из 20 кейсов точны |

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
