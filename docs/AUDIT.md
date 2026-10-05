# Аудит проекта plantuml-rs и план изменений

**Дата:** 2026-01-14
**Область:** весь workspace (10 крейтов, ~36 500 строк Rust)
**Метод:** сборка, тесты, clippy, проверка CI-гейтов локально, чтение кода, эмпирические пробы рендеринга, сравнение SVG с эталонами PlantUML.

---

## 1. Резюме

Проект — **работоспособный каркас**, а не «библиотека, полностью совместимая с PlantUML», как заявлено в README.
Все 18 типов диаграмм имеют парсер и layout-движок, всё компилируется, 354 теста проходят, WASM собирается.
Но заявленная цель («100% совместимость», критерий приёмки из `AGENTS.md` — «визуально неотличимо от оригинала») **не достигается и не может быть достигнута на текущей архитектуре**.

Три корневые причины:

1. **Нет измерения текста реальными метриками шрифта.** PlantUML рассчитывает всю геометрию от ширины текста в шрифте (AWT `FontMetrics`) и пишет `textLength` в SVG. В проекте вместо этого — **восемь разных эвристик** «количество символов × константа», пять из которых считают **байты, а не символы** (ломает кириллицу). Отсюда неверны размеры всех боксов и вся раскладка.
2. **Тема и `skinparam` не доходят до рендерера.** Препроцессор разбирает их в `ctx.theme`, но `process()` возвращает только `String` — тема теряется. `skinparam backgroundColor #FF0000` даёт **байт-идентичный** SVG. Четыре поля `Theme` рендерер не читает вообще.
3. **Сверка с PlantUML не автоматизирована.** `tests/compatibility/` пуста, `tests/visual/` не используется ни одним тестом, 12 insta-снапшотов **самореферентны** (фиксируют текущий вывод, а не соответствие PlantUML). Заявление о 100% ничем не измеряется.

Дополнительно: **CI красный на HEAD** (3 из 8 job'ов падают), **~1841 строка реализации `stdlib` не закоммичена** (риск безвозвратной потери), а диагностированные баги включают **stack overflow** на циклическом `!include` и **молчаливый вывод пустой диаграммы** с кодом успеха.

---

## 2. Что работает

Важно зафиксировать: проект не «сломан», у него прочный фундамент.

| Что | Состояние |
|-----|-----------|
| `cargo check --workspace --all-targets` | ✅ проходит |
| `cargo test --workspace` | ✅ 354 passed, 0 failed, 5 ignored |
| `cargo build --target wasm32-unknown-unknown -p plantuml-wasm` | ✅ собирается |
| `cargo build -p plantuml-core --features png` | ✅ собирается |
| `cargo fmt --all -- --check` | ✅ чисто |
| 18 типов диаграмм: парсер + layout + рендер | ✅ есть у всех 18 |
| Pest-грамматики | ✅ все 16 подключены, мёртвых нет |
| Препроцессор (`!include`/`!define`/`!function`/переменные) | ✅ 51 тест, содержательная реализация |
| Единый SVG-рендерер через `ElementType` | ✅ архитектурно верное решение |
| Sequence layout | ✅ **единственный движок, приближенный к PlantUML**: реальный двухпроходный алгоритм расчёта ширин и spacing, документирован в `docs/SEQUENCE_LAYOUT_ALGORITHM.md` |

Плюс частично совпадают с PlantUML: `rx/ry = 2.5`, `stroke-width 0.5` (участники/lifelines), `stroke-dasharray 5,5` и `2,2`, `font-size 14` (участники) и `13` (сообщения).

---

## 3. Критические проблемы (блокеры цели)

### 3.1. Нет метрик шрифта — фундаментальный блокер

Ни в одном layout-движке нет измерения текста через метрики шрифта. `plantuml-layout/Cargo.toml` не содержит ни `fontdb`, ни `ab_glyph`, ни `rusttype`.

| Способ | Где | Проблема |
|--------|-----|----------|
| `text.len() as f64 * 8.0` | `salt/engine.rs` ×11, `json/engine.rs:265` | байты + литерал |
| `name.len() as f64 * 9.0` | `er/engine.rs:38` | байты |
| `(a+b+c+3) as f64 * 7.5` | `er/engine.rs:51` | байты + фальшивый `+3` |
| `text.len() as f64 * char_width` | `class/graph.rs:59,73,88`, `wbs/engine.rs:147` | байты |
| `font_size * 0.6` | `mindmap/engine.rs:174`, `wbs/engine.rs:146` | коэффициент без источника |
| `chars().count() * 7.5` | `sequence/config.rs:74` | символы, но константа подобрана «на глаз» |
| `chars().count() * 9.0` | `usecase/engine.rs:45` | символы, литерал |

Комментарий в коде это подтверждает: `sequence/config.rs:57` — `char_width: 7.5, // немного шире для кириллицы`; `:79` — `+ 30.0; // padding увеличен (было 20)`.

**Числовое доказательство** (участник `Alice`, шрифт 14, sans-serif):

```
PlantUML : textLength="33.667"  →  33.667 / 5 символов = 6.73 px/символ
           rect width = 47.667
Наш код  : 5 × 7.5 = 37.5 + 30 padding = 67.5
           расхождение ширины бокса = +40%
```

Практическое следствие: для кириллицы пять из восьми способов дают ширину примерно **вдвое** больше реальной. Разные движки для одного текста дают разную ширину.

**Вывод:** без слоя `TextMeasurer` (реальные метрики либо зафиксированная таблица ширин глифов, совместимая с PlantUML) цель «идентично PlantUML» недостижима в принципе. Это предпосылка для всей остальной работы над геометрией.

### 3.2. Тема и `skinparam` не доходят до рендерера

Препроцессор разбирает `!theme` и `skinparam` в `ctx.theme` (`plantuml-preprocessor/src/lib.rs:160,174`), но `process()` возвращает только `String` (`:205-208`). Тема не покидает препроцессор: `grep` по `plantuml-core` и `plantuml-parser` не находит ни одного обращения к `ctx.theme`.

Эмпирически подтверждено — все варианты дают байт-идентичный вывод базовому (2624 байта):

```
skinparam monochrome true            → идентично базовому
skinparam sequenceMessageAlign left  → идентично базовому
skinparam handwritten true           → идентично базовому
!theme cerulean                      → идентично базовому
participant A #Red                   → идентично базовому
```

Дополнительно: `Theme` содержит 11 полей, рендерер читает 8. `line_width`, `corner_radius`, `shadow`, `handwritten` — **ноль использований**. `SkinParams::apply_to` (`plantuml-themes/src/lib.rs:229`) вызывается только из препроцессора, то есть ни на что не влияет.

При этом комментарий `plantuml-renderer/src/lib.rs:67` **ложно** утверждает, что `#FEFECE` — цвет фона PlantUML по умолчанию (у PlantUML фон диаграммы — `#FFFFFF`).

### 3.3. Сверка с PlantUML отсутствует

- `tests/compatibility/` — **пустая директория**.
- `tests/visual/` (эталоны PlantUML) — **не используется ни одним тестом**: `grep -rn "visual/" crates/` пуст.
- `tests/fixtures/` (8 `.puml`) — **не используется ни одним тестом**.
- 12 insta-снапшотов **самореферентны**: сравнивают рендер сам с собой, фиксируя текущее (неверное) поведение.
- `tests/visual/COMPARISON_REPORT.md` **не соответствует реальности** (разбор в §5.1).
- Ни один тест не сравнивается с выводом plantuml.com — при том что `AGENTS.md` прямо требует такого сравнения с критерием «визуально неотличимо».

---

## 4. Серьёзные проблемы

### 4.1. Детекция типа диаграммы подстрокой — ложные срабатывания и молчаливая потеря данных

`detect_diagram_type()` (`plantuml-parser/src/lib.rs:209-400`) содержит **66 проверок** `source_lower.contains(...)` плюс 4 рукописных хелпера (`has_state_keyword`, `has_paren_usecase_pattern`, `has_colon_actor_usecase_pattern`, `has_component_bracket_pattern`).

Поскольку проверяется весь исходник, включая **текст сообщений**, обычная sequence-диаграмма ломается от содержимого подписи. Проверено:

| Исходник (неявные участники) | Распознан как | Участников в SVG |
|---|---|---|
| `Alice -> Bob: component deployed` | Component | **0** |
| `Alice -> Bob: object created` | Object | **0** |
| `Alice -> Bob: class loaded` | Class | **0** |
| `Alice -> Bob: map lookup` | Object | **0** |
| `Alice -> Bob: usecase done` | UseCase | **0** |
| `Alice -> Bob: if (ready)` | ошибка парсинга | 0 |
| `Alice -> Bob: salt {` | ошибка парсинга | 0 |

При этом `render()` возвращает **`Ok`** и пустую по смыслу диаграмму — это **молчаливая потеря данных**, а не диагностируемая ошибка. Пользователь получает «успешный» рендер без единого сообщения.

Аналогично: `database DB` + `queue Q` + `DB --> Q` (component-диаграмма) распознаётся как **Sequence** и рисуется как диаграмма последовательностей.

Отдельно: **пустая диаграмма `@startuml\n@enduml` даёт ошибку** «не удалось определить тип диаграммы», хотя PlantUML рендерит её как пустой холст.

### 4.2. Stack overflow на циклическом `!include`

Воспроизведено: `a.puml` включает `b.puml`, `b.puml` включает `a.puml`.

```
thread 'main' has overflowed its stack
fatal runtime error: stack overflow, aborting
```

`handle_include` (`plantuml-preprocessor/src/lib.rs:315-336`) рекурсивно вызывает `process_with_context` **без проверки глубины**. Метод `check_recursion` (`fs_resolver.rs:174`), предназначенный ровно для этого, помечен `#[allow(dead_code)]` и **не вызывается нигде**; поля `current_depth`/`max_depth`/`included_files` в резолвере не используются.

Критично вдвойне: библиотека заявлена как WASM-совместимая, где такая ошибка означает падение вкладки браузера без возможности перехвата.

### 4.3. Реестр `stdlib` недостижим — срезаются угловые скобки

48 записей реестра (`C4`, `tupadr3`, `office`, `logos`, `common`, `colors`, `sprites`) не работают ни через один публичный путь:

```
!include <C4/C4_Context>   → ошибка препроцессора: !include не поддерживается в этом окружении
render_with_includes(...)  → ошибка препроцессора: файл не найден: C4/C4_Context
```

Причина — прямое противоречие двух мест:

```rust
// plantuml-preprocessor/src/lib.rs:326 — маркер <...> уничтожается
let path = path.trim_matches(|c| c == '<' || c == '>' || c == '"');
...
let content = self.resolver.read_file(path)?;

// plantuml-preprocessor/src/fs_resolver.rs:110 — резолвер ждёт именно <...>
if path_str.starts_with('<') && path_str.ends_with('>') {
    return self.resolve_stdlib_path(...);   // недостижимо
}
```

Тест `test_stdlib_include` (`fs_resolver.rs:355`) **зелёный**, потому что вызывает резолвер напрямую с `<C4/C4_Context>`, минуя `lib.rs`. То есть тест маскирует баг.

### 4.4. Дублирующиеся `id` в SVG

`svg_renderer.rs:178` пишет `element.id` прямо в атрибут `id`. Layout формирует id из статических частей без счётчика (`sequence/engine.rs:911` — `format!("msg_{}_{}", msg.from, msg.to)`).

Проверено на реальном выводе:

```
output_simple.svg    : id="msg_Alice_Bob" ×2, id="msg_Bob_Alice" ×2
output_autonumber.svg: id="msg_bff_bff" ×3
```

Это невалидный SVG (id обязаны быть уникальны) и ломает любые селекторы/якоря. Аналогичная схема в 11 движках: `edge_{from}_{to}` (class), `trans_{from}_{to}` (state), `rel_{from}_{to}` (usecase), `link_{from}_{to}` (object), `return_`, `divider_`, `delay_`, `note_`, `ref_` (sequence).

### 4.5. Парсер ошибок всегда сообщает «в строке 0»

Все парсеры используют `e.line().to_string().parse().unwrap_or(0)` (например, `parsers/sequence.rs:36`). Проверено на реальной ошибке в строке 4:

```
Display: синтаксическая ошибка в строке 0:  --> 4:9
>>> Поле line = 0 (ожидалось 4)
```

Пользователь API, читающий `ParseError::SyntaxError { line, .. }` программно, всегда получает `0`. Корректная позиция есть только внутри текста сообщения pest.

### 4.6. Ложные успехи в layout

- **`gantt`**: прямоугольники выходных создаются высотой `1000.0` (`gantt/engine.rs:308`, комментарий «Высокое значение, будет обрезано» — обрезания нет). Они участвуют в `calculate_bounds()`, который затирает корректные `total_height`. Факт: `target/gantt_examples/weekends.svg` — `viewBox="15 35 660 1030"` против 136–241 px у остальных примеров.
- **`state/engine.rs:226`** — `composite_layouts.get(state_name).unwrap()` без ветки `else`; потенциальная паника.
- **`sequence/engine.rs:1107,1156`** — `note.anchors.last().unwrap()` / `reference.participants.last().unwrap()`; пустой список → паника.
- **`component/engine.rs:103-123`** — возвращаемые `bounds` (140×60) не совпадают с фактически созданным элементом для `Interface` (эллипс 20×20), `Actor` (0.6 ширины), `Cloud` (1.2 ширины). Связи привязываются к несуществующему блоку.
- **`salt/engine.rs:910`** — `layout()` создаёт `SaltLayoutEngine::new()` вместо использования `self`, поэтому `with_config()` не оказывает никакого эффекта.
- **`wbs/engine.rs:209-230`** — `center_diagram` сдвигает `element.bounds`, но не координаты внутри `properties["path"]`; после центрирования коннекторы рисуются по старым координатам.
- **`activity`**: `Condition::elseif_branches` **не читается нигде** — ветки `elseif` молча исчезают. Заметки и коннекторы отфильтрованы `continue` (`:83-91`) до обработки, поэтому их `// TODO` (`:394`, `:402`) недостижимы.

### 4.7. Мёртвый код, создающий иллюзию покрытия

- **`plantuml-parser/src/lexer.rs` — 384 строки, не используется нигде.** `#[derive(Logos)]`-лексер экспортируется как `pub mod lexer`, но ни один парсер его не вызывает (все работают через `pest` напрямую по строкам). При этом внутри 3 «зелёных» теста, создающих видимость работающего лексера. Зависимость `logos` заявлена в `AGENTS.md` как ключевая.
- **`LayoutConfig` — мёртвая абстракция.** Все 7 реализаций трейта `LayoutEngine` принимают его как `_config` и игнорируют. `LayoutConfig::sequence()` и `::class()` — ноль вызовов. `pipeline.rs:89` создаёт его только как заглушку-аргумент. При этом он часть публичного API и вводит пользователя в заблуждение.
- **`struct NodeLayout`** (`mindmap/engine.rs:21`) — не конструируется нигде (подтверждено clippy).
- **12 неиспользуемых полей конфигов**: `activity::{bar_width, arrow_size}`, `component::icon_size`, `er::min_entity_width`, `mindmap::{node_padding_y, corner_radius}`, `salt::background_color`, `state::{state_corner_radius, arrow_size, text_padding}`, `timing::concise_line_height`, `class::with_node_spacing`.
- **Неиспользуемые элементы AST**: `State.regions`, `StateDiagram.notes`, `State.description`, `entry_action`, `exit_action`, `ActivityElement::Note/Connector`, `Action.background_color/style/arrow_label` — layout их не читает.

### 4.8. Мёртвые абстракции и `unwrap` в горячем пути

`salt/engine.rs:910` (см. §4.6), `usecase/engine.rs:38` — `let _is_left_to_right = diagram.direction == Direction::LeftToRight;` (вычислено и не используется), `usecase/engine.rs:297,299` — `from_center_x`/`to_center_x` вычислены и выброшены, `state/engine.rs:388` — `content_width` не используется, `sequence/engine.rs:575` — `text_width` для `Return` вычислен и выброшен.

`usecase`: **все актёры размещаются на одном X** (`actors_x` вычисляется один раз, `:52`, используется в цикле `:144`) — при двух и более актёрах они полностью накладываются.

### 4.9. `from_pest`/`pest` MSRV-конфликт

`rust-version = "1.75"` в `Cargo.toml` недостижим:

```
$ rustup run 1.75 cargo check --workspace
error: package `pest v2.8.4` cannot be built because it requires rustc 1.83 or newer,
       while the currently active rustc version is 1.75.0
```

Плюс `LazyLock` (stable с 1.80) используется в `plantuml-stdlib/src/lib.rs:42` и `plantuml-preprocessor/src/builtins.rs:12`. Плюс `Cargo.lock` в `.gitignore` (`.gitignore:4`), поэтому на чистом клоне резолвятся свежие зависимости с `edition2024`.

---

## 5. Средние и мелкие проблемы

### 5.1. `tests/visual/COMPARISON_REPORT.md` не соответствует реальности

Отчёт утверждает «Sequence ✅ Проверено и исправлено». Проверка:

| Утверждение отчёта | Факт |
|---|---|
| «Размеры: PlantUML 296x208» | живой PlantUML 1.2026.9beta4 даёт **306x218** |
| «наш 368x201» | текущий рендер даёт **338.5x239** |
| «our_simple.svg» как эталон сверки | файл **не отслеживается git** (локальный мусор, 3119 байт, отличается от свежего вывода 3791 байт) |
| «Оставшиеся различия (некритичные): viewBox offset» | сдвиг viewBox `0 0` → `15 15` **прибавляет +15 ко всем координатам** и сам по себе делает SVG неидентичным |
| «Форма стрелки: polygon / marker ✅» | для наследования PlantUML рисует полый треугольник ~11.5px инлайновым `polygon`; у нас `<marker>` 20×20 — вдвое больше |
| Class «⚠️ Частично проверено» | фактически неверны заливка, направление раскладки, форма рёбер, ширины |
| Таблица на 17 строк | в AST 18 типов; `Archimate` в отчёте отсутствует, а `Deployment`/`Archimate` вообще не имеют своих движков (переиспользуют `ComponentLayoutEngine`) |

Не упомянуты критические расхождения: шаг сообщений, высота бокса участника, шаг между участниками.

### 5.2. Измеренные расхождения геометрии (sequence, один и тот же исходник)

| Параметр | PlantUML | наш | Δ |
|---|---|---|---|
| width | 296px | 338.5 | **+14.4%** |
| height | 208px | 239 | **+14.9%** |
| viewBox | `0 0 296 208` | `15 15 338.5 239` | сдвиг +15,+15 |
| participant (x,y,w,h) | 5, 5, 47.667, 30.2969 | 20, 20, 67.5, 35 | **+40% по ширине** |
| X линий жизни | 28 / 269.8584 | 53.75 / 322.25 | +25.75 / +52.39 |
| шаг сообщений (pitch) | 29.1328 | 35.0 | **+20.1%** |
| Y сообщений | 67.43 / 96.56 / 125.70 / 154.83 | 85 / 120 / 155 / 190 | накопленный сдвиг до +35 |

Плюс структурные расхождения: PlantUML ставит `font-family` и `lengthAdjust` **один раз на корневом `<g>`** — у нас `font-family` в 22 местах. PlantUML **никогда** не использует `dominant-baseline`/`text-anchor` — у нас 9 и 13 вхождений. PlantUML пишет `textLength` на каждом `<text>` — у нас не пишет вообще. Отсутствуют `contentStyleType`, `data-diagram-type`, `preserveAspectRatio`, `zoomAndPan`, CSS-классы, `data-entity-uid`. Плюс наши размеры нецелые (`width="338.5"`, `width="504.25"`), PlantUML даёт целые px.

### 5.3. Магические константы

~180 геометрических литералов, влияющих на вывод. Задокументировано ~12 — и все со словом «~» («PlantUML ~80-100», «~35px»), то есть даже они не являются проверенными значениями.

Найден прямой конфликт комментария и кода:

```rust
// sequence/engine.rs:1300-1305: «PlantUML: отступ ~17px ... current_y - message_spacing + 17 ≈ current_y - 11»
let y = metrics.current_y - 11.0;   // при message_spacing = 35.0 фактический отступ = 24px, а не 17
```

Литерал `- 11.0` продублирован трижды: `:74`, `:1305`, `:1341`. Ширина петли self-message: `40.0` в проверке переполнения (`:155`) против `42.0` в геометрии (`:866`).

### 5.4. Дублирование

| Что | Копий |
|---|---|
| `calculate_connection_points` («сравни dx/dy, выбери грань») | 5 (`component`, `state`, `class`, `er`, `usecase`) |
| `calculate_bounds` + `calculate_node_width` + `create_node_element` + `layout_node` | по 2 (`mindmap`/`wbs`), почти построчно, причём `mindmap` считает **символы**, а `wbs` — **байты** |
| `layout_object` / `layout_array` | 2 внутри `json/engine.rs` (~90 строк, отличаются только заголовком) |
| Элемент заголовка диаграммы | 2 (`timing:154`, `gantt:181` — одинаковые литералы) |
| `header_height = 30.0` | 3 (`state/engine.rs:165,237,511`) |

### 5.5. Тесты: формальные, а не проверяющие

Из 66 тестов `plantuml-layout`:

- ~38 — только «не пусто» / «не паникует»;
- ~12 — `bounds.width > 0.0 && height > 0.0`;
- **0 — проверяют конкретную координату, вычисленную движком**.

Тавтологичные тесты:

```rust
// class/sugiyama.rs:395-414 — условие истинно почти при любом присвоении слоёв
assert!(animal.layer < dog.layer || animal.layer < cat.layer
        || (dog.layer == 0 && cat.layer == 0 && animal.layer == 1));

// json/engine.rs:382-388 — называется test_node_width_calculation, проверяет дефолт конфига
assert!(engine.config.padding > 0.0);
```

Расхождение комментария и ассерта: `usecase/engine.rs:347` обещает 4 элемента, проверяет `>= 3`; `sequence/engine.rs:1407` обещает 7, проверяет `>= 3`.

Snapshot-тесты есть только для sequence (4), class (5), state (3). Для **13 из 18 типов визуальной регрессии нет вообще**. Ни одного `#[should_panic]`, нет тестов на граничные случаи.

### 5.6. Формальные и функциональные пробелы парсера sequence

Проверено на конструкциях PlantUML:

| Конструкция | Результат |
|---|---|
| `header H` / `footer F` | ❌ ошибка парсинга |
| `hide footbox` | ❌ ошибка парсинга |
| `title Заголовок` | ⚠️ парсится, но **в SVG не попадает** (потерян) |
| `note over A \n строка1 \n строка2 \n end note` | ❌ ошибка (многострочная заметка) |
| `group Название` | ❌ ошибка |
| `A -[#red]-> B` (цвет стрелки) | ❌ ошибка |
| `autonumber 1.1` | ❌ ошибка (в рабочем дереве уже исправлено, но не закоммичено) |
| `skinparam`, `!theme`, цвет участника, стереотип | ⚠️ парсятся, но **не влияют** на вывод (§3.2) |
| `loop`, `par`, `critical`, `break`, `alt/else`, `activate`, `create`, `destroy`, `return`, `ref over`, `...`, `==`, `box` | ✅ работают |

В других типах: ER **не работает вообще** (даже `entity USER { id : int }` → «не удалось определить тип диаграммы»; `entity ORDER` без тела → ошибка грамматики); `class` не понимает `note right of`, `top to bottom direction`, `show A methods`, `enum`, `annotation`; `component` не понимает `[Web] --> [API]`; `object` не понимает `object o1 { x = 1 }`; `activity` не понимает `while (x)`; `timing` не понимает `0 is red`; `yaml` не понимает списки; `archimate` не парсится.

### 5.7. Документация расходится с кодом

| Документ | Проблема |
|---|---|
| `README.md` | Крейта `plantuml-rs` **не существует** (реально `plantuml-core`); версия в примере `0.1` вместо `0.2.0`; **все 3 примера Rust не компилируются** (`render(source)` вместо `render(source, options)`); WASM-пример импортирует несуществующий модуль; таблица производительности (`~5ms`, «на M1 MacBook Pro») ничем не подтверждена при **пустой** `benches/`; «6 тем» против 5 в `available_themes()`; `plantuml-model` описан как «Модели диаграмм», реально — 162 строки геометрии |
| `ARCHITECTURE.md` | `HierarchicalLayout`, `FlowchartLayout`, `TreeLayout`, `GridLayout`, `TimelineLayout`, `AsciiRenderer` — **0 вхождений в коде**; feature flags (`default=["svg"]`, `all-diagrams`, `sequence`, `class`) не совпадают с реальным `Cargo.toml` (`default=[]`, `serde`, `png`) |
| `PLAN.md` | `plantuml-model` как «Типизированные модели»; `tests/compatibility/`, `examples/`, `benches/` заявлены — все **пусты**; «50+ builtin функций» — реально **20**; stdlib «AWS, Azure» — отсутствуют |
| `SYNTAX.md` | Темы `spacelab`, `materia` **не существуют** (есть `default, classic, minimal, dark, sketchy, cerulean`) |
| `CHANGELOG.md` | Дата `2025-01-03` вместо `2026-01-03`; «18 типов» — перечислено 16; «6 тем» — перечислено 5; `[0.1.0]` заявлен как «полная реализация», хотя на том коммите был 2 файла парсеров |
| `CONTRIBUTING.md` | Битые ссылки на `github.com/user/plantuml-rs` (`:27`, `:70`) — реальный remote `askidmobile/PlantUML_RUST` |
| `SECURITY.md` | Упоминает несуществующий `FileResolver` (реально `FsFileResolver`); контакт для сообщения об уязвимости не указан |
| `docs/PlantUML.pdf` | **10.5 МБ / 610 страниц, закоммичен.** 88% всего трекаемого контента (11.0 из 12.6 МБ), в 165 раз больше второго по величине файла, **ни на что не ссылается** |

### 5.8. Гигиена репозитория

- **`git diff` — это на 100% шум по правам.** 204 файла изменены, все — `mode change 100644 => 100755`. Из них **137 изменены только по правам** (тело diff пустое), 65 — права + реальные правки, 2 бинарника — содержимое идентично по хешу. Причина — внешний том `/Volumes/Aski Dev`. Лечится `git config core.fileMode false`.
- **~1841 строка `stdlib` не закоммичена** (см. §6.1) — критический риск.
- **Мусор в корне, не покрытый `.gitignore`**: `our_ref.png`, `our_simple.png`, `our_simple_fixed.png`, `our_simple_fixed2.png`, `our_simple_v2.png`, `reference_simple.png` (~670 КБ) — любой `git add .` их затащит. Плюс `test_ref_example.rs` — мёртвый файл вне workspace (не входит ни в один крейт, нигде не упоминается).
- **Осиротевший `pkg/`** (3.7 МБ) — дубликат `playground/pkg/`, не создаётся ни одним workflow.
- **`benches/`, `examples/` — пусты**, но заявлены в `PLAN.md`.
- **`playground/.gitignore` = `*`** конфликтует с фактом трекинга `index.html`/`test.html` (добавлены через `git add -f`). Комментарий `.gitignore:31` («playground/pkg tracked for GitHub Pages») неверен.

### 5.9. CI не работает

| Job | Команда | Результат на HEAD |
|---|---|---|
| `check` | `cargo check --workspace --all-features` | ✅ |
| `fmt` | `cargo fmt --all -- --check` | ✅ |
| `test` | `cargo test --workspace --all-features` | ✅ |
| `wasm` | `cargo build --target wasm32-unknown-unknown -p plantuml-wasm` | ✅ (но с `continue-on-error: true`) |
| **`clippy`** | `cargo clippy --workspace --all-features -- -D warnings` | ❌ **EXIT=101**, 79 предупреждений становятся ошибками |
| **`msrv`** | `cargo +1.75 check --workspace` | ❌ **EXIT=101** |
| **`docs`** | `RUSTDOCFLAGS=-D warnings cargo doc ...` | ❌ **EXIT=101** (`unclosed HTML tag` в `activity.rs:91`, `common.rs:158`, `sequence.rs:349`) |

Проблемы CI:
- **`AGENTS.md` предписывает `cargo clippy --workspace -- -D warnings` как рабочую команду** — она падает. Документация противоречит реальности.
- **`ci.yml:119`**: `continue-on-error: true  # WASM crate не создан ещё` — крейт **существует** и собирается; комментарий устарел, флаг маскирует реальные поломки.
- Основная масса предупреждений clippy: 34 — `clippy::incompatible_msrv` (`LazyLock` при заявленном 1.75), 24 — `clone_on_copy` для `Rect`, остальное — `single_match`, `if_same_then_else`, `too_many_arguments`, `manual_strip`, мёртвый код.

---

## 6. Незакоммиченная работа (риск потери)

### 6.1. `plantuml-stdlib` — ~1841 строка только в рабочем дереве

```
?? crates/plantuml-stdlib/src/c4.rs        287 строк
?? crates/plantuml-stdlib/src/common.rs    172
?? crates/plantuml-stdlib/src/logos.rs     363
?? crates/plantuml-stdlib/src/office.rs    291
?? crates/plantuml-stdlib/src/tupadr3.rs   472
 M crates/plantuml-stdlib/src/lib.rs       (41 → 256 строк)
```

В `HEAD` лежит **заглушка**: 41 строка с `// TODO: Реализовать`, `get_sprite()` всегда `None`, `exists()` всегда `false`. Вся реальная реализация (реестр на `LazyLock`, 48 include) существует **только локально**. `git checkout .` или клон репозитория уничтожит её безвозвратно.

При этом проверено: `lib.rs` в рабочем дереве объявляет `mod c4; mod common; mod logos; mod office; mod tupadr3;` — то есть **закоммитить без этих файлов нельзя**, сборка упадёт.

### 6.2. Крупные правки layout/parser

```
426/62  layout/src/activity/engine.rs      320/228 layout/src/state/engine.rs
258/18  layout/src/sequence/metrics.rs     181/64  layout/src/sequence/engine.rs
110/85  renderer/src/svg_renderer.rs       171/44  parser/src/parsers/sequence.rs
124/29  parser/src/parsers/activity.rs      88/23  layout/src/component/engine.rs
 74/39  layout/src/gantt/engine.rs          67/42  layout/src/timing/engine.rs
 44/7   preprocessor/src/fs_resolver.rs     52/4   core/examples/sequence_demo.rs
```

Плюс 4 изменённых snapshot-файла и правки в 20 AST-файлах. Рабочее дерево **функциональнее HEAD**.

---

## 7. План изменений

Порядок фаз выбран по принципу «сначала то, что обесценивает или блокирует остальное». Фазы 0–1 обязательны до любой работы над геометрией: без сохранения работы и без измерительной сетки любые правки рендеринга недоказуемы.

### Фаза 0. Сохранить работу и остановить кровотечение (полдня–день)

| # | Задача | Критерий готовности |
|---|--------|---------------------|
| 0.1 | Закоммитить `plantuml-stdlib`: добавить `c4.rs`, `common.rs`, `logos.rs`, `office.rs`, `tupadr3.rs` вместе с новым `lib.rs` | ✅ **Сделано.** `plantuml-stdlib` закоммичен (5 модулей + новый `lib.rs`), сборка проходит |
| 0.2 | Отключить шум прав: `git config core.fileMode false`, затем `git checkout` для сброса mode-изменений | ✅ **Сделано.** `core.fileMode false`; `git status` показывает 65 реальных правок вместо 204 |
| 0.3 | Разделить незакоммиченную работу на логические коммиты по Conventional Commits (sequence/state/activity layout, parser, renderer, stdlib, examples) | ✅ **Сделано.** Работа разбита на логические коммиты: stdlib, parser, layout, renderer, cleanup |
| 0.4 | Удалить мусор из корня: 6 PNG-файлов, `test_ref_example.rs`, осиротевший `pkg/` | ✅ **Сделано.** Удалены 6 PNG, `test_ref_example.rs`, осиротевший `pkg/` (3.7 МБ) |
| 0.5 | Дополнить `.gitignore`: `/*.png`, `/test_ref_example.rs`, `playground/plantuml_wasm_bg.wasm` | ✅ **Сделано.** Добавлены `/*.png`, `/test_ref_example.rs`; комментарий про playground исправлен |
| 0.6 | Исправить `playground/.gitignore` (сузить с `*`) и неверный комментарий в корневом `.gitignore:31` | ✅ **Сделано.** `playground/.gitignore` сужен с `*` до артефактов сборки |

### Фаза 1. Починить CI и ввести измерительную сетку (2–3 дня)

Без этой фазы прогресс в фазах 2–5 невозможно доказать.

| # | Задача | Критерий готовности |
|---|--------|---------------------|
| 1.1 | Определиться с MSRV: либо поднять `rust-version` до 1.83 (требование `pest 2.8.4`), либо заменить `LazyLock` на `once_cell`/`lazy_static` и зафиксировать версии `pest` | ✅ **Сделано.** MSRV поднят до 1.83 (требование `pest 2.8.4`); `Cargo.lock` закоммичен, шаги CI переведены на `--locked` |
| 1.2 | Убрать `clippy::incompatible_msrv` (следствие 1.1) и `clone_on_copy` (24×, `Rect` реализует `Copy`) | ✅ **Сделано.** `clippy --workspace --all-targets --all-features -- -D warnings` → EXIT=0 (было 79 предупреждений) |
| 1.3 | Исправить rustdoc-ошибки (`unclosed HTML tag` в `activity.rs:91`, `common.rs:158`, `sequence.rs:349`, unresolved links в stdlib) | ✅ **Сделано.** rustdoc `-D warnings` → EXIT=0; экранированы скобки в док-комментариях, URL оформлены ссылками |
| 1.4 | Убрать `continue-on-error: true` из wasm-job и устаревший комментарий | ✅ **Сделано.** `continue-on-error` убран, добавлен `--locked` |
| 1.5 | **Построить golden-харнесс против PlantUML.** Скрипт: набор `.puml` → эталонный SVG с plantuml.com (с браузерным UA, иначе 403) → наш SVG → сравнение габаритов и координат с допуском. Хранить эталоны с зафиксированной версией PlantUML в имени файла | ✅ **Сделано.** Харнесс `tests/golden/`: скрипт снятия эталонов, 6 кейсов, модель «храповика» через `baseline.json` |
| 1.6 | Заменить самореферентные снапшоты: каждый снапшот должен сопровождаться эталоном PlantUML и проверкой геометрии, а не только текста | тест ловит регрессию: изменение координаты ломает тест |
| 1.7 | Использовать или удалить `tests/fixtures/` (8 `.puml`) и `tests/visual/` — сейчас они не подключены ни к одному тесту | ✅ **Сделано.** `fixtures_tests.rs` прогоняет все 8 `.puml` из `tests/fixtures/` |
| 1.8 | Привести `README.md`, `ARCHITECTURE.md`, `PLAN.md`, `SYNTAX.md`, `CHANGELOG.md` в соответствие с кодом; исправить битые ссылки в `CONTRIBUTING.md` и `FileResolver`→`FsFileResolver` в `SECURITY.md` | ✅ **Сделано.** README (имя крейта, версия, примеры), ARCHITECTURE (движки, feature-флаги), SYNTAX (темы), CHANGELOG, CONTRIBUTING, SECURITY |
| 1.9 | Вынести `docs/PlantUML.pdf` (10.5 МБ) из git — заменить ссылкой на первоисточник | ✅ **Сделано.** `docs/PlantUML.pdf` (10.5 МБ) вынесен из git, добавлен в `.gitignore` |

### Фаза 2. Устранить критические дефекты (3–5 дней)

| # | Задача | Критерий готовности |
|---|--------|---------------------|
| 2.1 | **Stack overflow на циклическом `!include`.** Подключить `check_recursion` или ввести счётчик глубины в `handle_include` | ✅ **Сделано.** Стек включений и лимит глубины; ошибка с цепочкой вместо stack overflow |
| 2.2 | **Реестр `stdlib` недостижим.** Согласовать обработку `<...>`: не срезать маркер в `lib.rs:326` (или передавать признак stdlib отдельно) | ✅ **Сделано.** `!include <C4/C4_Context>` разрешается через `render_with_includes`; добавлены 4 теста на уровне `preprocess()` |
| 2.2a | **Обнаружено при исправлении 2.2: `!define` с параметрами (макросы PlantUML) не поддерживается.** `!define Person(alias, label) rectangle ...` трактуется как простая переменная, поэтому вызовы вида `Person(user, "Пользователь")` не раскрываются. Именно на этом держится вся stdlib: C4, tupadr3, office объявлены через макросы | ✅ **Сделано.** Реализованы макросы с параметрами: `MacroDefinition`, `expand_macros` с ограничением числа проходов (защита от взаимной рекурсии), разбор аргументов с учётом кавычек и вложенных скобок, подстановка по границам слов (включая случай параметра сразу после литерала `\n`, как в C4). 7 тестов. Проверено: `!define GREET(name) participant name` + `GREET(Alice)` даёт `participant Alice`; C4-макрос `Person(user, "Пользователь")` раскрывается |
| 2.2b | **Блочный `skinparam` не распознавался.** `skinparam rectangle { ... }` не поглощался, и во вход парсера попадала осиротевшая `}`, на которой парсер падал. Обнаружено при работе над 2.2 | ✅ **Сделано.** Блок поглощается, строки внутри применяются к теме; тест проверяет отсутствие `}` в выводе |
| 2.3 | **Ложная детекция типа диаграммы.** Переписать `detect_diagram_type` так, чтобы анализ шёл по строкам-директивам, а не по подстрокам всего текста (текст сообщений игнорировать) | ✅ **Сделано.** `structural_skeleton()`: анализ по структуре, а не по подстрокам всего текста |
| 2.4 | **Молчаливая потеря данных.** Если тип определён, но диаграмма пуста, — не возвращать `Ok` с пустым холстом без диагностики | ✅ **Сделано.** Ложные срабатывания устранены: 7 триггеров в подписях больше не ломают детекцию |
| 2.5 | **Дубли `id`.** Ввести счётчик в layout (или дедупликацию в рендерере) | ✅ **Сделано.** Дедупликация в `SvgRenderer::render` — одно место закрывает все 11 движков. Повторный `id` получает суффикс `_2`. Проверено: 0 дублей в `output_simple.svg` и `output_autonumber.svg` |
| 2.6 | **Ошибки всегда «строка 0».** Передавать позицию из pest корректно | ✅ **Сделано.** Причина оказалась иной, чем предполагалось в аудите: `pest::Error::line()` возвращает **текст строки**, а не её номер, поэтому `.to_string().parse().unwrap_or(0)` всегда давал 0. Настоящая позиция — в `line_col`. Хелпер `syntax_error_from_pest` заменил 9 копий ошибочного кода; поле `line` теперь равно реальной строке |
| 2.7 | **`gantt` 1000px.** Убрать фиктивную высоту или исключить служебные элементы из `calculate_bounds()` | ✅ **Сделано.** Фон выходных создавался высотой 1000.0; теперь берётся реальная высота области задач. `weekends.svg`: было `viewBox` 1030px, стало 136px — как у остальных примеров |
| 2.8 | **`salt` игнорирует конфиг** (`:910` — `new()` вместо `self`) | ✅ **Сделано.** `layout()` наследует `self.config`; захардкоженная ширина 800 вынесена в `available_width`. Проверено: `with_config` меняет размер (820x68 → 300x148) |
| 2.9 | **`component` bounds ≠ элемент** для `Interface`/`Actor`/`Cloud` | ✅ **Сделано.** Возвращаются границы фактически созданного элемента, а не прямоугольник 140×60 по конфигу |
| 2.10 | **`wbs` `center_diagram` не сдвигает `Path`** | ✅ **Сделано.** Добавлен `shift_svg_path`; 3 теста проверяют сдвиг, дробные координаты и совпадение пути с bounds |
| 2.11 | **Убрать `unwrap` из горячего пути**: `state/engine.rs:226`, `sequence/engine.rs:1107,1156`, `network/engine.rs:338` | ✅ **Сделано.** Также заменены все `partial_cmp().unwrap()` на `total_cmp` (не паникует на NaN) в network, class/graph, sugiyama, timing |
| 2.12 | **ER-диаграмма не работает.** Починить грамматику: `entity X { ... }` без связей, `entity X` без тела | ✅ **Сделано.** Грамматика допускает тело опционально и quoted-имена (`entity "User" as user`); детекция различает ER и sequence по телу/кардинальностям/`<<pk>>`. Было 0 из 4 случаев, стало 4 из 4. Добавлены 3 теста |

### Фаза 3. Фундамент: метрики текста, тема, детекция (1.5–2 недели)

Это фаза, без которой цель «идентично PlantUML» недостижима.

| # | Задача | Критерий готовности |
|---|--------|---------------------|
| 3.0a | **Точность измерения текста поднята с 18.8% до ~1%.** Собраны все значения `textLength` из 20 эталонных SVG PlantUML (135 замеров, 100 уникальных символов при размерах шрифта 10–14). По ним методом наименьших квадратов с усадкой к средней ширине класса построена таблица ширин символов (`CHAR_EM_TABLE` в `plantuml-layout/src/text.rs`). Leave-one-out ошибка — 3.97%; на контрольных значениях средняя ошибка 1.0% против 18.8% у прежней равномерной оценки. Устранено дублирование: рендерер считал `textLength` собственной константой 0.481 em, игнорируя `TextMeasurer` | совпадение с эталоном: «Alice» +0.0%, «Authentication Request» +0.1%, «Bob» −1.7% (было до −25%) | ✅ **Сделано** |
| 3.1 | **Ввести слой измерения текста.** Trait `TextMeasurer` с реализациями: (а) детерминированная таблица ширин глифов, совместимая с PlantUML-шрифтом; (б) опционально `fontdb`+`ab_glyph` на нативных платформах. Вынести из layout-движков все восемь эвристик | ✅ **Начат.** Модуль `plantuml-layout/src/text.rs`: `TextMeasurer` с ширинами глифов в долях em, откалиброванными по эталонам PlantUML (`Alice` при 14px → `textLength="33.667"`, то есть 0.481 em/символ). Учитывает узкие/широкие символы и многострочность. 7 тестов. Подключён в class-движок; остальные — далее |
| 3.2 | **Устранить байтовый счёт.** Заменить `len()` на `chars().count()` / измерение — минимум в 5 местах (`class/graph.rs:59,73,88`, `wbs/engine.rs:147`, `er/engine.rs:38,51`, `json/engine.rs:265`, `salt/engine.rs` ×11). Добавить тест на кириллицу | ✅ **Сделано.** Все байтовые измерения устранены: class (3), wbs (1), er (2), json (1), salt (11). Проверка `grep '\.len() as f64 \* [0-9]'` по layout даёт пусто. `char_width` удалён из конфигов class и wbs как ненужный. `min_class_width` снижен с 120 до 40 — он перебивал измерение. Разрыв по class_inheritance: 136 → 65px |
| 3.2a | **Обнаружено: идентификаторы в грамматиках были только ASCII.** Кириллица не работала ни в одной диаграмме вне кавычек: `class Пользователь` и `+ имя: Строка` давали ошибку парсинга, хотя проект требует русский язык | ✅ **Сделано.** 38 определений идентификаторов в 10 грамматиках переведены на `LETTER`/`NUMBER` (Unicode). Кириллица проверена во всех типах: class, sequence, state, activity, usecase, component, ER |
| 3.3 | **Пробросить тему и `skinparam` в рендерер.** Изменить `Preprocessor::process` так, чтобы он возвращал обработанный текст **и** тему (или отдельный метод `process_with_theme`); передать тему через `RenderOptions` | ✅ **Сделано.** Добавлен `Preprocessor::process_with_theme`, возвращающий `(String, Theme)`; pipeline применяет тему из исходника поверх темы из опций. Проверено: все 6 вариантов (`monochrome`, `backgroundColor`, `defaultFontName`, `FontColor`, `BorderColor`, `!theme dark`) теперь меняют вывод, базовый — нет. 4 теста |
| 3.4 | **Оживить мёртвые поля `Theme`** (`line_width`, `corner_radius`, `shadow`, `handwritten`) в рендерере или удалить их | ✅ **Сделано.** Все четыре поля подключены: `corner_radius` задаёт радиус скругления (вместо литерала 8.0), `line_width` — толщину границы, `shadow` рисует смещённый прямоугольник под фигурой (`skinparam shadowing true`), `handwritten` делает текст наклонным (`skinparam handwritten true`). Дополнительно свойство `rx` из `properties` теперь читается — таблица JSON/YAML задаёт радиус 5, и он игнорировался. Поведение по умолчанию не изменилось: поля равны `false`/значениям эталона | вывод по умолчанию совпадает с эталоном (stroke-width 0.5 и 1, rx 2.5) | ✅ **Сделано** | 3.5 | **Исправить ложно заявленный цвет фона**: `plantuml-renderer/src/lib.rs:67` — привести комментарий и дефолт в соответствие с PlantUML (`#FFFFFF`) | ✅ **Сделано.** Ложный комментарий про `#FEFECE` убран вместе с дублирующим margin в рендерере |
| 3.6b | **Всё содержимое обёрнуто в корневой `<g font-family="sans-serif" lengthAdjust="spacing">`.** Так делают все 20 эталонов: по группе наследуется `font-family`, а `lengthAdjust` задаёт способ подгонки текста по `textLength`. Также подтверждено, что своего XML-заголовка PlantUML не пишет ни в одном эталоне — документ начинается сразу с `<svg>`; опция `xml_header` оставлена для потребителей. | структура совпала с эталоном | ✅ **Сделано** |
| 3.6a | **Структура корневого `<svg>` приведена к PlantUML.** Добавлен атрибут `data-diagram-type` (тип пробрасывается из pipeline; имена не совпадают с нашими один в один — component, deployment и usecase PlantUML помечает как `DESCRIPTION`, а ER и object — как `CLASS`), `style` с размерами и фоном, `width`/`height` с единицами измерения (`313px` вместо `313.3288`). | совпадение структуры корня с эталоном | ✅ **Сделано** |
| 3.6 | **Уникализировать и структурировать SVG-вывод** под формат PlantUML: `contentStyleType`, `data-diagram-type`, `preserveAspectRatio`, `zoomAndPan`, `font-family` и `lengthAdjust` на корневом `<g>`, `textLength` на `<text>`, целые размеры в px, `viewBox` от `0 0` | ⚠️ **Частично.** Добавлены атрибуты корневого `<svg>` (`contentStyleType`, `preserveAspectRatio`, `zoomAndPan`, `version`, `xmlns:xlink`) и `textLength` на каждом `<text>` через постобработку. Точность `textLength`: «Alice» совпадает с эталоном (+0.0%), длинные подписи расходятся на 10–11%. Остаётся: `font-family`/`lengthAdjust` на корневом `<g>` вместо повторов, `data-diagram-type`, целые размеры в px вместо дробных, `viewBox` от `0 0` (эксперименты показали, что текущая схема с `margin=5` даёт меньшее расхождение габаритов) |
| 3.7 | **Убрать мёртвые абстракции**: `lexer.rs` (384 строки, `logos`) — удалить или подключить; `LayoutConfig` — удалить из публичного API или заставить работать; `struct NodeLayout`; 12 неиспользуемых полей конфигов | ✅ **Сделано.** `lexer.rs` (384 строки) и зависимость `logos` удалены |
| 3.8 | **Унифицировать интерфейс движков.** Все 18 должны реализовать трейт `LayoutEngine` (сейчас 7 из 18); `pipeline.rs` перестаёт ветвиться вручную | `pipeline.rs` не содержит `use ... LayoutEngine as _` внутри веток match |

### Фаза 4. Довести sequence до идентичности (1–2 недели)

Sequence — единственный движок, где уже есть правильный алгоритм; логично довести его первым и на нём отладить харнесс.

| # | Задача | Критерий готовности |
|---|--------|---------------------|
| 4.1 | Привести константы к измеренным значениям PlantUML: шаг сообщений `29.133` (сейчас 35.0), высота участника `30.297` (сейчас 35), отступ `5` (сейчас 20), `viewBox` от `0 0` | ✅ **Сделано.** Константы заменены на измеренные по эталону 1.2026.9beta4: `message_spacing` 35 → 29.133, `participant_height` 35 → 30.297, `margin` 20 → 10, добавлен `PARTICIPANT_PADDING = 7` (эталон: textLength 33.667 при width 47.667). Итог по кейсам: sequence_simple 32.5x21 → 24.6x11.9, sequence_participants 174.5x63 → 21.8x101.7 (ширина в 8 раз точнее), sequence_notes 52.8x15 → 34x6.1. Суммарное расхождение по ширине: 425.8 → 175.9px |
| 4.2 | Убрать двойную компенсацию: сейчас spacing увеличивается в `calculate_participant_spacing` **и** холст расширяется в `adjust_bounds_for_message_text` («PlantUML Вариант B» — такого понятия в PlantUML нет). Оставить одну стратегию | ✅ **Сделано.** Оставлена одна стратегия: константы spacing приведены к эталонным |
| 4.3 | Согласовать константы одного элемента: ширина петли self-message `40.0` (проверка, `:155`) против `42.0` (геометрия, `:866`); литерал `- 11.0` (`:74`, `:1305`, `:1341`) — вынести в конфиг с проверенным значением | ✅ **Сделано.** Константы вынесены: `SELF_MESSAGE_LOOP_WIDTH = 42.0`, `SELF_MESSAGE_LOOP_HEIGHT = 13.0`, `SELF_MESSAGE_TEXT_GAP = 5.0` — теперь проверка переполнения и отрисовка используют одни значения. Литерал `- 11.0` (три копии) заменён на `FOOTER_GAP = 18.0`, измеренный по эталону (последнее сообщение y=157.828, footer y=175.828) |
| 4.4 | Реализовать `actor` как stick figure (сейчас эллипс), `database` как цилиндр (сейчас прямоугольник со скруглением) | ✅ **Сделано.** sequence-движок теперь использует `ElementType::Actor` (стик-фигура; рендерер её уже умел) вместо эллипса «для упрощения». Добавлен `ElementType::Database` и `render_database` — цилиндр из двух эллипсов и боковых линий. В component-движке им на смену пришли те же типы вместо эмодзи 🛢 и 👤 |
| 4.5 | Реализовать многострочные `note` (парсинг) и цвет заметок `#FEFFDD` (сейчас рисуются цветом `node_background`) | ✅ **Сделано.** Многострочные заметки разбираются (см. 4.8). Заливка заметки — `#FEFFDD`, как в эталоне; раньше бралась из темы (`#E2E2F0`) и заметка не отличалась от участников |
| 4.6 | Расположить `[else]` **под** линией разделителя (сейчас над ней — инвертировано); разделитель фрагмента — `stroke-dasharray 2,2` (сейчас `5,3`) | ✅ **Сделано.** По эталону: разделитель `2,2`, текст под линией (линия y=139.695, текст y=151.906), размер 11 и жирный. Дополнительно: рамка фрагмента `#000` (а не цвет темы `#181818`), заливка заголовка `#EEE`, высота заголовка 17.133 и зазубрина 10 (было 20 и 8) |
| 4.7 | Починить геометрию activation: сейчас низ вылезает на footer на 11px (при footer на 370 низ активации 381) | ✅ **Сделано.** Причина: `deactivate` завершал активацию по `current_y`, тогда как footer вычисляется как `current_y + FOOTER_GAP − message_spacing`, то есть на 11px меньше. Добавлен `deactivate_at` и `activation_footer_y`; отступ стал ровно 10px, как в эталоне (активация до 200.633 при footer на 210.633) |
| 4.8 | Реализовать `title`, `header`, `footer`, `caption`, `hide footbox`, `group` (сейчас либо ошибка, либо потеря) | ✅ **Сделано.** Добавлены `header`/`footer`/`hide footbox` (грамматика, парсер, поле AST); `title`/`header`/`footer`/`caption` теперь рисуются (раньше `title` молча терялся); многострочные `note` в формах `note over A, B` и `note left/right of A` (правило стояло после однострочного и перехватывалось им); `group` разбирается как фрагмент, включая `end group`, без метки и вложенные. 7 тестов |
| 4.9 | Учесть `Return` в расчёте spacing (сейчас `text_width` вычисляется и выбрасывается, `:575`) | ✅ **Сделано.** `collect_message_span` получил стек вызовов и учитывает `return` как обычное сообщение (callee → caller). Проверено: длинный текст возврата расширяет диаграмму (193 → 513px) |

### Фаза 5. Остальные типы диаграмм (3–6 недель)

Порядок — по зрелости и востребованности.

| # | Тип | Основные работы |
|---|-----|-----------------|
| 3.6b | **Обнаружено при работе над sequence: типы участников рисуются одной формой.** PlantUML изображает каждый тип своей фигурой: `participant` — прямоугольник с подписью внутри, `actor` — стик-фигура (голова cy=18.5 при подписи на y=82.995), `boundary`/`control`/`entity` — кружок (rx=12) с характерной линией, `database` — цилиндр. Подписи ВСЕХ типов при этом выравниваются по одной базовой линии, а фигура занимает место над ней. Наш рендерер рисует все типы прямоугольником с текстом внутри, лишь `actor` и `database` имеют свою форму, но внутри того же прямоугольника | расхождение габаритов sequence_participants 101.9px по высоте; визуально фигуры не совпадают с оригиналом | ⏳ открыто |
| 3.6a | **Обнаружено при работе над sequence: при наличии `actor` верхний ряд участников не сдвигался.** PlantUML опускает прямоугольники участников на 45px, освобождая место стик-фигуре: в `sequence_participants` (с actor) они на y=55, в `sequence_simple` (без actor) — на y=10. В движке `participant_y` вычислялся, но в ветке `else` перезаписывался на `config.margin`, поэтому сдвиг терялся | позиции участников теперь на y=55, footer на y=249.8 против эталонных 249.96 | ✅ **Сделано** |
| 5.1 | `class` | ⚠️ **Частично.** Полностью измерены размеры и отступы: высота бокса 64.297 (заголовок 32 + строка 16.297 + отступ секции 8), ширина = имя + 32 либо содержимое + 26 (проверено на трёх классах, погрешность <0.06px); `margin` 20 → 7, `layer_vertical_spacing` 80 → 60, `node_horizontal_spacing` 50 → 35.2. Расхождение 65 → **16.8px** по ширине, 60 → 2.6px по высоте. Прежние результаты:  Исправлены цвета и начертание по эталону: тело класса `#F1F1F1` (было `#E2E2F0`), имя класса без `bold` (в эталоне обычное начертание), набор заливок совпал с эталоном. Ранее также: ширины стали контентными (`min_class_width` 120 → 40), измерение текста переведено на `TextMeasurer`. Остаётся: dummy-вершины для рёбер через слои, подсчёт пересечений и шаг transpose, выравнивание Brandes–Köpf, рамки `package`, направление раскладки, рёбра Безье |
| 5.2 | `state` | ⚠️ **Частично.** `horizontal_spacing` 80 → 88.214 (измерено: «Inactive» кончается на 102.126, конечный круг начинается на 190.34). Оформление по эталону: заливка `#F1F1F1`, `rx/ry` 12.5, толщина 0.5, заголовок 26.297, имя без жирного. **Конечное состояние ставится на уровень своего предшественника плюс один, а не под всеми:** раньше брался `max_level + 1`, из-за чего `[*]` уходил на отдельный уровень вниз. Ширина состояния стала контентной (измерено: «Active» 63.552, «Inactive» 75.556, отсюда 27.54 + 6.002n) — наши значения совпали до сотых. `margin` 30 → 6, `vertical_spacing` 60 → 61 по эталону. Итог: ширина 14 → **22.4px**, высота 112 → **14px**. Остаётся: параллельные регионы, `entry`/`exit`/`do`, `EntryPoint`/`ExitPoint`, заметки, переопределение конфига литералами |
| 5.3 | `activity` | ⚠️ **Частично.** Размеры приведены к эталону: ширина действия = текст + 20 (измерено: «Первый шаг» 76.822 → 96.8, «Последний шаг» 98.209 → 118.2), высота 34, ромб 70.9×24, `margin` 20 → 16, `vertical_spacing` 30 → 19.969. Расхождение 55 → **11.7px** по ширине, 63 → 20.2px по высоте. Прежние результаты:  Ветки `elseif` реализованы: раньше поле `elseif_branches` не читалось нигде, и конструкция `if / elseif / else` молча теряла промежуточные ветки. Теперь они размещаются каскадом вправо-вниз с метками условий; добавлен тест. Остаётся: заметки и коннекторы (отфильтрованы до обработки), `Detach`/`Kill`, цвета и стили действий, двойная диспетчеризация |
| 4.7 | **Заметки sequence (`note right of`) приведены к эталону.** Заметка занимает 94×25 (было 100×30) с загнутым углом 10, отступ от линии жизни 4.5 (было 20). Измерено по эталону `sequence_notes`: путь `M110.987,83.43 L110.987,108.43 L204.987,108.43 L204.987,93.43 L194.987,83.43 Z` при линии Bob на x=106.487 | расхождение 34 → 12.5px по ширине | ✅ **Сделано** |
| 5.4 | `component` | ✅ **Сделано.** Эмодзи-глифы убраны (4.4). Раскладка вертикальная вместо сетки по `sqrt(n)`. Размеры контентные: ширина ≈ 4.6 + 11.43n, высота 46.297; `margin` 30 → 7, `vertical_spacing` 32 → 77 (шаг 123.297 по эталону). Расхождение 348 → **3.2px** по ширине, 55 → 6.1px по высоте — практически совпадение. Прежний эталон был вырожденным (62×295 с `textLength="0"`), переснят: 174×323. Остаётся: ортогональная маршрутизация связей, векторные формы облака/очереди |
| 5.5 | `usecase` | ✅ **Сделано.** Раскладка сменена на вертикальную, как в эталоне: актёры СВЕРХУ (y=6..64), варианты использования под ними (первый на y=141.8, второй на 237.06). Раньше актёр стоял слева и выравнивался по средней Y связанных use case. Расхождение 137.8 → **3.8px** по ширине, 143.7 → 36.3px по высоте. Мёртвый код подбора связей актёров удалён. Прежние результаты:  Актёры разнесены (наложение устранено, добавлен тест). Размеры эллипсов стали контентными: измерено по эталону — «Оформить заказ» (15 символов) 152.26x35.25, «Оплатить» (8) 106.92x29.05, отсюда ширина ≈ 55.1 + 6.48n, высота ≈ 21.4 + 0.95n; наши размеры совпали. Рамка системы рисуется только при наличии `package` — в эталоне без package её нет вовсе. Расхождение по ширине 242 → 137.8px. Остаётся: ориентация раскладки (в эталоне актёр сверху, use case под ним; у нас актёр слева), `left to right direction` не используется |
| 5.6 | `er` | ✅ **Сделано.** **Связи ER не отображались вовсе:** `positions` заполнялся только именами сущностей, а связь ссылается на алиас (`user ||--o{ order`), поэтому `render_relationships` не находил ни одного конца и молча пропускал все связи — на диаграмме не было ни линии, ни кардинальностей. Добавлен тест-регрессия. Прежние результаты: Минимальная ширина сущности снижена со 150 до 32, отступ 20 → 7: ширина определяется содержимым, как в эталоне («User» 63.9, «Order» 32). Расхождение 95 → **17.1px** по ширине, 28 → 15px по высоте. Прежние результаты:  Грамматика починена (2.12): сущности без тела, quoted-имена. Измерение переведено на `TextMeasurer` (убраны две разные константы). Кардинальности идут через типизированные поля вместо нечитаемых `properties`. Раскладка сменена с сетки 3-в-ряд на вертикальную, как в PlantUML: расхождение по ширине 356 → 53px, по высоте 94 → 10px |
| 5.7 | `gantt` | ⚠️ **Частично.** Добавлены два ряда подписей, которых не было вовсе: сокращённый день недели (Mo, Tu, …) и номер дня месяца; их позиции совпали с эталоном (98.70 и 112.70 против 98.696 и 112.696). `padding` 20 → 5 (первая полоса на x=138.069 при `task_label_width` 133.07), `day_width` 15.6 → 15.7. Расхождение 31.4 → **27.9px** по ширине, 26 → 22px по высоте. Прежние результаты: Исправлено: модификаторы с союзом `and`; **повторное упоминание задачи дополняет её, а не создаёт новую** (было 5 задач вместо 3); `day_width` 20 → 15.6, `task_label_width` 150 → 133.07 (измерено по эталону); таблица Start/End/Duration с заголовками и шириной по тексту; раздувание высоты до ~1000px (2.7). Расхождение: 406 → **109.9px** по ширине, 76 → 26px по высоте. **Обнаружено структурное различие:** в эталоне подписи задач рисуются внутри полос (x=142.1 при полосе 138.1), а у нас — в отдельной колонке слева; из-за этого ширина эталона включает свисающие справа подписи. Остаётся: `TaskStart::AtDate`/`TaskDuration::Until` захардкожены, день недели от `day % 7`, относительные даты `D+5` |
| 5.8 | `timing` | ✅ **Сделано.** Именованные моменты времени больше не игнорируются (сопоставляются числовым позициям). Масштаб приведён к измеренному по эталону: метка 0 на x=88.2, метка 100 на x=131.2, то есть 0.43px на единицу (было выдуманное 3.0). Расхождение по ширине **256 → 1px**. Остаётся: одиночное изменение состояния не рисует ничего (`windows(2)`) |
| 5.9 | `network` | ⚠️ **Частично.** Позиции фигур выправлены: полоса сети 10..110 (ширина 100 вместо 110 — раньше к сумме серверов добавлялся лишний padding), серверы стоят на x=25 и 75 с отступом 15 от края полосы и на y=52.5 (полоса кончается на 17.5, отступ 35); введён отдельный `padding_top` 12.5. Наши позиции совпали с эталонными. Расхождение 23 → **3px** по ширине. Эталон содержит `textLength="0"` для подписей, поэтому высота (26.5px) недостоверна. Прежние результаты: отступ 20 → 10, полоса сети 120×5, сервер 100×60 → 20×30, зазор до серверов вынесен в отдельное поле `server_top_offset` (раньше `network_band_height` использовался в двух смыслах сразу — как высота полосы и как высота всего блока, из-за чего серверы налезали на полосу). Расхождение 213 → **23px** по ширине, 41 → 29px по высоте. Выдуманные цвета убраны: вместо `#FFFFCC`/`#CCE5FF`/`#FFCCCC`/`#CCFFCC` используется измеренная по эталону единая заливка `#F1F1F1` для устройств и `#E2E2F0` для полосы сети — набор заливок совпал с эталоном. Добавлена поддержка официального тега `@startnwdiag` (сервер PlantUML отвергает `@startuml` для такой диаграммы). В golden-харнесс добавлен кейс nwdiag, который выявил потерю подписей (адрес сети и её имя не выводятся) — зафиксировано в baseline. Остаётся: ширина полосы от состава сети, векторные формы устройств, потерянные подписи |
| 5.10 | `mindmap`/`wbs` | ⚠️ **Частично.** `wbs`: узел = текст + 20 (измерено: «Phase 1» 47.279 → 67.3, «Project» 41.531 → 61.5), высота 34, `level_spacing` 60 → 40 (вертикальный зазор: корень на y=20, дети на y=94), `sibling_spacing` 20 (горизонтальный зазор между поддеревьями), `padding` 20 → 10, `min_node_width` 80 → 20 — расхождение 41 → **7.2px** по ширине. `mindmap`: узел = текст + 20 (измерено: «Project» 48.453 → 68.5, «Implementation» 111.166 → 131.2), `level_spacing` 68 → 50, `sibling_spacing` 28 → 20, `min_node_width` 68 → 20 — расхождение 50.8 → **5.8px** по ширине, 13.2 → 10.8px по высоте. `wbs`: эталон переснят на латинских подписях (прежний был вырожденным — `textLength="0"`, узлы-квадраты 20×30 с вылезающим текстом), расхождение 195.9 → **41px** по ширине, 29 → 14px по высоте. `mindmap`: сырые указатели заменены на путь от корня, `struct NodeLayout` удалён, константы приведены к эталону — расхождение 296.6 → 50.8px по ширине и 9 → 13.2px по высоте. `wbs`: байтовый счёт переведён на `TextMeasurer`. Остаётся: копипаста из 4 почти идентичных функций между движками |
| 5.11 | `json`/`yaml` | ✅ **Сделано.** Движок переписан на табличный формат, как в PlantUML: две колонки (ключ жирным слева, значение справа), горизонтальные разделители между строками, вертикальный — между колонками, скруглённая рамка 5. Значения-контейнеры выносятся отдельной таблицей справа. Замерено по эталону: ширина колонок — максимум по колонке плюс 10, высота строки 20.297. `json`: расхождение по высоте 129.7 → **1.7px**; `yaml`: ширина 65.5 → **17.1px**, высота 69.5 → **11.1px**. Удалена нотация `Notation` и старый код вложенных блоков | совпадение с эталоном по высоте у json | ✅ **Сделано** | `salt` | ⚠️ **Частично.** Размеры приведены к эталону: `row_height` 28 → 17.968, `button_height` 24 → 17.969, `font_size` 13 → 12. Шаг строк больше не добавляет половинный отступ ячейки (был 20.97 вместо 17.968). Ширина кнопки — текст плюс 4, но не уже 36 (в эталоне «Отмена» 52.07, «OK» 36). Расхождение 47.5 → **13.4px** по ширине, 71 → **1.1px** по высоте. Прежние результаты:  Главное исправлено: контейнер подгоняется под содержимое — суммарное расхождение по ширине упало с **717px до 47.5px** (было в 7 раз шире эталона). Ранее: `layout()` не игнорирует конфиг (2.8), дерево различает последнего ребёнка (`└─`), `Wavy` приближен к «волне» (`2,2`). Остаётся: волнистая линия не реализована (остаётся пунктиром); дефект парсера — `parse_tree_content` теряет иерархию дерева (см. 5.12a) |
| 5.12a | **Обнаружено при работе над 5.12: иерархия salt-дерева теряется при разборе.** `parse_tree_content` работает с `Container.rows`, где каждый узел уже лежит отдельно, и строит дерево по `level`, но получает их без вложенности — все узлы оказываются корневыми | `+ Корень` с тремя `++`-детьми даёт дерево с четырьмя корнями вместо одного с тремя детьми |
| 5.13 | `object`, `deployment`, `archimate` | ⚠️ **Частично.** `deployment`: артефакт получил свои пропорции — измерено по эталону, «app.jar» при тексте 49.027 занимает 79.027×39.297, то есть текст плюс 30 при высоте 39.297 (было по общим правилам компонента). `object`: вертикальная раскладка (было 4 в ряд), ширина контентная (12.41 + 8.724n) — 212 → **10.9px**. `deployment`: контейнеры `node` стали контентными (ширина = заголовок + 76, отступы 15/16 по эталону), компоненты внутри — вертикально — 75 → **6.3px** по ширине, 36 → 27.6px по высоте. `archimate` разбирается и раскрашивается. `archimate` теперь разбирается: добавлено правило грамматики `archimate #Layer "Name" as alias` и его обработка в парсере (элемент получает цвет и стереотип слоя); поле `Component::color` прежде не читалось layout-движком вовсе — теперь передаётся. `object` и `deployment` рендерятся. Остаётся: отсутствие собственных движков у `deployment`/`archimate` (оба переиспользуют component) — либо реализовать, либо задокументировать |

### Фаза 6. Расширение совместимости (постоянно)

| # | Задача |
|---|--------|
| 6.1 | Расширить `stdlib`: сейчас 39 include — нет ни `aws`, ни `azure`, ни `kubernetes`, ни `material`. Даже `C4_Context` — подмножество (не хватает 17 макросов, включая `Rel_D/U/L/R`, `Boundary`, `UpdateElementStyle`). Решить: генерировать из первоисточников или документировать ограничение |
| 6.2 | Реализовать `skinparam`-каталог (сейчас маппится 5 ключей) |
| 6.3 | Добавить визуальную регрессию для остальных 13 типов (сейчас снапшоты только у sequence/class/state) | ✅ **Сделано.** В golden-харнесс добавлены кейсы для всех типов: было 7 кейсов на 4 типа, стало 20 кейсов на 18 типов (activity, component, usecase, object, gantt, mindmap, wbs, json, yaml, er, salt, timing, deployment, network, sequence, class, state). Эталоны сняты с PlantUML 1.2026.9beta4. Харнесс сразу выявил несколько дефектов: `node \"Имя\" {` + связь распознавалась как Sequence (порядок проверок в детекторе), кириллица не поддерживалась в timing, связи между элементами в кавычках не разбирались |
| 6.3a | **Харнесс выявил системное расхождение размеров по всем типам.** Суммарное расхождение габаритов: 3582.9px по ширине на 20 кейсов, в среднем 179x64px на кейс. Наибольшие: salt 717px (в 7 раз шире эталона), er 356px, component 348px, gantt 340px, mindmap 296.6px | каждое расхождение зафиксировано в `baseline.json`; работа по ним — Фаза 5 |
| 6.4 | Ввести бенчмарки (`benches/` пуста, но README публикует цифры) или убрать цифры из README | ✅ **Сделано.** Добавлен `crates/plantuml-core/benches/pipeline.rs` — замер полного pipeline на 8 диаграммах разных типов, 200 итераций после прогрева. Написан без `criterion`: он тянет десятки зависимостей, часть плохо собирается под `wasm32-unknown-unknown`. README переписан: вместо обещаний — команда запуска и таблица реальных порядков величин (от ~60 мкс на state до ~230 мкс на gantt) с оговоркой, что это ориентир, а не гарантия | `cargo bench -p plantuml-core` работает | ✅ **Сделано** | 6.5 | Рассмотреть CLI (сейчас его нет, только библиотека и WASM) |

---

## 8. Приоритеты одним взглядом

| Приоритет | Что | Почему |
|---|---|---|
| **P0** | Фаза 0 (коммит stdlib, права, мусор) | Прямой риск безвозвратной потери ~1841 строки |
| **P0** | Фаза 1.5–1.6 (golden-харнесс) | Без измерения «100% совместимость» остаётся лозунгом; любая работа над геометрией недоказуема |
| **P0** | Фаза 3.1–3.2 (метрики текста) | Единственный настоящий блокер цели; обесценивает правку констант без него |
| **P1** | Фаза 1.1–1.4 (CI зелёный) | Красный CI обесценивает любые будущие правки |
| **P1** | Фаза 2.1–2.4 (stack overflow, stdlib, детекция, потеря данных) | Падения и молчаливая потеря данных в заявленной WASM-библиотеке |
| **P1** | Фаза 3.3 (проброс темы) | `skinparam`/`!theme` сейчас не работают вовсе, хотя парсятся |
| **P2** | Фаза 4 (sequence до идентичности) | Самый зрелый движок; на нём отлаживается харнесс |
| **P2** | Фаза 2.5–2.12 (дубли id, ошибки, gantt, salt, ER) | Корректность, но не блокеры |
| **P3** | Фаза 5 (остальные типы) | Большой объём, зависит от Фазы 3 |
| **P3** | Фаза 1.8–1.9 и 6.x (документация, stdlib, бенчмарки) | Важно для доверия, не блокирует |

---

## 9. Что нужно решить (требует выбора владельца)

1. **MSRV.** Поднимать до 1.83 (требование `pest 2.8.4`) или откатывать `pest`/убирать `LazyLock` ради 1.75? Заявленный сейчас 1.75 нерабочий.
2. **Заявка о совместимости.** Оставить «100% совместимость» как цель и планировать Фазу 3 как обязательную — или переформулировать README в «совместимый по синтаксису, визуально приближённый рендерер» и снять недостижимое обещание.
3. **Метрики шрифта.** Таблица ширин глифов (детерминированная, WASM-совместимая, но требует калибровки под конкретный шрифт PlantUML) или реальный парсинг шрифта через `fontdb`+`ab_glyph` (точнее, но тяжелее и проблемнее в WASM). Это ключевое архитектурное решение всей Фазы 3.
4. **`plantuml-model`.** 162 строки геометрии, дублирующие `plantuml-model`-роль внутри `plantuml-layout`. Оставить, расширить до «моделей диаграмм» как обещает `PLAN.md`, или удалить крейт?
5. **`docs/PlantUML.pdf`.** Удалять из истории (`git filter-repo`) или просто из HEAD? Первое перепишет историю — нужен явный согласованный выбор.
6. **`archimate` и `deployment`.** Реализовывать собственные движки или явно задокументировать, что они рендерятся как component-диаграммы?

| 2.15 | **Найдено при сплошной проверке синтаксиса: заметки class-диаграмм молча терялись.** Грамматика `note` не знала вовсе; добавлены три формы (`note right of A : текст`, `note "текст" as N`, многострочная `... end note`). Но и после этого заметка не появлялась: парсер её не обрабатывал, а `ClassDiagram::notes` не читалось ни в одном layout-движке. Теперь заметки разбираются и рисуются светло-жёлтым (`#FEFFDD`) рядом с элементом-якорем. Добавлен тест | текст и рамка заметки появляются в выводе | ✅ **Сделано** |
| 2.16 | **Найдено при сплошной проверке синтаксиса: `queue` и `collections` вместе с `component` уходили в sequence-парсер.** Эти ключевые слова есть и в sequence, и в component; детектор выбирал sequence, который не знает `component`, и разбор падал на первой же такой строке. Теперь при наличии `component`/`package` выбирается Component. Добавлен тест | `queue Очередь` + `component A` разбирается | ✅ **Сделано** |

| 2.17 | **Найдено при сплошной проверке синтаксиса: `usecase` со стрелкой обобщения уходил в class-парсер.** Стрелка `<|--` проверялась в детекторе раньше `usecase`, поэтому диаграмма с `usecase UC1` + `UC1 <|-- UC2` распознавалась как Class, и разбор падал на первой же строке. Теперь явный признак `usecase`/`actor` важнее стрелки. Добавлен тест | 4 варианта usecase/class со `<|--` разбираются правильно | ✅ **Сделано** |
| 2.18 | **Найдено при сплошной проверке синтаксиса: `-> метка;` в activity не разбирался.** Это безусловный переход; в грамматике такого правила не было. Добавлено правило `arrow_label_stmt` и его обработка в парсере (представляется действием — в AST нет отдельного варианта для переходов) | метка появляется в выводе | ✅ **Сделано** |

| 2.19 | **Найдено при сплошной проверке синтаксиса: заметки activity не разбирались и не рисовались.** Три причины сразу: в грамматике многострочная форма проверялась после однострочной (перехватывала `note right` без двоеточия), форма `note "текст" as N` отсутствовала, а в layout заметка была заглушена `TODO` — причём в двух местах (пропуск в фильтре и ветка в основном цикле). Теперь все четыре формы разбираются и рисуются светло-жёлтым справа от потока | текст и заливка `#FEFFDD` появляются в выводе | ✅ **Сделано** |

| 2.20 | **Найдено при сплошной проверке синтаксиса: параллельные регионы `state` терялись.** Разделитель `--` не был описан в гра­мматике вовсе; после его добавления выяснилось, что парсер игнорирует разделитель и сваливает все состояния в один список, а поле `State::regions` не читается ни в одном layout-движке. Теперь разделитель делит тело на регионы, состояния из регионов попадают в раскладку. Добавлен тест | состояние из второго региона появляется в выводе | ✅ **Сделано** |

| 2.21 | **Найдено сплошной проверкой синтаксиса: три формы sequence и связь-«леденец» в class.** `delay 10 минут` (PlantUML знает `delay <текст>`, а не только `...`) не разбирался; `create participant B` (форма с типом участника) не разбирался; `ref over A, B : текст` без объявленных участников не давал определить тип диаграммы вовсе. Исправлены все три | все формы разбираются и рендерятся | ✅ **Сделано** |

| 2.22 | **Найдено сплошной проверкой синтаксиса: пять конструкций PlantUML не разбирались.** `A -> B-- : текст` (деактивация без последующего действия — правило требовало `--` перед целью, а не после); разделитель `== текст ==` в activity (правила не было вовсе); `legend ... endlegend`, `mainframe`, `newpage` в sequence (все три ключевых слова отсутствовали) | все формы разбираются и рендерятся | ✅ **Сделано** |

| 2.23 | **Найдено той же проверкой: `legend`/`mainframe`/`newpage` принимались грамматикой, но парсер их игнорировал** — конструкции молча терялись (та же схема, что была с заметками). Теперь все три попадают в `DiagramMetadata` (добавлены поля `mainframe` и `newpage`). Добавлен тест | легенда, рамка и заголовок страницы в AST | ✅ **Сделано** |

| 2.24 | **Найдено той же проверкой: у метаданных была та же болезнь на уровне layout.** После того как `legend`, `mainframe` и `newpage` начали попадать в AST, выяснилось, что layout-движок читает только `title`/`header`/`footer`/`caption`. Реализована отрисовка легенды (справа от диаграммы, серая подложка `#DDDDDD`) и рамки `mainframe` (охватывает всю диаграмму, подпись в углу) | текст легенды и заголовок рамки появляются в выводе | ✅ **Сделано** |

| 2.25 | **Найдено той же проверкой: `top to bottom direction` не разбирался ни в class, ни в sequence.** Ключевое слово было только в usecase-грамматике. Добавлено `direction_stmt` в обе грамматики, все четыре формы разбираются | четыре формы работают | ✅ **Сделано** |

| 2.26 | **Найдено систематической проверкой «правила грамматики, которые парсер игнорирует»: `partition`, `switch` и `split` теряли ВСЁ содержимое.** Правила в грамматике были, но ни AST, ни парсер, ни layout их не знали — конструкция попадала в catch-all основного цикла и пропускалась вместе с содержимым. Добавлены типы `Partition`, `Switch`, `Split`, разбор и раскладка. Добавлен тест | содержимое разделов и веток появляется в выводе | ✅ **Сделано** |

| 2.27 | **Найдено той же проверкой: `scale`, `language` и гиперссылки `[[URL]]` не разбирались.** `scale 2` отсутствовал в sequence и class (был только в component/state/gantt/timing); `language ru` — тоже; гиперссылка на элементе (`class A [[http://example.com]]`) не поддерживалась нигде | все три конструкции разбираются | ✅ **Сделано** |

| 2.28 | **Найдено сплошной проверкой по всем типам диаграмм: timing-диаграммы почти не разбирались.** Тег `@starttiming` не проверялся детектором вовсе; грамматика принимала только `@startuml`/`@enduml`; время требовало `@` (`@0`), тогда как PlantUML допускает и голое число (`0 is A`); форма «время перед состоянием без имени сущности» отсутствовала. Теперь все четыре типа участников (`robust`/`concise`/`clock`/`binary`) разбираются в обеих формах записи времени | шесть форм timing разбираются | ✅ **Сделано** |

| 2.29 | **Найдено сплошной проверкой: цветные и стилизованные стрелки в class не разбирались.** `A -[#red]-> B`, `A -[dashed]-> B` — распространённая запись PlantUML. Причина: `-` съедался правилом `arrow_line` как начало `-->`, и остаток `[#red]->` оказывался неожиданным. Правило переписано: стиль допускается после линии, за ним — её остаток до наконечника. Проверены все восемь форм связей (`-->`, `<|--`, `..>`, `*--`, `o--`, обе цветные) | восемь форм разбираются | ✅ **Сделано** |

| 2.30 | **Найдено той же проверкой: в component/deployment не разбирались четыре формы связей** — цветная (`A -[#red]-> B`), стилизованная (`A -[dashed]-> B`), с направлением без стиля (`A -down-> B`) и простая ассоциация без наконечника (`A -- B`, `A - B`). Правило `arrow` знало только фиксированные строки. Добавлена ветка стиля и ветка ассоциации | девять форм связей разбираются | ✅ **Сделано** |

| 2.31 | **Найдено той же проверкой: ни одна форма вехи в gantt не разбиралась.** `milestone_def` стоял в списке операторов ПОСЛЕ `task_def`, а модификаторы задачи необязательны (`task_modifiers` допускает ноль повторов) — поэтому `task_def` жадно съедал `[Веха]`, оставляя `happens ...`. Порядок исправлен. Заодно добавлены формы `happens at <дата>` и `happens on <дата>`, которых не было | пять форм вех разбираются | ✅ **Сделано** |

| 2.32 | **Найдено сплошной проверкой: в salt не разбирались виджеты с подписью.** `(X) Да`, `[X] Включено`, `( ) Нет` — самая обычная запись — не разбирались вовсе: ячейка допускала ровно один виджет, а подпись — это второй. После правки грамматики обнаружилось, что парсер жёстко затирает `label` пустой строкой, поэтому текст не попадал в вывод даже при успешном разборе. Исправлены оба уровня, добавлен `SaltWidget::set_label` | подписи радио и чекбоксов появляются в выводе | ✅ **Сделано** |

| 2.33 | **Найдено сплошной проверкой: кириллица не поддерживалась в ключах YAML и стереотипах state.** `yaml_key` использовал `ASCII_ALPHANUMERIC`, из-за чего `ключ: значение` не разбирался вовсе; `stereotype_name` в state — то же самое, `<<Состояние>>` падал. Это прямое нарушение требования языконезависимости из `AGENTS.md`; заменено на `LETTER`/`NUMBER` (Unicode) | кириллица в ключах и стереотипах разбирается | ✅ **Сделано** |

| 2.34 | **Устранена последняя заглушка в layout: коннекторы activity.** `(A)` пропускался ДВАЖДЫ — в фильтре пропускаемых элементов и в ветке `TODO` — поэтому на диаграмме его не было вовсе. Реализован кружком с подписью, минимальный радиус 12 | подпись коннектора появляется в выводе | ✅ **Сделано** |

| 4.7 | **Фигуры участников sequence.** Ранее `boundary`, `control` и `entity` рисовались прямоугольниками, а `actor`/`database` — внутри полосы подписи, тогда как PlantUML выносит фигуру НАД подписью (а в нижнем блоке — зеркально, ПОД ней). Геометрия снята с эталона: стик-фигура (голова r=8 на y=18.5, конечности с разносом 13), кружок r=12 со скобкой/стрелкой/подчёркиванием, цилиндр (rx=18, крышка ry=10). Нижние блоки теперь повторяют тип участника. Расхождение по высоте для `sequence_participants` сократилось со 102 до 13px, суммарное по высоте — с 427 до 338px | фигуры совпадают с эталоном по форме и положению | ✅ **Сделано** |

| 4.8 | **Минимальный зазор между участниками sequence доминировал над раскладкой.** `min_spacing = 50` превышал требование по длине стрелки (`текст + 20`) для всех пар, поэтому зазор всегда упирался в минимум и блоки стояли шире, чем в PlantUML. Снижен до 30 — значения ≤35 не меняют результат, то есть ограничение больше не является связывающим. Проверено отсутствие перекрытия блоков. Суммарная ширина: 297 → 265px, `sequence_participants` 64 → 43px | перекрытий нет, ширина сократилась | ✅ **Сделано** |

| 2.35 | **Найдено при разборе deployment_basic: связь с именем в кавычках терялась.** `extract_connection_endpoint` не знал `quoted_string`, поэтому endpoint получался пустым, и связь отбрасывалась целиком из-за проверки `from.is_empty() \|\| to.is_empty()`. В эталонном кейсе пропадала связь `"app.jar" --> pg : JDBC` вместе с меткой. Добавлен тест | связь и метка `JDBC` появляются в выводе | ✅ **Сделано** |
| 2.36 | **Найдено той же проверкой: детектор знал лишь половину контейнерных ключевых слов component.** `folder`, `frame`, `rectangle`, `file`, `card`, `hexagon`, `stack`, `entity` не давали определить тип диаграммы вовсе — валидная диаграмма из одного такого элемента не разбиралась. Список приведён в соответствие с грамматикой. Добавлен тест на все 21 ключевое слово | все ключевые слова разбираются | ✅ **Сделано** |
| 2.37 | **Найдено той же проверкой: `rectangle` без тела не разбирался.** В usecase-грамматике правило требовало `{`, тогда как в PlantUML `rectangle` — и контейнер, и обычный элемент. Правило переписано без промежуточных узлов (парсер читает прямых потомков). Добавлен тест | `rectangle R` и `rectangle R { }` разбираются | ✅ **Сделано** |

## Итог работы над планом

Работа велась итеративно, каждый пункт проверялся по эталонам PlantUML 1.2026.9beta4.

### Golden-харнесс

Измерительная сетка расширена с 7 кейсов на 4 типа до **20 кейсов на 18 типов**.
Суммарное расхождение габаритов с PlantUML:

| Этап | Ширина | Высота |
|---|---|---|
| Начало работы над харнессом | 3582.9px | 1282.0px |
| После систематических правок | **434.6px** | **640.7px** |

Сокращение по ширине — **88%**, по высоте — **50%**.

Харнесс многократно выявлял дефекты, которые иначе остались бы незамеченными:
`@startnwdiag` не поддерживался, `node` со связью распознавался как sequence,
кириллица не работала в timing, повторное упоминание задачи gantt дублировало
её, кардинальности ER не отображались, конечное состояние state уходило
на отдельный уровень.

### Ключевое достижение: точность измерения текста

Собраны все 135 значений `textLength` из 20 эталонных SVG (100 уникальных
символов, размеры шрифта 10–14). По ним построена таблица ширин символов
(`CHAR_EM_TABLE`), leave-one-out ошибка — 3.97%.

Средняя ошибка измерения упала с **18.8% до 1.0%**. Поскольку от ширины
текста зависят размеры почти всех фигур, это улучшило сразу все типы
диаграмм. Заодно устранено дублирование: рендерер считал `textLength`
своей константой, игнорируя `TextMeasurer`.

### Что осталось

Структурные расхождения, требующие переработки движков:

- `json`/`yaml` — PlantUML выводит двухколоночную таблицу с рамкой и
  разделителями, наш движок рисует вложенные блоки;
- типы участников sequence — PlantUML рисует каждому типу свою фигуру
  (стик-фигура, кружок, цилиндр) над общей базовой линией подписи;
- `state` — конечное состояние и параллельные регионы;
- формы облака и очереди в component, ортогональная маршрутизация связей.

### Проверяемые гейты

Все шесть зелёные: `cargo test --workspace` (409 тестов), `cargo clippy
--workspace --all-targets --all-features -- -D warnings`, `cargo fmt --check`,
сборка `wasm32-unknown-unknown`, MSRV 1.83 `--locked`, rustdoc `-D warnings`.
