# AGENTS.md — Руководство для AI-агентов

## Проект: plantuml-rs

Pure Rust библиотека для рендеринга UML диаграмм, совместимая с PlantUML.

## КРИТИЧЕСКИ ВАЖНО

- **Русский язык** — для всех коммуникаций, комментариев кода и документации
- **Pure Rust** — никаких C/C++ зависимостей
- **WASM-совместимость** — код должен компилироваться для `wasm32-unknown-unknown`
- **100% совместимость с PlantUML** — все рендеры ОБЯЗАТЕЛЬНО сравнивать с оригинальным PlantUML и добиваться ТОЧНОГО соответствия (не "близко", а идентично)

## ВЕРИФИКАЦИЯ РЕНДЕРИНГА

При работе над рендерингом диаграмм ОБЯЗАТЕЛЬНО:

1. **Сгенерировать эталон** на оригинальном PlantUML:
   - Открыть https://www.plantuml.com/plantuml/uml/
   - Ввести тестовый код диаграммы
   - Сделать скриншот результата

2. **Сравнить с нашим рендером**:
   - Запустить `cargo run -p plantuml-core --example sequence_demo`
   - Открыть сгенерированные SVG файлы
   - Визуально сравнить КАЖДЫЙ элемент

3. **Проверяемые аспекты** (должны быть ИДЕНТИЧНЫ):
   - Размеры и пропорции элементов (boxes, петли, стрелки)
   - Позиция текста относительно линий/элементов
   - Расстояния между элементами
   - Стиль линий (сплошные, пунктирные)
   - Цвета и заливки
   - Форма стрелок

4. **Критерий приёмки**: визуально неотличимо от оригинала

## 1. КОМАНДЫ СБОРКИ И ТЕСТИРОВАНИЯ

```bash
# Сборка и проверка
cargo build --workspace
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all

# Тесты
cargo test --workspace                              # все тесты
cargo test -p plantuml-parser                       # тесты одного crate
cargo test -p plantuml-parser test_parse_basic      # один тест по имени
cargo test -p plantuml-core --test sequence_tests   # интеграционные тесты

# WASM
cargo build --target wasm32-unknown-unknown -p plantuml-wasm

# Примеры и документация
cargo run -p plantuml-core --example sequence_demo
cargo doc --workspace --open
```

## 2. СТРУКТУРА ПРОЕКТА

```
crates/
├── plantuml-core/       # Главный фасад — публичный API (render, parse)
├── plantuml-ast/        # AST типы (SequenceDiagram, ClassDiagram, etc.)
├── plantuml-parser/     # Парсер (pest)
├── plantuml-preprocessor/  # Директивы (!include, !define, %date())
├── plantuml-model/      # Геометрические примитивы (Point, Rect, Size)
├── plantuml-layout/     # Layout engines (SequenceLayoutEngine, etc.)
├── plantuml-renderer/   # SVG рендеринг
├── plantuml-themes/     # Темы и skinparam
├── plantuml-stdlib/     # Стандартная библиотека PlantUML
└── plantuml-wasm/       # WASM биндинги
```

## 3. СТИЛЬ КОДА

### 3.1 Импорты
```rust
// Порядок: std → external crates → internal crates → local modules
use std::collections::HashMap;

use indexmap::IndexMap;
use thiserror::Error;

use plantuml_ast::sequence::SequenceDiagram;
use plantuml_model::{Point, Rect};

use crate::config::LayoutConfig;
use super::metrics::DiagramMetrics;
```

### 3.2 Типизированные ошибки
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("неожиданный токен: {0}")]
    UnexpectedToken(String),
    
    #[error("синтаксическая ошибка в строке {line}: {message}")]
    SyntaxError { line: usize, message: String },
}

pub type Result<T> = std::result::Result<T, ParseError>;
```

### 3.3 Документация (на русском)
```rust
/// Парсит PlantUML исходный код и возвращает AST диаграммы.
///
/// # Аргументы
/// * `source` - исходный код PlantUML
///
/// # Возвращает
/// * `Ok(Diagram)` - распарсенная диаграмма
/// * `Err(ParseError)` - ошибка парсинга
pub fn parse(source: &str) -> Result<Diagram> { ... }
```

### 3.4 Именование

- **Структуры/Enum**: `PascalCase` — `SequenceDiagram`, `ParticipantType`
- **Функции/методы**: `snake_case` — `parse_sequence`, `add_participant`
- **Константы**: `SCREAMING_SNAKE_CASE` — `DEFAULT_SPACING`
- **Модули/файлы**: `snake_case` — `sequence.rs`, `svg_renderer.rs`

### 3.5 Структура модулей
```rust
//! Документация модуля на русском

mod submodule;
pub use submodule::PublicType;

pub struct MainType { ... }

impl MainType {
    pub fn new() -> Self { ... }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature() { ... }
}
```

## 4. ТЕСТИРОВАНИЕ

### Snapshot тесты (insta)
```rust
#[test]
fn test_sequence_render_svg() {
    let svg = render("@startuml\nAlice -> Bob\n@enduml", &RenderOptions::default()).unwrap();
    insta::assert_snapshot!("simple_sequence", svg);
}
```

Принятие snapshots:
```bash
cd crates/plantuml-core/tests/snapshots
for f in *.snap.new; do mv "$f" "${f%.new}"; done
```

## 5. GIT WORKFLOW

```bash
# Формат коммитов (Conventional Commits на русском)
git commit -m "feat(parser): добавлен парсинг sequence diagrams"
git commit -m "fix(layout): исправлен расчёт позиции lifeline"
git commit -m "docs: обновлена документация API"
git commit -m "test: добавлены тесты для class diagrams"
git commit -m "refactor(renderer): оптимизация SVG генерации"
```

## 6. WASM-СОВМЕСТИМОСТЬ

При добавлении зависимостей проверять:
- Поддержка `wasm32-unknown-unknown`
- Отсутствие системных вызовов (filesystem, network, threads)
- Для шрифтов: отключать `system_fonts` feature

```toml
# WASM-совместимая конфигурация
[target.'cfg(target_arch = "wasm32")'.dependencies]
fontdb = { version = "0.21", default-features = false }
```

## 7. АРХИТЕКТУРА PIPELINE

```
Source → Preprocessor → Parser → AST → Layout → LayoutResult → Renderer → SVG
           (!include)    (pest)         (engine)               (svg crate)
```

## 8. ДОБАВЛЕНИЕ НОВОГО ТИПА ДИАГРАММЫ

1. Создать AST в `plantuml-ast/src/{type}.rs`
2. Добавить pest грамматику в `plantuml-parser/src/grammars/{type}.pest`
3. Реализовать парсер в `plantuml-parser/src/parsers/{type}.rs`
4. Реализовать layout engine в `plantuml-layout/src/{type}/`
5. Обновить `detect_diagram_type()` в `plantuml-parser/src/lib.rs`
6. Интегрировать в pipeline в `plantuml-core/src/pipeline.rs`
7. Написать тесты (unit + snapshot)

## 9. КЛЮЧЕВЫЕ ЗАВИСИМОСТИ

| Crate | Назначение |
|-------|------------|
| pest | PEG парсер |
| svg | SVG генерация |
| thiserror | Типизированные ошибки |
| serde | Сериализация |
| indexmap | Упорядоченный HashMap |
| insta | Snapshot тестирование |

## 10. РЕСУРСЫ

- PlantUML: https://plantuml.com/
- pest Book: https://pest.rs/book/
