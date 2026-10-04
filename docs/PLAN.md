# ПЛАН РАЗРАБОТКИ: plantuml-rs

## Библиотека на Pure Rust для рендеринга UML диаграмм
### Полная замена PlantUML Core (Java) с поддержкой WASM

---

## 1. ОБЗОР ПРОЕКТА

### 1.1 Цели

| Цель | Описание |
|------|----------|
| **Совместимость** | 100% совместимость с синтаксисом PlantUML |
| **Pure Rust** | Никаких C/C++ зависимостей |
| **WASM** | Работа в браузере через WebAssembly |
| **Формат вывода** | SVG (приоритет), PNG через растеризацию |
| **Все типы диаграмм** | 20+ типов как в оригинальном PlantUML |

### 1.2 Структура репозитория

```
plantuml-rs/
├── Cargo.toml                    # Workspace
├── crates/
│   ├── plantuml-core/           # Главная библиотека (фасад)
│   ├── plantuml-parser/         # Лексер + Парсер
│   ├── plantuml-ast/            # AST типы
│   ├── plantuml-preprocessor/   # Препроцессор (!include, !define, etc.)
│   ├── plantuml-model/          # Геометрические примитивы (Point, Rect, Size)
│   ├── plantuml-layout/         # Layout engines
│   ├── plantuml-renderer/       # SVG генерация
│   ├── plantuml-themes/         # Темы и skinparam
│   ├── plantuml-stdlib/         # Стандартная библиотека (иконки)
│   └── plantuml-wasm/           # WASM биндинги
├── tests/
│   ├── visual/                  # Визуальные регрессионные тесты
│   ├── compatibility/           # Тесты совместимости с PlantUML
│   └── fixtures/                # Тестовые файлы
├── examples/
├── docs/
└── benches/
```

---

## 2. АРХИТЕКТУРА

### 2.1 Высокоуровневый Pipeline

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           plantuml-rs PIPELINE                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────┐    ┌────────────────┐    ┌─────────────┐    ┌──────────┐ │
│  │   Source     │───▶│  Preprocessor  │───▶│   Lexer     │───▶│  Parser  │ │
│  │   Text       │    │  (!include,    │    │   (logos)   │    │  (pest/  │ │
│  │              │    │   !define)     │    │             │    │  lalrpop)│ │
│  └──────────────┘    └────────────────┘    └─────────────┘    └────┬─────┘ │
│                                                                     │       │
│                                                                     ▼       │
│  ┌──────────────┐    ┌────────────────┐    ┌─────────────┐    ┌──────────┐ │
│  │    SVG       │◀───│   Renderer     │◀───│   Layout    │◀───│   AST    │ │
│  │   Output     │    │   (svg crate)  │    │   Engine    │    │  ──────▶ │ │
│  │              │    │                │    │             │    │   Model  │ │
│  └──────────────┘    └────────────────┘    └─────────────┘    └──────────┘ │
│         │                                                                   │
│         ▼                                                                   │
│  ┌──────────────┐                                                           │
│  │    PNG       │  (optional: resvg + tiny-skia)                           │
│  │   Output     │                                                           │
│  └──────────────┘                                                           │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Детальная архитектура компонентов

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              PARSING LAYER                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         plantuml-preprocessor                        │   │
│  ├─────────────────────────────────────────────────────────────────────┤   │
│  │  • !include <file> / !include_once                                  │   │
│  │  • !define / !undef / !ifdef / !ifndef / !else / !endif             │   │
│  │  • !function / !procedure / !return / !endfunction                  │   │
│  │  • !$variable = value                                                │   │
│  │  • %date() / %version() / %filename() и 20 builtin функций          │   │
│  │  • !theme <name>                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                     │                                       │
│                                     ▼                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                           plantuml-parser                            │   │
│  ├─────────────────────────────────────────────────────────────────────┤   │
│  │                                                                      │   │
│  │  ┌─────────────┐     ┌─────────────────────────────────────────┐   │   │
│  │  │   Lexer     │────▶│              Grammar (pest)              │   │   │
│  │  │   (logos)   │     ├─────────────────────────────────────────┤   │   │
│  │  │             │     │  • sequence.pest                         │   │   │
│  │  │  Tokens:    │     │  • class.pest                            │   │   │
│  │  │  - Keywords │     │  • activity.pest                         │   │   │
│  │  │  - Arrows   │     │  • state.pest                            │   │   │
│  │  │  - Idents   │     │  • component.pest                        │   │   │
│  │  │  - Strings  │     │  • usecase.pest                          │   │   │
│  │  │  - etc.     │     │  • ... (20+ грамматик)                   │   │   │
│  │  └─────────────┘     └─────────────────────────────────────────┘   │   │
│  │                                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                     │                                       │
│                                     ▼                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                             plantuml-ast                             │   │
│  ├─────────────────────────────────────────────────────────────────────┤   │
│  │  pub enum Diagram {                                                  │   │
│  │      Sequence(SequenceDiagram),                                      │   │
│  │      Class(ClassDiagram),                                            │   │
│  │      Activity(ActivityDiagram),                                      │   │
│  │      State(StateDiagram),                                            │   │
│  │      Component(ComponentDiagram),                                    │   │
│  │      UseCase(UseCaseDiagram),                                        │   │
│  │      // ... 20+ вариантов                                            │   │
│  │  }                                                                   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                               LAYOUT LAYER                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                           plantuml-layout                            │   │
│  ├─────────────────────────────────────────────────────────────────────┤   │
│  │                                                                      │   │
│  │  trait LayoutEngine {                                                │   │
│  │      fn layout(&self, model: &DiagramModel) -> LayoutResult;         │   │
│  │  }                                                                   │   │
│  │                                                                      │   │
│  │  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐        │   │
│  │  │ SequenceLayout  │ │HierarchicalLayout│ │  FlowchartLayout│        │   │
│  │  ├─────────────────┤ ├─────────────────┤ ├─────────────────┤        │   │
│  │  │ • Participants  │ │ • Sugiyama algo │ │ • Flowchart     │        │   │
│  │  │ • Lifelines     │ │ • Layer assign  │ │ • Swim lanes    │        │   │
│  │  │ • Messages      │ │ • Crossing min  │ │ • Fork/Join     │        │   │
│  │  │ • Fragments     │ │ • Coordinate    │ │ • Conditions    │        │   │
│  │  │ • Activations   │ │ • Edge routing  │ │                 │        │   │
│  │  └─────────────────┘ └─────────────────┘ └─────────────────┘        │   │
│  │                                                                      │   │
│  │  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐        │   │
│  │  │   TreeLayout    │ │   GridLayout    │ │ OrthogonalRouter│        │   │
│  │  ├─────────────────┤ ├─────────────────┤ ├─────────────────┤        │   │
│  │  │ • MindMap       │ │ • Salt/Wireframe│ │ • Edge routing  │        │   │
│  │  │ • WBS           │ │ • Tables        │ │ • Avoid overlaps│        │   │
│  │  │ • OBS           │ │ • Grids         │ │ • Minimize bends│        │   │
│  │  └─────────────────┘ └─────────────────┘ └─────────────────┘        │   │
│  │                                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                             RENDERING LAYER                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                          plantuml-renderer                           │   │
│  ├─────────────────────────────────────────────────────────────────────┤   │
│  │                                                                      │   │
│  │  trait Renderer {                                                    │   │
│  │      fn render(&self, layout: &LayoutResult) -> RenderOutput;        │   │
│  │  }                                                                   │   │
│  │                                                                      │   │
│  │  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐  │   │
│  │  │   SvgRenderer   │    │   PngRenderer   │    │  AsciiRenderer  │  │   │
│  │  ├─────────────────┤    ├─────────────────┤    ├─────────────────┤  │   │
│  │  │ • svg crate     │    │ • resvg         │    │ • Text-based    │  │   │
│  │  │ • Shapes        │    │ • tiny-skia     │    │ • ASCII art     │  │   │
│  │  │ • Text          │    │ • fontdb        │    │                 │  │   │
│  │  │ • Gradients     │    │ • ab_glyph      │    │                 │  │   │
│  │  │ • Markers       │    │                 │    │                 │  │   │
│  │  └─────────────────┘    └─────────────────┘    └─────────────────┘  │   │
│  │                                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. ТЕХНИЧЕСКИЙ СТЕК

### 3.1 Зависимости (Pure Rust, WASM-совместимые)

| Категория | Crate | Версия | Назначение | WASM |
|-----------|-------|--------|------------|------|
| **Лексер** | `logos` | 0.14 | Быстрый лексер через proc-macro | ✅ |
| **Парсер** | `pest` | 2.7 | PEG парсер для грамматик | ✅ |
| **Парсер (альт.)** | `lalrpop` | 0.20 | LR(1) для сложных грамматик | ✅ |
| **Графы** | `petgraph` | 0.6 | Структуры графов, алгоритмы | ✅ |
| **SVG генерация** | `svg` | 0.17 | Построение SVG документов | ✅ |
| **SVG→PNG** | `resvg` | 0.42 | Растеризация SVG | ✅ |
| **2D рендеринг** | `tiny-skia` | 0.11 | CPU растеризатор | ✅ |
| **Шрифты** | `fontdb` | 0.21 | База шрифтов | ✅* |
| **Шрифты (рендер)** | `ab_glyph` | 0.2 | Растеризация глифов | ✅ |
| **Ошибки** | `thiserror` | 1.0 | Типизированные ошибки | ✅ |
| **WASM** | `wasm-bindgen` | 0.2 | JS биндинги | ✅ |
| **Сериализация** | `serde` | 1.0 | JSON/YAML для моделей | ✅ |

\* С отключённым `system_fonts` feature

### 3.2 Минимальные требования

```toml
# Cargo.toml (workspace)
[workspace]
resolver = "2"
members = [
    "crates/*",
]

[workspace.package]
edition = "2021"
rust-version = "1.75"
license = "MIT OR Apache-2.0"

[workspace.dependencies]
# Parsing
logos = "0.14"
pest = "2.7"
pest_derive = "2.7"

# Data structures
petgraph = { version = "0.6", default-features = false }
indexmap = "2.0"
smallvec = "1.13"

# Rendering
svg = "0.17"
resvg = { version = "0.42", default-features = false }
tiny-skia = "0.11"
fontdb = { version = "0.21", default-features = false }
ab_glyph = "0.2"

# Utils
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }

# WASM
wasm-bindgen = "0.2"
```

---

## 4. ФАЗЫ РАЗРАБОТКИ

### 4.1 Обзор фаз

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            ROADMAP ПРОЕКТА                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ФАЗА 0: Инфраструктура          ████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░       │
│  [2-3 недели]                                                                │
│                                                                              │
│  ФАЗА 1: Sequence + Class        ░░░░░░░░████████████████░░░░░░░░░░░░░       │
│  [2-3 месяца]                                                                │
│                                                                              │
│  ФАЗА 2: Activity + State        ░░░░░░░░░░░░░░░░░░░░░░░░████████████░       │
│  [2-3 месяца]                    + Component                                 │
│                                                                              │
│  ФАЗА 3: Остальные UML           ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░████   │
│  [2-3 месяца]                                                                │
│                                                                              │
│  ФАЗА 4: Non-UML диаграммы       ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░██ │
│  [3-4 месяца]                                                                │
│                                                                              │
│  ФАЗА 5: Полировка + WASM        ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░█│
│  [1-2 месяца]                                                                │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 4.2 ФАЗА 0: Инфраструктура (2-3 недели)

#### Цели
- Настроить workspace и CI/CD
- Создать базовые типы и трейты
- Настроить тестирование

#### Задачи

| ID | Задача | Приоритет | Оценка |
|----|--------|-----------|--------|
| 0.1 | Инициализировать Cargo workspace | High | 1 день |
| 0.2 | Настроить CI (GitHub Actions) | High | 1 день |
| 0.3 | Создать crate `plantuml-ast` с базовыми типами | High | 2 дня |
| 0.4 | Создать трейты `LayoutEngine`, `Renderer` | High | 1 день |
| 0.5 | Настроить визуальное тестирование (insta + image diff) | Medium | 2 дня |
| 0.6 | Создать примеры PlantUML файлов для тестов | Medium | 1 день |
| 0.7 | Документация архитектуры (ARCHITECTURE.md) | Medium | 1 день |

#### Deliverables
- [ ] Рабочий workspace с CI
- [ ] Базовые типы AST
- [ ] Инфраструктура тестирования
- [ ] Шаблон для добавления новых типов диаграмм

---

### 4.3 ФАЗА 1: Sequence + Class Diagrams (2-3 месяца)

#### 1A: Препроцессор (3-4 недели)

| ID | Задача | Приоритет | Оценка |
|----|--------|-----------|--------|
| 1.1 | Парсер директив `!include`, `!include_once` | High | 3 дня |
| 1.2 | Переменные `!$var = value` и подстановка | High | 2 дня |
| 1.3 | Условия `!if`, `!ifdef`, `!else`, `!endif` | High | 2 дня |
| 1.4 | Макросы `!define`, `!definelong` | High | 3 дня |
| 1.5 | Функции `!function`, `!procedure` | Medium | 4 дня |
| 1.6 | Builtin функции (`%date`, `%version`, 50+) | Medium | 5 дней |
| 1.7 | Директива `!theme` | Low | 2 дня |

#### 1B: Sequence Diagram Parser (2-3 недели)

| ID | Задача | Приоритет | Оценка |
|----|--------|-----------|--------|
| 1.8 | Грамматика pest для sequence diagrams | High | 3 дня |
| 1.9 | Парсинг участников (actor, participant, boundary, etc.) | High | 2 дня |
| 1.10 | Парсинг сообщений (->>, -->, etc.) | High | 2 дня |
| 1.11 | Парсинг фрагментов (alt, opt, loop, par, etc.) | High | 3 дня |
| 1.12 | Парсинг активаций (activate/deactivate) | Medium | 2 дня |
| 1.13 | Парсинг заметок (note left/right/over) | Medium | 1 день |
| 1.14 | Парсинг разделителей (== title ==) | Low | 1 день |

#### 1C: Sequence Diagram Layout (2-3 недели)

| ID | Задача | Приоритет | Оценка |
|----|--------|-----------|--------|
| 1.15 | Расположение участников (горизонтальное) | High | 3 дня |
| 1.16 | Расчёт высоты lifelines | High | 2 дня |
| 1.17 | Позиционирование сообщений | High | 3 дня |
| 1.18 | Layout фрагментов (вложенные alt/opt) | High | 4 дня |
| 1.19 | Self-messages | Medium | 2 дня |
| 1.20 | Активации (стек активаций) | Medium | 2 дня |

#### 1D: Class Diagram Parser (2 недели)

| ID | Задача | Приоритет | Оценка |
|----|--------|-----------|--------|
| 1.21 | Грамматика pest для class diagrams | High | 2 дня |
| 1.22 | Парсинг классов, интерфейсов, абстрактных | High | 2 дня |
| 1.23 | Парсинг полей и методов | High | 2 дня |
| 1.24 | Парсинг отношений (--|>, ..|>, --o, etc.) | High | 2 дня |
| 1.25 | Парсинг пакетов и namespaces | Medium | 2 дня |
| 1.26 | Парсинг стереотипов <<stereotype>> | Low | 1 день |

#### 1E: Class Diagram Layout — Sugiyama Algorithm (3-4 недели)

| ID | Задача | Приоритет | Оценка |
|----|--------|-----------|--------|
| 1.27 | Cycle removal (обратные рёбра) | High | 2 дня |
| 1.28 | Layer assignment (longest path) | High | 3 дня |
| 1.29 | Crossing minimization (barycenter) | High | 4 дня |
| 1.30 | Coordinate assignment | High | 3 дня |
| 1.31 | Orthogonal edge routing | High | 5 дней |
| 1.32 | Self-loops и multiple edges | Medium | 2 дня |

#### 1F: SVG Renderer (2 недели)

| ID | Задача | Приоритет | Оценка |
|----|--------|-----------|--------|
| 1.33 | Базовый SVG рендерер (rect, line, text) | High | 3 дня |
| 1.34 | Стрелки и маркеры | High | 2 дня |
| 1.35 | Шрифты и измерение текста | High | 3 дня |
| 1.36 | Skinparam (цвета, шрифты, отступы) | Medium | 3 дня |
| 1.37 | Тени и градиенты | Low | 2 дня |

#### Deliverables Фазы 1
- [ ] Полностью рабочий препроцессор
- [ ] Sequence diagrams: парсинг → layout → SVG
- [ ] Class diagrams: парсинг → layout → SVG
- [ ] 90%+ совместимость с PlantUML для этих типов

---

### 4.4 ФАЗА 2: Activity + State + Component (2-3 месяца)

#### 2A: Activity Diagram (4-5 недель)

| ID | Задача | Оценка |
|----|--------|--------|
| 2.1 | Парсер activity diagram (new syntax) | 1 неделя |
| 2.2 | Парсер activity diagram (legacy syntax) | 1 неделя |
| 2.3 | Flowchart layout engine | 2 недели |
| 2.4 | Swim lanes | 3 дня |
| 2.5 | Fork/Join nodes | 3 дня |

#### 2B: State Diagram (3-4 недели)

| ID | Задача | Оценка |
|----|--------|--------|
| 2.6 | Парсер state diagram | 1 неделя |
| 2.7 | Nested states layout | 1 неделя |
| 2.8 | Transitions routing | 1 неделя |
| 2.9 | Concurrent states | 3 дня |

#### 2C: Component/Deployment Diagrams (2-3 недели)

| ID | Задача | Оценка |
|----|--------|--------|
| 2.10 | Парсер component diagram | 4 дня |
| 2.11 | Парсер deployment diagram | 4 дня |
| 2.12 | Hierarchical nesting layout | 1 неделя |
| 2.13 | Ports и интерфейсы | 4 дня |

#### Deliverables Фазы 2
- [ ] Activity diagrams (оба синтаксиса)
- [ ] State diagrams с вложенными состояниями
- [ ] Component/Deployment diagrams
- [ ] Общий layout engine для иерархических структур

---

### 4.5 ФАЗА 3: Остальные UML (2-3 месяца)

| Тип диаграммы | Оценка | Сложность |
|---------------|--------|-----------|
| Use Case Diagram | 2 недели | Низкая |
| Object Diagram | 1 неделя | Низкая (reuse Class) |
| Timing Diagram | 3 недели | Средняя |
| Package Diagram | 1 неделя | Низкая |

#### Deliverables Фазы 3
- [ ] Все 9 типов UML диаграмм
- [ ] 95%+ совместимость с PlantUML

---

### 4.6 ФАЗА 4: Non-UML Diagrams (3-4 месяца)

| Тип диаграммы | Оценка | Примечания |
|---------------|--------|------------|
| **Gantt Chart** | 3 недели | Специфичный timeline layout |
| **MindMap** | 2 недели | Tree layout |
| **WBS** | 1 неделя | Reuse MindMap |
| **JSON/YAML** | 2 недели | Tree visualization |
| **Network (nwdiag)** | 3 недели | Grid-based layout |
| **Salt (Wireframe)** | 3 недели | Grid layout |
| **ER Diagram** | 2 недели | Reuse Class layout |
| **Archimate** | 2 недели | Reuse Component |

#### Deliverables Фазы 4
- [ ] Все non-UML типы диаграмм
- [ ] Стандартная библиотека (AWS, Azure, etc.)

---

### 4.7 ФАЗА 5: Полировка + WASM (1-2 месяца)

| ID | Задача | Оценка |
|----|--------|--------|
| 5.1 | WASM биндинги (wasm-bindgen) | 1 неделя |
| 5.2 | JavaScript API wrapper | 1 неделя |
| 5.3 | NPM package | 3 дня |
| 5.4 | Демо-сайт | 1 неделя |
| 5.5 | Оптимизация производительности | 2 недели |
| 5.6 | Документация API | 1 неделя |
| 5.7 | README, примеры, tutorials | 1 неделя |

#### Deliverables Фазы 5
- [ ] Рабочий WASM модуль
- [ ] NPM package `@plantuml-rs/core`
- [ ] Документация
- [ ] Демо-сайт

---

## 5. ПРИМЕРЫ РЕАЛИЗАЦИИ

### 5.1 Парсинг: Гибридный подход (Logos + Pest)

```rust
// plantuml-parser/src/lexer.rs
use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    #[token("@startuml")]
    StartUml,
    #[token("@enduml")]
    EndUml,
    #[token("participant")]
    Participant,
    #[token("actor")]
    Actor,
    
    // Arrows (sequence)
    #[regex(r"-+>")]
    ArrowRight,
    #[regex(r"<-+")]
    ArrowLeft,
    #[regex(r"\.+>")]
    DottedArrowRight,
    
    // Arrows (class)
    #[token("--|>")]
    Inheritance,
    #[token("..|>")]
    Implementation,
    #[token("--o")]
    Aggregation,
    #[token("--*")]
    Composition,
    
    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,
    
    // Strings
    #[regex(r#""[^"]*""#)]
    String,
}
```

### 5.2 Layout: Sugiyama Algorithm

```rust
// plantuml-layout/src/hierarchical/sugiyama.rs

pub struct SugiyamaLayout {
    config: LayoutConfig,
}

impl SugiyamaLayout {
    /// Основной алгоритм Sugiyama (Layered Graph Drawing)
    /// 
    /// Шаги:
    /// 1. Cycle Removal - удаление циклов (обратные рёбра)
    /// 2. Layer Assignment - распределение по слоям
    /// 3. Crossing Minimization - минимизация пересечений
    /// 4. Coordinate Assignment - назначение координат
    /// 5. Edge Routing - маршрутизация рёбер
    pub fn layout(&self, graph: &DiGraph<Node, Edge>) -> LayoutResult {
        let (acyclic_graph, reversed_edges) = self.remove_cycles(graph);
        let layers = self.assign_layers(&acyclic_graph);
        let (layered_graph, dummy_nodes) = self.add_dummy_nodes(&acyclic_graph, &layers);
        let ordered_layers = self.minimize_crossings(&layered_graph, &layers);
        let coordinates = self.assign_coordinates(&ordered_layers);
        let edges = self.route_edges(&coordinates, &reversed_edges);
        
        LayoutResult { nodes: coordinates, edges }
    }
}
```

### 5.3 SVG Renderer

```rust
// plantuml-renderer/src/svg/mod.rs

use svg::node::element::{Group, Line, Rectangle, Text, Path, Definitions, Marker};
use svg::Document;

pub struct SvgRenderer {
    theme: Theme,
    fonts: FontRegistry,
}

impl SvgRenderer {
    pub fn render(&self, layout: &LayoutResult) -> String {
        let mut document = Document::new()
            .set("viewBox", (0, 0, layout.width, layout.height))
            .set("xmlns", "http://www.w3.org/2000/svg");
        
        let defs = self.create_definitions();
        document = document.add(defs);
        
        for element in &layout.elements {
            let svg_element = match element {
                Element::Rectangle(r) => self.render_rectangle(r),
                Element::Line(l) => self.render_line(l),
                Element::Text(t) => self.render_text(t),
                Element::Path(p) => self.render_path(p),
                Element::Group(g) => self.render_group(g),
            };
            document = document.add(svg_element);
        }
        
        document.to_string()
    }
}
```

---

## 6. ТЕСТИРОВАНИЕ

### 6.1 Стратегия тестирования

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           ПИРАМИДА ТЕСТОВ                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│                              ┌─────────┐                                     │
│                             /  E2E     \                                     │
│                            /  (Visual)  \                                    │
│                           / Compatibility \                                  │
│                          /   with PlantUML \                                 │
│                         ───────────────────                                  │
│                        /   Integration      \                                │
│                       /   (Parser → Layout   \                               │
│                      /     → Render)          \                              │
│                     ─────────────────────────────                            │
│                    /        Unit Tests            \                          │
│                   /    (Lexer, Parser, Layout,     \                         │
│                  /         Renderer separately)     \                        │
│                 ─────────────────────────────────────                        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 6.2 Визуальное регрессионное тестирование

```rust
use insta::assert_snapshot;
use plantuml_core::render;

#[test]
fn test_simple_sequence() {
    let source = r#"
@startuml
Alice -> Bob: Hello
Bob --> Alice: Hi
@enduml
"#;
    
    let svg = render(source).unwrap();
    assert_snapshot!("simple_sequence", svg);
}
```

---

## 7. РИСКИ И МИТИГАЦИИ

| Риск | Вероятность | Влияние | Митигация |
|------|-------------|---------|-----------|
| **Layout алгоритмы сложнее ожидаемого** | Высокая | Высокое | Reverse-engineering PlantUML Java кода; использование академических статей |
| **100% совместимость недостижима** | Средняя | Среднее | Определить "95% compatibility" как достаточный |
| **Производительность WASM** | Низкая | Среднее | Профилирование; оптимизация критических путей |
| **Обновления PlantUML** | Средняя | Низкое | Следить за releases; модульная архитектура |
| **Шрифты в WASM** | Средняя | Среднее | Embedded шрифты; subset fonts |

---

## 8. ОЦЕНКА ТРУДОЁМКОСТИ

### С AI-ассистированной разработкой

| Фаза | Традиционная оценка | С AI-ассистентами |
|------|---------------------|-------------------|
| Фаза 0 | 2-3 недели | 1-2 недели |
| Фаза 1 | 2-3 месяца | 1-2 месяца |
| Фаза 2 | 2-3 месяца | 1-2 месяца |
| Фаза 3 | 2-3 месяца | 1-1.5 месяца |
| Фаза 4 | 3-4 месяца | 2-3 месяца |
| Фаза 5 | 1-2 месяца | 2-4 недели |
| **ИТОГО** | **12-18 месяцев** | **6-10 месяцев** |

---

## 9. ССЫЛКИ

- [PlantUML Official](https://plantuml.com/)
- [PlantUML GitHub](https://github.com/plantuml/plantuml)
- [Sugiyama Algorithm](https://en.wikipedia.org/wiki/Layered_graph_drawing)
- [pest Parser](https://pest.rs/)
- [logos Lexer](https://logos.maciej.codes/)
- [resvg](https://github.com/RazrFalcon/resvg)
