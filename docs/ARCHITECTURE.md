# Архитектура plantuml-rs

## Обзор

plantuml-rs — это модульная библиотека для рендеринга UML диаграмм. Архитектура разделена на независимые crates, каждый из которых отвечает за свою область.

---

## Диаграмма компонентов

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              plantuml-core                                   │
│                        (Публичный API / Фасад)                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  render(source: &str) -> Result<String, Error>                              │
│  render_to_png(source: &str) -> Result<Vec<u8>, Error>                      │
│  parse(source: &str) -> Result<Diagram, Error>                              │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
          ┌──────────────────────────┼──────────────────────────┐
          │                          │                          │
          ▼                          ▼                          ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   plantuml-     │     │   plantuml-     │     │   plantuml-     │
│  preprocessor   │────▶│     parser      │────▶│      ast        │
├─────────────────┤     ├─────────────────┤     ├─────────────────┤
│ • !include      │     │ • Lexer (logos) │     │ • Diagram enum  │
│ • !define       │     │ • Parser (pest) │     │ • Sequence AST  │
│ • !if/!else     │     │ • Грамматики    │     │ • Class AST     │
│ • !function     │     │                 │     │ • Activity AST  │
│ • %builtins     │     │                 │     │ • State AST     │
└─────────────────┘     └─────────────────┘     │ • ...           │
                                                └────────┬────────┘
                                                         │
                                                         ▼
                                               ┌─────────────────┐
                                               │   plantuml-     │
                                               │     model       │
                                               ├─────────────────┤
                                               │ • Типизированные│
                                               │   модели        │
                                               │ • Валидация     │
                                               │ • Трансформации │
                                               └────────┬────────┘
                                                        │
          ┌─────────────────────────────────────────────┼───────────────┐
          │                                             │               │
          ▼                                             ▼               ▼
┌─────────────────┐                        ┌─────────────────┐  ┌─────────────┐
│   plantuml-     │                        │   plantuml-     │  │  plantuml-  │
│     layout      │                        │     themes      │  │   stdlib    │
├─────────────────┤                        ├─────────────────┤  ├─────────────┤
│ • SequenceLayout│                        │ • Темы          │  │ • AWS icons │
│ • Sugiyama      │                        │ • skinparam     │  │ • Azure     │
│ • Flowchart     │                        │ • Стили         │  │ • K8s       │
│ • Tree          │                        │                 │  │ • C4        │
│ • Grid          │                        │                 │  │ • ...       │
└────────┬────────┘                        └────────┬────────┘  └──────┬──────┘
         │                                          │                  │
         └──────────────────────┬───────────────────┴──────────────────┘
                                │
                                ▼
                      ┌─────────────────┐
                      │   plantuml-     │
                      │    renderer     │
                      ├─────────────────┤
                      │ • SVG (svg)     │
                      │ • PNG (resvg)   │
                      │ • ASCII         │
                      │ • Shapes        │
                      │ • Text/Fonts    │
                      └────────┬────────┘
                               │
                               ▼
                      ┌─────────────────┐
                      │   plantuml-     │
                      │      wasm       │
                      ├─────────────────┤
                      │ • wasm-bindgen  │
                      │ • JS API        │
                      │ • NPM package   │
                      └─────────────────┘
```

---

## Описание crates

### plantuml-core

**Назначение**: Публичный API и фасад библиотеки.

**Зависимости**: Все остальные crates.

**Ключевые функции**:
```rust
pub fn render(source: &str) -> Result<String, Error>;
pub fn render_with_options(source: &str, options: RenderOptions) -> Result<String, Error>;
pub fn render_to_png(source: &str) -> Result<Vec<u8>, Error>;
pub fn parse(source: &str) -> Result<Diagram, Error>;
```

---

### plantuml-preprocessor

**Назначение**: Обработка директив препроцессора перед парсингом.

**Зависимости**: Минимальные (только std).

**Функциональность**:
- `!include <file>` / `!include_once`
- `!define` / `!undef`
- `!ifdef` / `!ifndef` / `!else` / `!endif`
- `!$variable = value`
- `!function` / `!procedure` / `!return`
- `!theme <name>`
- Builtin функции: `%date()`, `%version()`, `%filename()`, и 50+ других

**Пример**:
```rust
pub fn preprocess(source: &str, resolver: &dyn FileResolver) -> Result<String, PreprocessError>;
```

---

### plantuml-parser

**Назначение**: Лексический и синтаксический анализ.

**Зависимости**: `logos`, `pest`, `pest_derive`, `plantuml-ast`.

**Структура**:
```
plantuml-parser/
├── src/
│   ├── lib.rs
│   ├── lexer.rs           # Logos лексер
│   ├── grammars/          # Pest грамматики
│   │   ├── common.pest    # Общие правила
│   │   ├── sequence.pest
│   │   ├── class.pest
│   │   ├── activity.pest
│   │   └── ...
│   └── parsers/           # Парсеры для каждого типа
│       ├── mod.rs
│       ├── sequence.rs
│       ├── class.rs
│       └── ...
```

**Подход**: Двухфазный парсинг
1. **Лексер (logos)**: Быстрая токенизация
2. **Парсер (pest)**: PEG-грамматика для структуры

---

### plantuml-ast

**Назначение**: Типы AST для всех типов диаграмм.

**Зависимости**: Минимальные (`serde` для сериализации).

**Основная структура**:
```rust
pub enum Diagram {
    Sequence(SequenceDiagram),
    Class(ClassDiagram),
    Activity(ActivityDiagram),
    State(StateDiagram),
    Component(ComponentDiagram),
    Deployment(DeploymentDiagram),
    UseCase(UseCaseDiagram),
    Object(ObjectDiagram),
    Timing(TimingDiagram),
    Gantt(GanttDiagram),
    MindMap(MindMapDiagram),
    Wbs(WbsDiagram),
    Json(JsonDiagram),
    Yaml(YamlDiagram),
    Network(NetworkDiagram),
    Salt(SaltDiagram),
    Er(ErDiagram),
    Archimate(ArchimateDiagram),
}
```

---

### plantuml-model

**Назначение**: Типизированные модели для layout и рендеринга.

**Зависимости**: `plantuml-ast`.

**Задачи**:
- Преобразование AST в layout-модели
- Валидация семантики
- Разрешение ссылок

---

### plantuml-layout

**Назначение**: Алгоритмы автоматического размещения элементов.

**Зависимости**: `petgraph`, `plantuml-model`.

**Layout engines** (реальные имена из `plantuml_layout`):

| Engine | Диаграммы | Алгоритм |
|--------|-----------|----------|
| `SequenceLayoutEngine` | Sequence | Двухпроходный расчёт ширин участников и spacing |
| `ClassLayoutEngine` | Class | Sugiyama (слои, барицентр, ортогональные рёбра) |
| `ActivityLayoutEngine` | Activity | Линейный расклад с ветвлениями и swimlane |
| `StateLayoutEngine` | State | Уровни состояний, вложенные composite |
| `ComponentLayoutEngine` | Component, Deployment, Archimate | Сетка |
| `UseCaseLayoutEngine` | UseCase | Актёры слева, система справа |
| `ObjectLayoutEngine` | Object | Сетка 4 в ряд |
| `TimingLayoutEngine` | Timing | Временная шкала |
| `GanttLayoutEngine` | Gantt | Диаграмма Ганта |
| `MindMapLayoutEngine` | MindMap | Дерево вправо, двухпроходный расчёт высот |
| `WbsLayoutEngine` | WBS | Дерево вниз |
| `JsonLayoutEngine` | JSON | Рекурсивный расклад |
| `YamlLayoutEngine` | YAML | Обёртка над JSON |
| `ErLayoutEngine` | ER | Сетка 3 в ряд |
| `NetworkLayoutEngine` | Network (nwdiag) | Полосы сетей, колонки серверов |
| `SaltLayoutEngine` | Salt | Сеточные контейнеры |

> Deployment и Archimate не имеют собственных движков — оба
> переиспользуют `ComponentLayoutEngine` (см. `plantuml-core/src/pipeline.rs`).

**Трейт**:
```rust
pub trait LayoutEngine {
    type Input;

    fn layout(&self, input: &Self::Input, config: &LayoutConfig) -> LayoutResult;
}
```

> **Известное ограничение:** трейт реализуют 7 движков из 18, остальные
> имеют собственные inherent-методы, и `pipeline.rs` ветвится вручную.
> Параметр `LayoutConfig` пока игнорируется большинством реализаций.
> См. `docs/AUDIT.md`, §4.8.

---

### plantuml-renderer

**Назначение**: Генерация визуального вывода.

**Зависимости**: `svg`; для PNG (feature `png`) — `resvg`, `tiny-skia`, `fontdb`.

**Рендереры**:
- **SvgRenderer**: единственный рендерер, работает через `ElementType`
- **PngRenderer**: растеризация готового SVG через resvg (feature `png`)

**Трейт**:
```rust
pub trait Renderer {
    type Output;
    
    fn render(&self, layout: &LayoutResult, theme: &Theme) -> Self::Output;
}
```

---

### plantuml-themes

**Назначение**: Темы оформления и skinparam.

**Зависимости**: `serde`.

**Функциональность**:
- Встроенные темы (default, sketchy, etc.)
- Пользовательские темы
- skinparam параметры
- Цветовые схемы

---

### plantuml-stdlib

**Назначение**: Стандартная библиотека иконок и спрайтов.

**Зависимости**: Минимальные.

**Содержимое**:
- AWS Architecture Icons
- Azure Icons
- Kubernetes Icons
- C4 Model
- Material Design Icons
- И другие

---

### plantuml-wasm

**Назначение**: WASM биндинги для браузера.

**Зависимости**: `wasm-bindgen`, `plantuml-core`.

**API**:
```javascript
// JavaScript
import init, { render, parse } from 'plantuml-rs';

await init();
const svg = render('@startuml\nAlice -> Bob\n@enduml');
```

---

## Поток данных

```
Source Text
    │
    ▼
┌─────────────────┐
│   Preprocessor  │  Раскрытие !include, !define, etc.
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│     Lexer       │  Токенизация (logos)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│     Parser      │  Синтаксический анализ (pest)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│      AST        │  Абстрактное синтаксическое дерево
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│     Model       │  Семантическая модель
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│     Layout      │  Вычисление позиций
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    Renderer     │  Генерация SVG/PNG
└────────┬────────┘
         │
         ▼
    Output (SVG/PNG)
```

---

## Принципы проектирования

### 1. Модульность
Каждый crate имеет чёткую ответственность и минимальные зависимости.

### 2. Расширяемость
Новые типы диаграмм добавляются через:
- Новый вариант в `Diagram` enum
- Новая грамматика в `plantuml-parser`
- Новый layout engine в `plantuml-layout`

### 3. WASM-совместимость
Все зависимости выбраны с учётом `wasm32-unknown-unknown` target.

### 4. Pure Rust
Никаких C/C++ зависимостей. Вся логика реализована на Rust.

### 5. Тестируемость
- Unit тесты в каждом модуле
- Integration тесты для full pipeline
- Visual regression тесты для рендеринга

---

## Зависимости между crates

```
plantuml-core
    ├── plantuml-parser
    │       ├── plantuml-ast
    │       └── plantuml-preprocessor
    ├── plantuml-model
    │       └── plantuml-ast
    ├── plantuml-layout
    │       └── plantuml-model
    ├── plantuml-renderer
    │       ├── plantuml-layout
    │       └── plantuml-themes
    ├── plantuml-themes
    └── plantuml-stdlib

plantuml-wasm
    └── plantuml-core
```

---

## Feature flags

Реальные флаги (см. `crates/plantuml-core/Cargo.toml`):

```toml
[features]
default = []                # без дополнительных возможностей
serde = ["dep:serde"]       # сериализация AST
png = ["plantuml-renderer/png"]  # PNG через resvg + tiny-skia + fontdb
```

> Флагов `svg`, `wasm`, `all-diagrams`, `sequence`, `class` не существует:
> SVG-рендеринг всегда доступен, WASM собирается отдельным крейтом
> `plantuml-wasm`, а все типы диаграмм включены всегда.
> PNG по умолчанию **выключен** — требуется `features = ["png"]`.
