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

| 2.38 | **Найдено той же проверкой: `interface` уводил component-диаграмму в class-парсер, а `device` не принимал тело.** `component A` + `interface I` распознавалось как Class (проверка `interface` стоит раньше), и разбор падал на слове `component`. Отдельно `device D { ... }` не разбирался: `device` не входил в список контейнерных ключевых слов, а ветка «не-контейнера» не требовала отсутствия `{` | пять форм разбираются | ✅ **Сделано** |

| 2.39 | **Найдено сплошной проверкой ключевых слов: `enum` и `annotation` не давали определить тип диаграммы.** Детектор class проверял только `class`/`interface`/`abstract class`, тогда как грамматика знает ещё `enum` и `annotation` — такая диаграмма не разбиралась вовсе. Список приведён в соответствие с грамматикой. Заодно систематически проверены все 61 ключевое слово из грамматик: 28 самостоятельных элементов разбираются, остальные — модификаторы и аргументы `hide`/`show` | `enum`/`annotation` разбираются | ✅ **Сделано** |

| 5.13 | **Зазор между контейнерами component/deployment был общим с зазором между элементами.** Один параметр `vertical_spacing` обслуживал и шаг компонентов, и расстояние между узлами, а эталон требует разного: 77 для компонентов (`component_basic`) и 49 для узлов (`deployment_basic`) — разница 28px на каждый зазор. Введён отдельный `package_vertical_spacing`. Расхождение по высоте: `deployment_basic` 26 → 2px, суммарное 338 → 314px | оба кейса совпадают с эталоном по высоте | ✅ **Сделано** |

| 2.40 | **Найдено сплошной проверкой: официальная форма `@startnwdiag` не разбиралась.** Грамматика требовала обёртку `nwdiag { ... }`, тогда как в документации PlantUML содержимое при этом теге идёт сразу. Принимаются обе формы. Добавлен тест | обе формы разбираются | ✅ **Сделано** |
| 2.41 | ~~`[T1] is closed` в gantt не разбирался~~ — **запись неверна, откатано.** Я предположил, что такая форма существует, потому что в грамматике есть `closed_stmt` для выходных (`saturday is closed`), и добавил поддержку `is closed`/`is open`. Проверка на plantuml.com дала HTTP 400 на обе формы: **такого синтаксиса в PlantUML нет**. Поддержка откачена, тест удалён. Поле `GanttTask::is_active` остаётся мёртвым — это отдельная задача | синтаксис не существует | ❌ **Откатано** |

| 2.42 | **Проверка ранее добавленных конструкций на plantuml.com выявила ещё две выдуманные.** `delay <текст>` без двоеточия — сервер отвечает 400, правильная форма `delay: <текст>` (с двоеточием); форма исправлена, тест закрепляет обе допустимые и обе недопустимые записи. `language <код>` валиден ТОЛЬКО в gantt — добавление его в sequence и class откатано. Остальные 14 добавленных конструкций (заметки class/activity, `partition`/`switch`/`split`, `usecase <\|--`, `-> метка`, `create participant`, `ref over`, `legend`, `mainframe`, `newpage`, `direction`, `scale`, `[[URL]]`) подтверждены | соответствие PlantUML | ✅ **Сделано** |

| 2.43 | **Продолжение проверки на plantuml.com выявило ещё три несуществующие конструкции, добавленные мной ранее.** (1) Тег `@starttiming` — сервер отвечает 400 на любую форму; timing-диаграммы используют `@startuml`. Поддержка тега и его детекция откатаны. (2) `device` — 400 во всех формах, включая `device D1` без тела; ключевое слово убрано из контейнерных. `agent` существует, но тела не принимает (`agent A { }` → 400). (3) `component` и `artifact`, наоборот, тело ПРИНИМАЮТ — у нас `component A { }` не разбирался; исправлено. Добавлен тест | соответствие PlantUML | ✅ **Сделано** |

| 2.44 | **Продолжение проверки журнала на plantuml.com: разделитель activity и даты вех.** (1) Разделитель activity существует только из ЧЁРТОЧЕК (`--`, `----`); форма с `=` даёт 400 — `==` это разделитель sequence. Правило сужено до чёрточек. (2) Веха в gantt привязывается ТОЛЬКО к задаче (`happens at [T1]'s end`, `happens after [T1]'s end`); формы с датой (`happens 2024-01-01`, `happens at <дата>`, `happens on <дата>`) дают 400 — убраны. `project starts <дата>` при этом валиден и оставлен | соответствие PlantUML | ✅ **Сделано** |
| 2.45 | **Проверены оставшиеся записи журнала — подтверждены.** Параллельные регионы `state` (`--`), коннекторы activity `(A)`, `enum`/`annotation`, деактивация `A -> B--`, время без имени сущности (`0 is 1`), `mainframe`, `newpage`, `direction`, `scale`, `[[URL]]`, цветные стрелки, salt-виджеты с подписью, кириллица в YAML и стереотипах | 9 из 9 проверок пройдено | ✅ **Сделано** |

| 2.46 | **Списки ключевых слов component/deployment приведены к официальной документации.** Сверил с разделами «Declaring element» и «Nestable elements» (plantuml.com/en/deployment-diagram): `device` в списке НЕТ — ключевое слово удалено из грамматики; отсутствовали `action`, `circle`, `label`, `person`, `process` — добавлены; `collections` ошибочно числился контейнерным — убран. `package_keyword` был ОТДЕЛЬНЫМ списком и разъезжался с `container_keyword` — теперь ссылается на него. Детекция `queue`/`collections` расширена на все контейнерные признаки. Два теста переписаны под официальный список | 28 объявляемых элементов разбираются, 17 контейнерных принимают тело, 9 неконтейнерных — нет | ✅ **Сделано** |
| 2.47 | **Проверка `binary`/`clock` в timing: подозрение не подтвердилось.** Ранее я счёл их несуществующими по HTTP 400, но ошибка была в МОЁМ тесте: для `binary` нужен маркер `@0` и состояния вида `B is high`, а не `0 is 1`. Документированный пример с `clock`/`binary`/`concise`/`robust` отдаётся сервером нормально — типы участников валидны и оставлены без изменений | ложное подозрение снято | ✅ **Проверено** |

| 2.48 | **14 из 15 задокументированных типов связей не разбирались.** Сверил с разделом «Linking or arrow»: `--*`, `--o`, `--+`, `--#`, `-->>`, `--0`, `--^`, `--(0`, `-0-`, `-0)-`, `-(0-`, `-(0)-`, `~~`, `==` — ни одна не работала. Причины: украшенные формы проверялись ПОСЛЕ обычных (поэтому `-->>` разбиралось как `-->` с остатком `>`), а `arrow_direction` знал только `up`/`down`/`left`/`right` без сокращений `ri`/`le`/`do`, которые встречаются в документированных примерах (`-ri(0)->`). Порядок исправлен, сокращения добавлены (длинные варианты первыми, иначе `do` перехватывает `down`). Добавлен тест | 15 типов связей и 11 направлений разбираются | ✅ **Сделано** |

| 2.49 | **Короткая форма актёра `:Alice:` не разбиралась.** Документация sequence-диаграмм описывает её наравне с `actor Alice`. Грамматика её не знала вовсе; после добавления правила выяснилось, что парсер его не обрабатывает и участник не создаётся. Добавлена и в объявления, и в `participant_ref` (чтобы работали сообщения `:Alice: -> :Bob:`), с защитой от конфликта с message через `!(ws* ~ arrow)`. Добавлен тест | 4 формы работают, включая сообщения и `box` | ✅ **Сделано** |
| 2.50 | **`autonumber 1.1` / `autonumber inc` не давали определить тип диаграммы.** Без других признаков (`participant`, стрелок) такая диаграмма не опознавалась. `autonumber` добавлен в детекцию sequence. Добавлен тест | диаграмма опознаётся | ✅ **Сделано** |

| 2.51 | **Три конструкции state не разбирались, а четвёртая теряла данные.** (1) `state A #FF0000` — правила не было вовсе; после добавления выяснилось, что парсер его не читает и цвет молча теряется. (2) Многострочная заметка проверялась ПОСЛЕ однострочной, у которой двоеточие необязательно — `note right of A` без двоеточия перехватывалось ею (та же ошибка, что была в class и activity). (3) `left to right direction` в state отсутствовал. Добавлен тест | 7 форм работают, цвет доходит до AST | ✅ **Сделано** |
| 2.52 | **Сверка activity-грамматики с документацией: 20 из 20 конструкций работают** — действия (включая цветные), все ветвления, циклы, fork/split/switch/partition, swimlane, detach/kill, backward, goto/label, коннекторы, заметки, стрелки. Пробелов не найдено | 20/20 | ✅ **Проверено** |

| 2.53 | **Алиасы в class-диаграммах не поддерживались.** `class "Длинное имя" as short` — обычная запись PlantUML — не разбиралась: правила `alias_part` в грамматике class не было вовсе, хотя в component и sequence оно есть. После добавления правила выяснилось, что парсер его не читает — алиас молча терялся, и связи по нему не находились. Добавлено в четыре объявления (class/interface/enum/annotation) и в парсер. Добавлен тест | 4 формы работают, связи по алиасу находятся | ✅ **Сделано** |
| 2.54 | **Сверка class-грамматики с документацией: 21 из 22 конструкций работают** (после правки алиасов) — тела классов с видимостью и модификаторами `{static}`/`{abstract}`, generic, extends/implements, стереотипы, цвет, все виды связей с кардинальностями и метками, package, namespace, together, skinparam | 22/22 | ✅ **Проверено** |

| 2.55 | **Атрибуты сети `color` и `description` не разбирались.** `server_in_network` (`identifier ~ server_attributes?`) проверялся РАНЬШЕ атрибутов и перехватывал `color`/`description` как имя сервера — строка `color = "#FFAAAA"` оказывалась неожиданной, и вся диаграмма падала. Порядок исправлен. Добавлен тест | оба атрибута разбираются и доходят до AST | ✅ **Сделано** |
| 2.56 | **Сверка salt, network, json/yaml, timing с документацией: 20 из 22 конструкций работают.** Проверены вкладки, меню, дерево, скролл, группа `{^"..."}`, кнопки, поля, выпадающие списки, все разделители; две сети, группы, цвета; вложенный JSON, `#highlight`, скалярный и вложенный YAML, списки; четыре типа участников timing | 20/22 после правок | ✅ **Проверено** |
| 2.57 | **Найдена потеря данных: `left to right direction` принимается грамматикой в четырёх типах диаграмм, но НЕ сохраняется нигде** — поля `direction` нет в `DiagramMetadata`, парсеры его не пишут, layout не читает. Тип `Direction` в AST объявлен, но не используется. Это тот же паттерн «разбирается, но игнорируется», что был у цвета state и алиасов class | зафиксировано, не исправлено | ⏳ **Открыто** |

| 2.58 | **Направление раскладки сохраняется в AST.** Ранее `left to right direction` принималось грамматикой в четырёх типах диаграмм, но НИ ОДИН парсер его не обрабатывал — данные терялись полностью, а тип `Direction` в AST не использовался. Добавлено поле `DiagramMetadata::direction` и разбор в class, sequence, state (usecase уже сохранял в своём поле). Порядок слов учитывается: `top to bottom` и `bottom to top` содержат одни слова. Формы `bottom to top` и `right to left` PlantUML отвергает (400) — наш парсер тоже. Добавлен тест | обе валидные формы сохраняются | ⚠️ **Частично** — layout направление пока не читает |

| 5.14 | **Узлы deployment рисуются объёмными, как в PlantUML.** Раньше `ElementType::Group` (создаётся только для узлов и пакетов component) рисовался плоским прямоугольником с полосой заголовка. Эталон рисует трёхмерный бокс: основной прямоугольник плюс скошенный верхний правый угол со смещением 10px. Реализован полигон из шести точек и три внутренних ребра; структура совпадает с эталоном точка в точку. Заголовок центрируется по основному прямоугольнику | форма совпадает с эталоном | ⚠️ **Частично** — габариты не изменились (258x274 против 295x276), расхождение по ширине 37px остаётся |

| 2.59 | **Критический дефект: паника на кириллице перед вызовом процедуры.** `find_function_calls` собирал `Vec<char>` и возвращал индексы в СИМВОЛАХ, а `process_function_calls` резал строку по ним как по БАЙТОВЫМ. На ASCII это совпадает, на кириллице — нет: срез попадал в середину символа и паниковал «byte index N is not a char boundary». Индексы переведены в байтовые. Добавлен тест | паника устранена | ✅ **Сделано** |
| 2.60 | **Пункт 2.2 был закрыт не полностью: реестр stdlib недостижим из публичного API.** `Preprocessor::new` использовал `NoopFileResolver`, поэтому `!include <C4/C4_Context>` через `render` и `parse_diagram` падал с «!include не поддерживается» — весь реестр из 48 включений был мёртвым кодом. Прежний тест проходил, потому что конструировал препроцессор С резолвером. Добавлен `StdlibResolver` (только встроенные данные, без файловой системы — безопасен для WASM) и сделан типом по умолчанию. Добавлен тест | включения работают через публичный API | ✅ **Сделано** |
| 2.61 | **C4 всё ещё не работает сквозным путём.** После достижимости stdlib выяснилось: макросы объявлены через `!define` и подставляют параметр ВНУТРЬ кавычек (`rectangle "==e_label..."`) — при вызове `Person(a, "Имя")` получается `"=="Имя""`. Попытка снимать кавычки в макросах сломала существующий тест `test_macro_args_respect_quotes`, который фиксирует их сохранение как осознанное решение, — откатано. Вероятная причина в самом содержимом C4 в нашей библиотеке (лишние кавычки вокруг параметра), а не в препроцессоре. Поддержка `!unquoted` добавлена (нужна для актуального C4) | зафиксировано | ⏳ **Открыто** |

| 2.62 | **C4 заработал сквозным путём — вопрос 2.61 закрыт.** Определяющая проверка на plantuml.com: `!define M(a) class "PRE a POST"` даёт `PRE Имя POST` и при `M("Имя")`, и при `M(Имя)` — то есть PlantUML кавычки в аргументах макроса СНИМАЕТ. Наш тест `test_macro_args_respect_quotes` фиксировал обратное; он обновлён под проверенное поведение. Дополнительно найдены и исправлены три пробела: алиас и стереотип допустимы в ЛЮБОМ порядке (`"X" <<person>> as a` — форма, которую порождает C4); двунаправленные стрелки `<-->`/`<->` (порядок альтернатив: `<-` перехватывал начало); `<-down->` — направленная двунаправленная. В макросах `*_Boundary` убрана лишняя `{`: PlantUML берёт её от вызывающего. Добавлены 6 интеграционных тестов | C4_Context/Container/Component, направления, BiRel, границы работают | ✅ **Сделано** |

| 6.1a | **Определения спрайтов не поддерживались — 30 из 39 включений реестра не разбирались.** Работало 9. Содержимое библиотек иконок (`logos`, `office`, `tupadr3`) состоит из `sprite $имя [ШxВ/цветов] { ...hex... }`, а такого правила в грамматике не было вовсе. Правило добавлено в class и component. После правки разбираются **все 39** включений. Добавлены 2 интеграционных теста | 39/39 | ✅ **Сделано** |

| 3.7 | **Метрика текста не учитывала ЖИРНОСТЬ.** Заголовки узлов, имена классов и заголовки состояний PlantUML рисует полужирным, а он шире обычного. По 15 жирным подписям эталонов среднее отношение составило 1.0828, а средняя ошибка на них — 7.60% против 2.40% на обычных. Добавлен `TextMeasurer::width_bold` и применён к заголовкам узлов и именам классов. `deployment_basic`: 37 → 32px | учтена жирность | ✅ **Сделано** |
| 3.8 | **В таблицу ширин символов добавлена кириллица (66 символов), измеренная напрямую.** Раньше она оценивалась по классу (`0.82` для заглавных против реальных ~0.69). Проверка: сумма по буквам совпадает с прямым замером PlantUML с точностью 0.3%; «Сервер приложений» −4.9% → −0.3%, «Тестирование» −0.0% | точность измерения выросла | ✅ **Сделано** |
| 3.9 | **Найден скрытый дефект gantt: неверный размер шрифта подписи задачи.** В эталоне `font-size=11`, у нас стояло 10. Ширина диаграммы сходилась лишь потому, что завышенное измерение текста случайно попадало в нужную ветку выноса подписи за полосу. После исправления измерения это вскрылось. `date_font_size` 10 → 11: `gantt_basic` 28 → 22px | размер шрифта исправлен | ✅ **Сделано** |
| 3.10 | **Побочный эффект уточнения измерения: `sequence_participants` 43 → 49px.** Часть прежнего совпадения держалась на завышенной ширине кириллических подписей сообщений. Итог по сумме положительный: общая ширина 265 → 262px, deployment 37 → 32, gantt 28 → 22. Baseline обновлён осознанно | регрессия зафиксирована | ⚠️ **Частично** |

| 4.9 | **Найдена двойная добавка отступов в длине стрелки sequence.** `message_label_width` уже добавляет 16px, а `required_length` прибавлял ещё 20 — итого на 12px больше эталона на каждую пару. Эталон: «сообщение» 76.261 + 16 + **8** = 100.261, ровно шаг участников в `sequence_participants`. Исправлено на 8. `sequence_participants` 49 → **11px**, общая ширина 262 → **230px** (лучший результат за всю работу) | 49 → 11px | ✅ **Сделано** |
| 4.10 | **Побочный эффект той же правки: `sequence_fragments` 4 → 8px, `sequence_notes` 3 → 7px.** Их прежнее совпадение держалось на раздутых шагах. Суммарный эффект резко положительный (−32px по ширине), baseline обновлён осознанно | зафиксировано | ⚠️ **Частично** |

| 5.15 | **Поле вокруг state-диаграмм было занижено.** Эталон `state_simple`: контент начинается с 16.3 при viewBox 226, то есть поля ~16-25. У нас `margin = 6` давал контент с 1 и viewBox 207. Прямой счёт по эталону: 226 − 185 (контент) = 41, то есть ~20 на сторону. `margin` 6 → 15. Результат: `state_simple` dw 18 → **0px**, dh 14 → 4px. Общая ширина 230 → **212px**, высота 314 → **304px** | 18 → 0px | ✅ **Сделано** |

| 5.16 | **Направление раскладки наследования было ИНВЕРТИРОВАНО.** Код намеренно разворачивал рёбра `Inheritance`/`Realization`, чтобы родитель оказался на слое 0 (сверху). Проверено на plantuml.com: для `Child --|> Parent` PlantUML ставит РЕБЁНКА сверху (y=27.85), родителя снизу (y=135.85). Подтверждается эталоном `class_inheritance`: Dog и Cat на y=7, Animal на y=131.29. Разворот убран | структура совпала с эталоном | ✅ **Сделано** |
| 5.17 | **Маркер видимости включался в измеряемую ширину члена класса.** PlantUML рисует `+`/`-`/`#` отдельной ИКОНКОЙ, а место под неё уже учтено в `CLASS_CONTENT_EXTRA = 26`. Проверка по эталону: «bark()» 42.253 + 26 = 68.253 — ровно ширина бокса Dog. С «+» в тексте бокс выходил на 7.4px шире. Маркер исключён из измерения. `class_inheritance` dw 17 → **3px**; общая ширина 212 → **198px** | 17 → 3px | ✅ **Сделано** |

| 3.11 | **Ширина дня в gantt была 15.7 вместо 16.0.** Прямой счёт по эталону: сетка идёт от x=136.069 до 696.069, то есть 560px; проект длится 35 дней; 560 / 35 = **16.0**. У нас стояло 15.7, что давало 549.5. `gantt_basic` dw 22 → **12px**; общая ширина 198 → **188px** | 22 → 12px | ✅ **Сделано** |

| 5.18 | **Поле рендерера было 5px вместо 7px.** Измерено по эталонам: контент class- и component-диаграмм начинается с x=7.0 при viewBox с началом в 0. Симметричное поле 5 оставляло диаграмму на 4px уже эталона. `margin` рендерера 5 → 7 | эталонные поля 7 совпали | ✅ **Сделано** |
| 5.19 | **`class_inheritance` совпал с эталоном ТОЧНО: 204x210, dw=0, dh=0.** Суммарное поле у class должно быть 21.3 (измерено: слева 7.00, справа 14.28, сверху 7.00, снизу 14.41). Оно складывается из поля рендерера (7.0) и поля движка, поэтому `margin` class 7 → 3.5. Распределение слева/справа при этом НЕ совпадает с эталонным (у нас 7/14.3 суммарно, но иначе разложено) | dw=0, dh=0 | ⚠️ **Частично** |
| 5.20 | **`state_simple` dw 4 → 0.** Под новое поле рендерера поле state-движка скорректировано 15 → 13 | dw=0 | ✅ **Сделано** |

| 4.11 | **Отступы внутри sequence-фрагмента были завышены.** Измерено по эталону `sequence_fragments`: расстояние между центрами сообщений 47.94, от начала фрагмента до первого сообщения 44.27, от последнего сообщения до низа 12. У нас было 72.13 / 47.0 / 45.14. Скорректированы: разделитель секций 43 → 19, отступ заголовка 26 → 23, нижний отступ `fragment_padding + 5` → `fragment_padding − 5`. `sequence_fragments` dh 50 → **13px** | dh 50 → 13 | ✅ **Сделано** |
| 4.12 | **Общая высота 283 → 246px.** Остаток расхождения фрагмента — из-за того, что после ПОСЛЕДНЕГО сообщения секции мы всё ещё продвигаем курсор на полную высоту сообщения (~29px), тогда как эталон добавляет 12. Это структурная правка (нужно считать низ по фактическому низу потомков), не константа | зафиксировано | ⏳ **Открыто** |

| 5.21 | **Вертикальный отступ между usecase был 25px вместо 60px.** Измерено по эталону `usecase_basic`: эллипсы UC1 и UC2 имеют центры 159.426 и 251.584 при полуосях 17.626 и 14.524, то есть зазор между краями 60.01. У нас стояло 25. `vertical_spacing` 25 → 60: dh 32 → **3px** | dh 32 → 3 | ✅ **Сделано** |
| 5.22 | **Поле usecase было 20px вместо 16px.** Под поле рендерера 7 (и эталонное левое поле 6) `margin` 20 → 16: dw 8 → **4px**, dh 3 → 1px. Общая ширина 183 → **179px**, высота 246 → **215px** | dw 8 → 4 | ✅ **Сделано** |

| 5.23 | **Отступ между дорожками timing был 20px вместо 5px.** Измерено по эталону `timing_basic`: подписи участников на y=32.99 и 98.29, шаг 65.3. При `lane_height = 60` отступ равен 5. У нас стояло 20, шаг выходил 80. `lane_spacing` 20 → 5: dh 22 → **8px** | dh 22 → 8 | ✅ **Сделано** |
| 5.24 | **Раскладка WBS устроена иначе, чем у нас.** Эталон ставит детей НЕ в общий ряд по глубине: Task 1.1 и Task 1.2 (дети Phase 1) стоят в одном столбце на разных строках, а Task 2.1 (первый ребёнок Phase 2) — на одной строке с Task 1.1. Шаг строк при этом 73.97 затем 48.97. Наша модель «все узлы одной глубины в один ряд» этому не соответствует. Попытка поправить одним `level_spacing` сделала хуже (dh 30 → 80), изменение откачено | зафиксировано, не исправлено | ⏳ **Открыто** |

| 5.25 | **Минимальная ширина ячейки salt была 52.07 вместо 35.** В эталоне `salt_basic` поле ввода занимает 66.07..101.07 = ровно 35. У нас `min_cell_width` стоял 52.07 (совпадал с шириной кнопки «Отмена» в первом столбце), из-за чего ОБА столбца выходили по 52.07. `min_cell_width` 52.07 → 35: `salt_basic` dw 17 → **1px**; общая ширина 179 → **163px** | dw 17 → 1 | ✅ **Сделано** |

| 1.10 | **ЧЕТЫРЕ ЭТАЛОНА ИЗ ДВАДЦАТИ БЫЛИ ВЫРОЖДЕННЫМИ.** В `er_basic`, `network_nwdiag`, `json_basic`, `object_basic` сервер отдал SVG с `textLength="0"` у всех подписей и заниженными габаритами. `fetch_references.py` по умолчанию пропускает существующие файлы, поэтому они не обновлялись. После `--force` эталоны стали: `er_basic` 85x208 → **127x226**, `network_nwdiag` 117x129 → **273x140**. Это значит, что харнесс сравнивал часть кейсов с мусором, а «улучшения» по ним могли быть мнимыми | эталоны исправлены | ✅ **Сделано** |
| 5.26 | **Минимальная ширина ячейки salt была 52.07 вместо 35.** В эталоне поле ввода занимает 66.07..101.07 = ровно 35. `min_cell_width` 52.07 → 35: `salt_basic` dw 17 → **1px** | dw 17 → 1 | ✅ **Сделано** |
| 5.27 | **Обнаружен скрытый крупный дефект: `network_nwdiag` dw 7 → 149px** после исправления эталона. Правильный эталон 273x140, наш 124x106. Диаграмма nwdiag строится в разы меньше нужного — ранее расхождение маскировалось вырожденным эталоном | зафиксировано | ⏳ **Открыто** |

| 5.28 | **Размер сервера nwdiag был фиксированным 20x30 вместо измеренного.** Эталон: бокс сервера 60.084x33.969, причём 60.084 = ширина текста «web01» 40.084 + 20. У нас стояло 20x30, из-за чего диаграмма выходила в разы меньше. `server_width` 20 → 60.084, `server_height` 30 → 33.969: `network_nwdiag` dw 149 → **29px**, dh 34 → 30px; общая ширина 277 → **157px** | dw 149 → 29 | ⚠️ **Частично** — ширина должна вычисляться по тексту, а не быть константой (см. 5.29) |
| 5.29 | **Ширина сервера nwdiag должна зависеть от текста, а не быть константой.** Формула эталона: ширина текста имени + 20. Сейчас стоит 60.084 (значение для «web01»). Для других имён будет неверно. Требует переработки позиционирования: `server_x_position`, `network_width` и `Rect` используют одно и то же поле | зафиксировано | ⏳ **Открыто** |
| 5.30 | **Геометрия nwdiag устроена иначе.** В эталоне имя сети и адрес стоят СЛЕВА от полосы (текст адреса x=5..81.564, полоса начинается на 86.564), а у нас адрес рисуется иначе и полоса начинается с padding=10. Полоса эталона 180.168 = 2*(60.084+30) | зафиксировано | ⏳ **Открыто** |

| 1.11 | **Добавлена защита от вырожденных эталонов.** (1) `fetch_references.py` теперь распознаёт ответ, где у всех подписей `textLength="0"`, и перезапрашивает его до трёх раз вместо сохранения. (2) В `golden_tests.rs` добавлен тест `golden_references_are_not_degenerate`, который ловит такой эталон в CI. (3) Проведён аудит всех 20 эталонов — вырожденных больше нет | защита на двух уровнях | ✅ **Сделано** |

| 5.31 | **Конечный узел activity был того же размера, что начальный.** В эталоне `activity_basic` начальный круг имеет rx=10, конечный — rx=11. Оба использовали `node_radius`. Добавлена добавка 1px для `Stop` и `End`: dh 16 → **14px** | dh 16 → 14 | ✅ **Сделано** |
| 5.32 | **В activity нет ромба слияния после ветвления.** Эталон рисует ромб 24x24: ветви кончаются на 176.94, ромб занимает 182.94..206.94, следующее действие начинается на 226.94 (отступы 6 и 20). У нас ветви соединяются стрелками без ромба. Добавление ромба дало перелёт: наши боковые стрелки идут 20px вниз, тогда как в эталоне ветви кончаются на одной высоте и соединение горизонтальное. Нужно сперва поправить длину боковых стрелок, потом добавлять ромб | откачено, зафиксировано | ⏳ **Открыто** |

| 5.33 | **Добавлен ромб слияния в activity (закрыт пункт 5.32).** Причина прошлого перелёта найдена: `layout_element` возвращает курсор, УЖЕ продвинутый на `vertical_spacing` под следующую стрелку, поэтому низ ветви оказывался на 20px ниже её содержимого. Для точки слияния теперь берётся фактический низ (`then_end_y - vertical_spacing`), затем ромб 24x24 с отступом 6. Геометрия совпала с эталоном: ветви 143.92..177.92 (эталон 142.97..176.94), ромб 183.92..207.92 (182.94..206.94), действие 227.89..261.89 (226.94..260.91) — расхождение 0.97 это систематический сдвиг. `activity_basic` dh 14 → **4px** | dh 14 → 4 | ✅ **Сделано** |

| 5.34 | **Горизонтальный разнос ветвей activity был 60 вместо 50.** Измерено по эталону: центры действий ветвления на 54.34 и 154.29, то есть 99.95 друг от друга; у нас было 120. `horizontal_spacing` 60 → 50: `activity_basic` dw 15 → **5px**; общая ширина 157 → **147px**. После этого `activity_basic` перестал быть в топе расхождений | dw 15 → 5 | ✅ **Сделано** |

| 5.35 | **Геометрия nwdiag переработана: подписи сети стоят СЛЕВА от полосы.** Измерено по эталону: имя сети (55.959..81.564) и адрес (5..81.564) имеют общий правый край 81.564, полоса начинается на 86.564 — то есть зазор 5. У нас обе подписи рисовались одной строкой НАД полосой, а полоса начиналась с padding. Исправлено: столбец подписей вычисляется как максимум по всем сетям, полоса сдвигается за него. Ширина полосы тоже исправлена: `n*w + (n-1)*spacing + 2*inset` = 180.168 (было 210.168). `padding` 10 → 5 (эталонный левый край адреса). В `NetworkLayoutConfig` добавлен измеритель текста | — | ✅ **Сделано** |
| 5.36 | **`max_width` не учитывал сдвиг полосы** — подписи слева вылезали за пределы диаграммы, потому что ширина считалась как `network_width + padding * 2`. Исправлено на `band_x + network_width - padding`. `network_nwdiag` dw 29 → **7px**; общая ширина 147 → **125px** | dw 29 → 7 | ✅ **Сделано** |
| 5.37 | **Высота nwdiag: dh 30 → 35px.** Эталон резервирует снизу ~34px (серверы кончаются на 91.84 при холсте 140), у нас 5. Причина не выяснена — вероятно, место под следующую сеть. Ширина при этом улучшилась на 22px, так что суммарный эффект положительный | регрессия зафиксирована | ⚠️ **Частично** |

| 5.38 | **Высота nwdiag: найден запас снизу (закрыт пункт 5.37).** Эталон резервирует под последней сетью ещё один `network_spacing`: серверы кончаются на 91.84, холст 140 при полях 7+7. Добавлен этот запас: dh 35 → **0px**. `network_nwdiag` теперь 280x140 против эталонных 273x140 | dh 35 → 0 | ✅ **Сделано** |

| 5.39 | **Ширина сервера nwdiag стала контентной (закрыт пункт 5.29).** Формула эталона: ширина текста имени + 20. Константа 60.084 была верна только для «web01». Метрика при этом чуть ухудшилась (dw 7 → 9), потому что наше измерение текста даёт ±5% на этих строках: «web01» +2.03%, адрес сети +5.26%, «dmz» −6.38%. Модель правильная, точность ограничена измерением | модель исправлена | ✅ **Сделано** |
| 5.40 | **Точность измерения текста на служебных строках nwdiag: ±5%.** «210.0.0.0/24» +5.26%, «dmz» −6.38% при средней погрешности 2.4% по всем эталонным строкам. Это ограничивает точность контентных размеров в nwdiag | зафиксировано | ⏳ **Открыто** |

| 3.12 | **Цифры в PlantUML ТАБЛИЧНЫЕ, а у нас были разной ширины.** Измерено: все десять цифр имеют ровно **0.6362 em**, тогда как в нашей таблице стояли значения от 0.6071 до 0.6776. Плюс `'.'` был 0.3680 против реальных 0.3179, а `'/'` и `':'` отсутствовали вовсе (оценивались по классу). После исправления «210.0.0.0/24» даёт 76.562 против эталонных 76.564 — попадание в 0.003%. Общая ширина 125 → **122px** | точность измерения выросла | ✅ **Сделано** |
| 3.13 | **Добавлены 10 отсутствовавших латинских букв** (f, x, z, G, H, N, V, X, Y, Z), измеренных напрямую. Раньше они оценивались по классу. `network_nwdiag` dw 9 → **5px** | точность выросла | ✅ **Сделано** |

| 5.41 | **Разобраны причины расхождения deployment (dw=28) — две РАЗНЫЕ, не одна.** Измерено на простых случаях, снятых с сервера: (1) **поля**. У узла `node` левое поле 16, правое 25 (у плоского компонента — 7 и 13.8); (2) **смещение узлов**. В эталоне `deployment_basic` узлы стоят на x=16 и x=29, в простом случае со связью — на 16 и 27, то есть узлы смещены друг относительно друга, а у нас выровнены по одному краю. Первое даёт +12.7, второе +15.3. Проверено, что единая добавка +28 к габаритам обнуляет расхождение, но она компенсирует СРАЗУ ОБА дефекта — это подгонка, а не исправление. Изменение откачено | причины разделены | ⏳ **Открыто** |
| 5.42 | **Поля элементов в component-движке симметричны (7/7), а у эталона зависят от типа.** Плоский компонент: 7 слева, 13.8 справа. Объёмный узел deployment: 16 слева, 25 справа. Движок один и тот же, различие определяется типом элемента. Требует разделения полей по типу, а не единой константы | зафиксировано | ⏳ **Открыто** |

| 5.43 | **Поля узлов отделены от полей компонентов (закрыта половина пункта 5.41).** Введён `node_margin = 16` — узлы размещаются с ним, тогда как плоские компоненты остаются на `margin = 7`. Начало координат остаётся на `margin`, правое поле узла получает добавку 18. `component_basic` не изменился, `deployment_basic` dw 28 → **24px**; общая ширина 122 → **118px** | dw 28 → 24 | ✅ **Сделано** |
| 5.44 | **`package_y` резервировал строку под компоненты, даже когда их нет.** Формула `components.len() / num_cols + 1` давала 1 при нуле компонентов, из-за чего диаграмма из узлов сдвигалась вниз на 123px: первый узел стоял на y=130.3 вместо 7. Исправлено на `div_ceil`. На габариты не влияет (bounds считаются по элементам), но убирает пустую полосу сверху | смещение убрано | ✅ **Сделано** |

| 5.45 | **Mindmap считал ширину узла грубой оценкой вместо измерителя.** `символы × font_size × 0.6` давало для «Project» 58.8 при реальных 48.45 — на 21% больше. Ошибка накапливалась по уровням, потому что X каждого следующего уровня отсчитывается от правого края предыдущего. Заменено на `TextMeasurer`. В конфиг добавлен измеритель, `font_size` 13 → 14 (в эталоне 14) | — | ✅ **Сделано** |
| 5.46 | **`mindmap_basic` dw 10 → 6px**, общая ширина 118 → **114px**. Геометрия совпала: боксы Project 10..79.37 (эталон 10..78.45), Planning 129.37..210.62 (128.45..208.76), Timeline 260.62..342.44 (258.76..338.72). Остаточный дрейф ~1.9% накапливается от точности измерения текста | dw 10 → 6 | ✅ **Сделано** |

| 3.14 | **Найдены и убраны ещё две грубые оценки ширины текста.** (1) `usecase`: ширина имени актёра считалась как `символы × 9.0`; для «Пользователь» это 108 при эталонных 103.1 (+4.8%), причём для латиницы ошибка была бы больше. Заменено на измеритель, в конфиг добавлены `text` и `font_size = 14` (проверено: наш вывод рисует font-size 14, как эталон). (2) `svg_renderer`: ширина autonumber считается как `длина × 7.0 + 3.0`. Строки короткие, влияние малое — оставлено с пометкой | модель исправлена | ⚠️ **Частично** |
| 3.15 | **Проверено, что грубых оценок ширины текста в layout больше не осталось.** Поиск по `len() as f64 *`, `chars().count() *`, `* 8.0` и подобным: остались только структурные умножения (число полей на высоту строки), а не оценка текста | проверено | ✅ **Сделано** |

| 5.47 | **Шаг строки задач gantt был 35 вместо 16.80.** Измерено по эталону: полосы задач на y=41.00, 57.80, 74.61 — шаг 16.80 при высоте полосы 12.80. У нас `row_height = 30`, `bar_height = 20`, `row_spacing = 5`, то есть шаг 35. Исправлено на `row_height = bar_height = 12.80`, `row_spacing = 4`. `gantt_basic` dh 26 → **6px**; общая высота 148 → **128px** | dh 26 → 6 | ✅ **Сделано** |
| 5.48 | **Осталось dh=6 у gantt: различие в полях, а не в раскладке.** Эталон: холст 130 при контенте 0..127.55, то есть поле сверху 0 (рамка таблицы у самого края) и 2.45 снизу. У нас контент 5..126.70, а рендерер добавляет по 7 с каждой стороны → 135.7. Плюс в эталоне подписи месяцев продублированы снизу (y=127.55), у нас их нет. Требует разбирательства с полями рендерера для gantt | зафиксировано | ⏳ **Открыто** |

| 5.49 | **Добавлены подписи месяцев в gantt — их не было вовсе.** PlantUML выводит шкалу месяцев ДВАЖДЫ: над таблицей и под ней. В эталоне «January 2024» стоит на y=11.14 и y=127.55. У нас месяц не был подписан нигде — это реальная несовместимость, а не расхождение габаритов. Подпись центрируется по диапазону своих дней (проверено: «January 2024», 31 день, центр 384.07 при диапазоне 136.069..632.07 — совпадает) | подписи появились | ✅ **Сделано** |
| 5.50 | **Честная регрессия метрики: `gantt_basic` dh 6 → 12px.** Содержимое теперь совпадает с эталоном (0..127.55 против нашего −0.86..127.55), но у эталона поля 0 сверху и 2.45 снизу, а рендерер добавляет по 7 с каждой стороны. То есть до правки расхождение маскировалось: контент был на 5.85 КОРОЧЕ эталонного, и это частично компенсировало лишние поля. Общая высота 128 → 134px | регрессия осознанная | ⚠️ **Частично** |
| 5.51 | **Модель полей рендерера не соответствует PlantUML.** Рендерер добавляет 7 со всех сторон, а у эталонов поля разные по типу диаграммы и по осям: gantt 0 слева и 20.8 справа, 0 сверху и 2.45 снизу; class 7 слева и 14.28 справа. Отрицательное верхнее поле выразить нельзя, поэтому `LayoutResult` должен нести собственное поле, а рендерер — не добавлять своё. Это разблокирует и ширину gantt (dw=8), и высоту | зафиксировано | ⏳ **Открыто** |

| 5.52 | **Реализована модель полей ПО ТИПУ диаграммы (закрыт пункт 5.51).** Рендерер добавлял фиксированные 7 со всех сторон и не мог выразить отрицательное поле. Теперь поля задаются четвёркой `(слева, сверху, справа, снизу)` в зависимости от `RenderOptions::diagram_type` — того же имени, что PlantUML пишет в `data-diagram-type`. Для GANTT измерены `(0, 0, 20.8, 2.45)`. `gantt_basic` dw 8 → **1**, dh 12 → **1**; общая 114x134 → **107x123**. Регрессий нет: `class_inheritance` остался ровно 204x210 | gantt 8/12 → 1/1 | ✅ **Сделано** |
| 5.53 | **Измерены поля всех 20 эталонов по типам диаграмм** (с учётом `textLength`, иначе правое поле завышается для текстовых элементов): WBS 20/20/20/20, MINDMAP 10/20/20/20, JSON и YAML ≈10, TIMING 20/20/0/14, SALT 6/17/9/9, SEQUENCE 10/10/…, CLASS 7/7/14/14 | данные собраны | ✅ **Сделано** |
| 5.54 | **Проверено, что подстановка измеренных полей для WBS и MINDMAP делает ХУЖЕ, и почему.** WBS: dw 10 → 36 (dh 30 → 4), MINDMAP: dw 6 → 22, dh 7 → 19. Причина: у этих типов расхождение было в СОДЕРЖИМОМ, а неверное поле его маскировало. WBS-контент у нас на 36 шире эталонного. Изменение откачено, чтобы не выдавать маскировку за исправление | зафиксировано | ⏳ **Открыто** |

| 5.55 | **WBS: размер шрифта был 13 вместо эталонных 12.** Проверено по эталону: подписи `fs=12`, боксы узлов равны тексту + 20 (padding 10 с каждой стороны). При 13 наши боксы выходили на 8–10% шире. `font_size` 13 → 12. Ширины совпали: Project 62.32 (эталон 61.53), Phase 67.08 (67.28), Task 70.77 (**70.78**). `wbs_basic` dw 10 → **3px** | dw 10 → 3 | ✅ **Сделано** |
| 5.56 | **WBS: поле рендерера 10 вместо 7.** Движок WBS сам добавляет `padding = 10` к границам, поэтому рендереру нужно ещё 10, чтобы суммарное поле совпало с эталонным (20 со всех сторон). `wbs_basic` dh 30 → **24px**; общая 107x123 → **100x117** | dh 30 → 24 | ✅ **Сделано** |
| 5.57 | **Остаток WBS dh=24 — различие модели раскладки, а не поля.** Эталон имеет ЧЕТЫРЕ ряда узлов: Project (y=20), Phase 1 и 2 (93.97), Task 1.1 и 2.1 (142.94), Task 1.2 (191.91). Шаг: 73.97, затем 48.97 и 48.97. У нас три ряда с шагом 74. При этом у эталона дети КОРНЯ стоят в одном ряду, а дети Phase 1 — в одном столбце на разных рядах. Единое правило из данных не выводится: для корня и для Phase 1 поведение разное | зафиксировано | ⏳ **Открыто** |

| 5.58 | **object_basic: высота бокса 60 вместо 36.30, отступ 30 вместо 7, зазор 50 вместо 77.** Измерено по эталону: боксы 36.30 высотой при отступе 7 от края, расстояние между боксами 77 (7 → 120.29 при высоте 36.30). `object_min_height` 60 → 36.3, `padding` 30 → 7, `vertical_spacing` 50 → 77. `object_basic` dh 14 → 6px (после следующих правок — 1px) | dh 14 → 6 | ✅ **Сделано** |
| 5.59 | **Профиль полей CLASS перенесён в рендерер, поле движка обнулено.** class, ER и object помечаются PlantUML одним типом `CLASS` и имеют один профиль: слева и сверху 7, справа и снизу ~14.3 (измерено 14.28/14.41, 14.49/14.11, 13.90/13.41). Раньше асимметрия выражалась костылём в поле class-движка (3.5), а object и ER её не получали вовсе. `object_basic` dw 7 → **0**, `er_basic` dw 7 → **0**, `class_inheritance` 0 → 1 (округление). Общая ширина 100 → **87px** | dw → 0 у двух кейсов | ✅ **Сделано** |
| 5.60 | **`er_basic` dh 1 → 8px** после переноса профиля: у ER расхождение по высоте было замаскировано неверным полем. Содержимое ER на 7.7px выше эталонного | зафиксировано | ⏳ **Открыто** |

| 5.61 | **component: добавка к габаритам была одинаковой по осям, а эталонные поля разные.** Эталон `component_basic` имеет поля слева 7, справа **13.81**, сверху 7, снизу **29.42**. Движок добавлял `margin * 2 = 14` по обеим осям, из-за чего справа выходило 21 вместо 13.81. Добавка к ШИРИНЕ вынесена в отдельную измеренную константу `6.81` (поле рендерера даёт остальные 7). `component_basic` dw 7 → **0px**; общая ширина 87 → **80px** | dw 7 → 0 | ✅ **Сделано** |
| 5.62 | **Высотная добавка component оставлена прежней осознанно.** Замена нижней добавки на измеренную (22.42, чтобы суммарно вышло эталонные 29.42) дала `component_basic` dh 2 → 6 и `deployment_basic` dh 2 → 11: у обоих содержимое на 6–9px ниже эталонного, и прежнее поле это маскировало. Итог по сумме оказался хуже (204 против 198), поэтому возвращено прежнее значение. Измеренные 29.42 записаны | зафиксировано | ⚠️ **Частично** |

| 5.63 | **json: вложенная таблица ставилась верхом в центр строки, а не центрировалась по ней.** Эталон: таблица массива занимает 40.09..80.68 при строке `tags` 50.59..70.89 — её центр 60.385 совпадает с центром строки 60.74. У нас верх таблицы ставился в центр строки, и она уезжала вниз на **20.65px**. `layout_table` возвращает `Size`, поэтому высота берётся предварительным вызовом. Содержимое совпало с эталоном: 188x81 против 186.89x81.19 | сдвиг убран | ✅ **Сделано** |
| 5.64 | **Добавлен профиль полей JSON и YAML** (10, 10, 11.11, 11.81 — измерено). `json_basic` dw 6 → **2**, dh 8 → **0**; `yaml_basic` dw 6 → **1**, dh 7 → **1**. Общая 80x117 → **71x103** | точное совпадение | ✅ **Сделано** |

| 5.65 | **MINDMAP: поля дополнены до эталонных.** Движок mindmap сам добавляет `padding = 10` со всех сторон, поэтому рендереру нужен профиль `(0, 10, 10.14, 10.81)` — он доводит суммарные поля до измеренных 10/20/20.14/20.81. Проверка сошлась: вертикальный контент у нас и у эталона ровно 205.19. `mindmap_basic` dw 6 → **2**, dh 7 → **0**; общая 71x103 → **67x96** | dh → 0 | ✅ **Сделано** |
| 5.66 | **network_nwdiag: контент выходит за объявленные границы движка на 5px.** `bounds` вычисляются вручную (`band_x + network_width - padding`), а не по элементам: bounds.width 263.97 при фактическом максимуме x 268.97. Из-за этого полю рендерера нельзя просто задать нужное значение — контент обрежется. Нужно сперва привести границы к элементам | зафиксировано | ⏳ **Открыто** |

| 5.67 | **network_nwdiag: границы приведены к элементам (закрыт пункт 5.66).** Ручной расчёт давал bounds.width 263.97 при фактическом максимуме x 268.97 — контент вылезал за границы на 5px. Теперь ширина и высота берутся как максимум из ручного значения и фактического содержимого. Добавлен профиль NWDIAG `(0, 7, 6.27, 7)`. `network_nwdiag` dw 5 → **2px**; общая 67x96 → **64x96** | dw 5 → 2 | ✅ **Сделано** |

| 3.16 | **Найден перелив контента за границы: метка состояния timing обрезалась бы.** Ширина метки была зашита константой `50.0`, а подпись «Обработка» занимает 68.15 — она вылезала за правый край диаграммы на **11.15px**. Проверено системно: прогон по всем 20 кейсам с учётом `text-anchor` показал, что перелив был ТОЛЬКО здесь. Ширина берётся по измерителю, в конфиг добавлен `text` | перелив устранён | ✅ **Сделано** |
| 5.68 | **timing_basic dw 3 → 21px — осознанная регрессия.** Устранив перелив, мы получили настоящий размер контента: 250 против эталонных 229. Прежние 232 держались на том, что метка «Обработка» НЕ учитывалась в границах целиком. Разбираться нужно с шириной колонки подписей участников: эталон имеет ось времени на x=91.73 (вертикаль меток на x=20), у нас колонка `participant_label_width = 120` плюс padding 20. Была проверена и 71.73 — тогда контент наоборот становится на 27 уже эталонного | зафиксировано | ⏳ **Открыто** |

| 5.69 | **timing: разобрана эталонная геометрия, найдена связь колонки подписей и шкалы.** Эталон: вертикаль меток участников на x=20, ось времени начинается на x=**91.732**, отметка 100 на 191.732, правая граница шкалы 196.732. То есть колонка подписей = 91.732 − 20 = **71.732**. У нас 120, ось на 140. | данные собраны | ✅ **Сделано** |
| 5.70 | **Подбор `participant_label_width` и `time_scale` по отдельности не работает — это подгонка.** Проверено: колонка 71.732 даёт контент на 27px УЖЕ эталонного, `time_scale` 0.5 — на 20px уже. Сканирование `time_scale` от 0.40 до 0.50 показало монотонное «улучшение» метрики при понижении (dw 18 при 0.40, 28 при 0.50) — то есть метрика просто реагирует на сжатие, а не на корректность. Все изменения откачены, `time_scale` возвращён к 0.43. Нужен разбор связи «колонка подписей ↔ масштаб шкалы ↔ диапазон времени», а не подбор | откачено | ⏳ **Открыто** |

| 5.71 | **timing: шкала НЕ зависит от значений времени — измерено на трёх диапазонах.** Снял с сервера три диаграммы: `@0/@100`, `@0/@50`, `@0/@200`. Во ВСЕХ трёх вертикали шкалы одинаковы: 20.0 (колонка меток), 32.635, 82.635, 132.635, 137.635. То есть диапазон времени не влияет на геометрию. | измерено | ✅ **Сделано** |
| 5.72 | **timing: подписи оси расставляются равномерно по числу событий, а не по значениям.** Для `@0/@100` (два события) подписи «0» и «100» стоят на 29.14 и 72.14, а метки на 32.635 и 82.635. Для `@0/@30/@50` (три события) подписи «0», «30», «50» стоят на 29.14, 175.64, 275.64, метки на 32.635 … 332.635 с шагом 50. То есть PlantUML раскладывает подписи равномерно, игнорируя сами значения: 30 и 50 разнесены на 100px, хотя разница между ними 20, а между 0 и 30 — 30. Гипотеза «постоянный шаг 50px на любое число меток» не подтвердилась: при двух событиях три метки, при трёх — семь. Точное правило не выведено | зафиксировано | ⏳ **Открыто** |

| 5.73 | **ВЫВЕДЕНО ПРАВИЛО ШКАЛЫ TIMING (закрывает 5.72).** По пяти замерам с plantuml.com (2, 3, 4, 5 событий и диапазоны `@0/@50`, `@0/@200`): (1) первое деление ВСЕГДА на x=32.635 при колонке меток 20; (2) шаг делений ВСЕГДА 50.0 — независимо от значений времени и числа событий; (3) подпись ставится на КАЖДОЕ событие: для значений 0/25/50/75 подписи на 29.14, 75.64, 125.64, 175.64, то есть шаг 50, а не пропорционально значениям; (4) диапазон времени на геометрию не влияет. Прежний код масштабировал по значениям (`(t - min_time) * time_scale`) — PlantUML так не делает | правило выведено | ✅ **Сделано** |
| 5.74 | **Правка по выведенному правилу НЕ внесена: требует прокинуть диаграмму в `draw_time_axis`.** Функция получает `min_time`/`max_time`, но не список событий, а подписи нужно ставить по событиям. Черновик правки был внесён и откачен как незавершённый: менять наполовину опаснее, чем не менять. Само правило зафиксировано (5.73), реализация — отдельная задача | откачено | ⏳ **Открыто** |

| 5.75 | **Правило шкалы timing РЕАЛИЗОВАНО (закрывает 5.73/5.74).** Шкала строится по событиям, а не по значениям времени: `draw_time_axis` получает диаграмму и берёт уникальные времена в порядке появления. Удалён `calculate_time_step` (подбор «красивого шага»), константы `TIME_FIRST_TICK_OFFSET = 50.0` и `TIME_TICK_STEP = 50.0`. Смещение уточнено по двум независимым замерам: в `timing_basic` ось на 91.732, деления на 141.732/191.732; в пробе `@0/@100` ось на 32.635, деления на 82.635/132.635 — в обоих случаях первое деление отстоит от оси ровно на 50 | реализовано | ✅ **Сделано** |
| 5.76 | **Геометрия оси timing совпала с эталоном точно.** После правки: ось 91.73 (эталон 91.732), деления 141.23 и 191.23 (эталон 141.732 и 191.732). Ширина колонки подписей установлена 71.732 — она стала работать только теперь, потому что раньше шкала масштабировалась по значениям и не привязывалась к оси | совпадение | ✅ **Сделано** |
| 5.77 | **timing_basic dw 21 → 27px — осознанная регрессия.** Наш контент теперь кончается на 207.88 против эталонных 228.82: последний участок ломаной короче. Это уже не про шкалу (она совпала), а про длину последнего интервала состояния. Прежние 250 держались на сдвинутой вправо оси | зафиксировано | ⏳ **Открыто** |

| 5.78 | **Длина шкалы timing тоже считалась по значениям времени (закрывает 5.77).** `timeline_width = time_range * time_scale` давало 43 при эталонных 105. Выведено правило: шкала = `50 + (событий − 1) × 50 + 5`, то есть первое деление на +50 от оси, каждое следующее ещё +50, правый край на 5 правее последнего деления. Проверено на трёх замерах (`timing_basic`, `@0/@100`, `@0/@100/25/50/75`). Шкала совпала точно: 91.73..196.73 против эталонных 91.732..196.732 | точное совпадение | ✅ **Сделано** |
| 5.79 | **Удалён `calculate_time_step` и обвязка диапазона времени.** Диапазон времени на геометрию timing больше не влияет вовсе — это подтверждено и замерами (диаграммы с диапазонами 50, 100 и 200 дают одинаковые координаты), и структурой кода: `_max_time` берётся только для нижней границы ломаных | чистка | ✅ **Сделано** |
| 5.80 | **timing_basic dw остался 27px, но теперь это ЧИСТОЕ расхождение содержимого.** Шкала, ось и деления совпали с эталоном до третьего знака. Остаток — метка `state_label_Сервер_1_1` выходит на 207.88, тогда как эталонный контент кончается на 228.82: у нас короче последний участок ломаной состояния | зафиксировано | ⏳ **Открыто** |

| 5.81 | **Геометрия шкалы timing совпала с эталоном ПОЛНОСТЬЮ.** Уточнено правило: подпись ставится на САМУ ОСЬ и далее каждые +50 (по одной на событие), а деления идут с +50 — то есть делений на одно меньше, чем подписей. Ширина подписи измеряется, а не берётся константой 30 (для «100» реальная 20.996, и край выходил на 15 вместо 10.5). Сверено по двум замерам. Результат: ось 91.73, центры подписей 91.73 и 141.73, деления 141.73 и 191.73 — совпадает с эталонными 91.732 / 91.73 / 141.73 / 141.732 / 191.732 | полное совпадение | ✅ **Сделано** |
| 5.82 | **Убран последний источник геометрии по значениям времени.** `TIME_FIRST_TICK_OFFSET` больше не используется, деления привязаны к подписям, `calculate_time_step` удалён ранее. Диапазон времени влияет только на нижнюю границу ломаных состояний | чистка | ✅ **Сделано** |

| 5.83 | **deployment: формула ширины узла ПОДТВЕРЖДЕНА, разобрана до константы.** Извлечены точные каркасы узлов из эталона (по комментариям `<!--cluster ...-->`): узел «Сервер приложений» тело 29..260, узел «Сервер БД» тело 16..168. Формула: **ширина тела = заголовок(жирный) + 65.86**, а наша константа 76 = 65.86 + 10 (3D-скос) — то есть согласована. Проверка: 165.136 + 65.86 = 231 ✓, 86.181 + 65.82 = 152 ✓. Наша ширина 238.72 = 162.722 + 76 (расхождение 2.28 от точности измерения текста) | формула подтверждена | ✅ **Сделано** |
| 5.84 | **deployment: сдвиг узлов НЕ выводится из данных (расхождение локализовано).** Эталон сдвигает только узел 1: левые края 29 и 16 (сдвиг 13). Гипотеза «сдвиг = (w₁−w₂)/6» дала 13.167 при эталонных 13, но **опровергнута** тремя пробами: при РАВНЫХ ширинах сдвиги разные — artifact даёт +11, component −2, пустые узлы −0.01. Ни ширина, ни тип содержимого сдвиг не объясняют. Единственное расхождение deployment — эти 13px; холст эталона 295 против нашего 270.72 | зафиксировано | ⏳ **Открыто** |

| 6.1b | **Спрайты теперь доходят до AST.** Найдено: грамматика `sprite_stmt` принимала определение, но парсер его НЕ обрабатывал — спрайт молча терялся, то есть содержимое библиотек иконок разбиралось и пропадало. Добавлены тип `Sprite` в AST (имя, строки пикселей, размеры, палитра) и разбор в парсере class. Проверено на реальном спрайте: `!include <logos/rust>` даёт спрайт `rust` 48x48. Добавлен тест | спрайты в AST | ✅ **Сделано** |
| 6.1c | **Отрисовка спрайтов ещё не реализована.** Спрайт теперь доступен в AST, но в SVG не превращается: нужен рендеринг растра (каждая hex-цифра — пиксель) в набор `<rect>` с палитрой PlantUML, плюс подстановка `<$имя>` в подписи. Это отдельная задача | зафиксировано | ⏳ **Открыто** |

| 6.1d | **Для отрисовки спрайтов извлечена палитра PlantUML.** Расшифрован PNG, который PlantUML встраивает в `<image xlink:href="data:image/png;base64,...">`: тип цвета 6 (RGBA), размер = размер спрайта в пикселях. Палитра 16 индексов `0`..`f` — серый градиент от `#F1F1F1` (alpha 0) через `#A7A7A7` (alpha 255) до `#121212`. То есть спрайт можно нарисовать прямоугольниками без PNG-кодера: каждая hex-цифра — пиксель своего цвета | палитра получена | ✅ **Сделано** |
| 6.1e | **Отрисовка спрайтов требует проброса через layout.** Спрайты есть в AST `ClassDiagram`, но в `LayoutConfig` и элементы раскладки не попадают, а подстановка `<$имя>` в подписи не делается вовсе. Нужно: (1) прокинуть таблицу спрайтов в layout; (2) заменить `<$имя>` в текстах на элемент-растр; (3) отрисовать растр прямоугольниками по извлечённой палитре. Это связная задача на несколько шагов, а не одна правка | зафиксировано | ⏳ **Открыто** |

| 6.1f | **СПРАЙТЫ РИСУЮТСЯ (закрывает 6.1c/6.1e).** Добавлен `ElementType::Sprite`, рендеринг набором прямоугольников по извлечённой палитре (PNG-кодер не нужен, новых зависимостей нет) и создание элементов в class-движке. Сквозная проверка: спрайт `$s` 4x4 даёт в выводе цвета `#E2E2E2`, `#D3D3D3`, `#C5C5C5` (цифра `0` полностью прозрачна и не рисуется). Добавлен тест | спрайты рисуются | ✅ **Сделано** |

| 6.1g | **Найдены источники для Фазы 6.1: репозитории `plantuml-stdlib`.** Список получен через GitHub API: `C4-PlantUML`, `Azure-PlantUML`, `Archimate-PlantUML`, `EIP-PlantUML`, `plantuml-kubernetes-sprites`, `cicon-plantuml-sprites`, `gilbarbara-plantuml-sprites`. Ранее я предполагал имена `aws-icons`/`azure-icons`/`kubernetes-icons` — таких репозиториев нет, и это была догадка, а не проверенный факт | источники найдены | ✅ **Сделано** |
| 6.1h | **Спрайты kubernetes используют СЖАТЫЙ формат `[64x63/16z]` — наш разбор их отбрасывает.** Файл `k8s-sprites-unlabeled-25pct.iuml` (17.3 КБ, 33 спрайта) проверен: разбор проходит без ошибок, но спрайтов в AST 0, потому что тело — не шестнадцатеричные цифры, а сжатые данные (base64-подобный алфавит). Формат `16z` — это zlib-сжатие PlantUML. Для поддержки aws/azure/kubernetes нужен распаковщик этого формата | зафиксировано | ⏳ **Открыто** |

| 6.1i | **ФОРМАТ СПРАЙТОВ `16z` ПОЛНОСТЬЮ РАСКРЫТ (закрывает 6.1h).** Проверено на `k8s-sprites-unlabeled-25pct.iuml`: (1) тело — base64 с алфавитом PlantUML (`0-9A-Za-z-_`); (2) после декодирования — **raw deflate** (zlib без заголовка, `wbits=-15`); (3) распакованное — по **одному байту на пиксель**, значения `0..15` = индексы той же палитры, что у hex-формата. Контроль: 64×63 = 4032 пикселя, распаковалось ровно 4032 байта, максимум значения 15 | формат раскрыт | ✅ **Сделано** |
| 6.1j | **Для распаковки нужен inflate, а его в зависимостях нет.** В workspace нет ни `flate2`, ни `miniz_oxide`. Варианты: добавить крейт (проверить WASM-совместимость) либо реализовать raw deflate вручную (~80 строк, зависимостей не добавляет). Решение отложено до следующего шага | зафиксировано | ⏳ **Открыто** |

| 6.1k | **РЕАЛИЗОВАН РАСПАКОВЩИК `raw deflate` — сжатые спрайты разблокированы (закрывает 6.1j).** Написан вручную, без новых зависимостей (сохраняет WASM-совместимость): все три типа блоков по RFC 1951 — сохранённые, фиксированные и динамические коды Хаффмана. Проверено на **реальном** спрайте kubernetes `$master` из `k8s-sprites-unlabeled-25pct.iuml`: распаковка даёт ровно **4032** байта (64×63 по заголовку) со значениями `0..15` — то есть формат совместим с нашей палитрой | распаковщик готов | ✅ **Сделано** |
| 6.1l | **Процессная находка: тест на реальных данных поймал ошибку в моём коде.** Первая версия декодера хранила коды отдельно от символов и индексировала чужую таблицу — падение `index out of bounds: len is 10 but index is 17`. Синтетические тесты это пропускали; ошибку выявил тест на настоящем потоке из stdlib | исправлено | ✅ **Сделано** |
| 6.1m | **Остаётся: пробросить распаковку в разбор спрайтов.** `inflate` готов и проверен, но парсер по-прежнему принимает только hex-тело и отбрасывает сжатые спрайты. Нужно: распознать суффикс `z` в размере, декодировать base64 алфавитом PlantUML, распаковать, разложить байты в строки пикселей | зафиксировано | ⏳ **Открыто** |

| 6.1n | **Сжатые спрайты декодируются в строки пикселей (закрывает 6.1m).** Добавлена `decode_compressed_sprite`: перестановка алфавита base64, декодирование, распаковка, раскладка по строкам. Проверено на реальном `$master`: 63 строки по 64 hex-символа — ровно как заявлено в заголовке `[64x63/16z]`. Результат совместим с hex-форматом, поэтому отрисовка работает без изменений | декодирование работает | ✅ **Сделано** |
| 6.1o | **Остаётся: подключить декодирование в разбор спрайтов.** Функция готова и проверена, но парсер (`plantuml-parser`) не зависит от `plantuml-stdlib`, а зависимость есть у препроцессора. Нужно решить, где вызывать декодирование: либо в препроцессоре при раскрытии `!include`, либо добавить зависимость парсеру | зафиксировано | ⏳ **Открыто** |

| 6.1p | **Сжатые спрайты подключены к разбору (закрывает 6.1o).** `plantuml-parser` получил зависимость от `plantuml-stdlib`; распознаётся суффикс `z` в размере, сжатое тело декодируется. Сквозная проверка на реальном файле: `k8s-sprites-unlabeled-25pct.iuml` даёт **33 спрайта**, каждый 64x63 с 63 строками пикселей. Добавлен тест | 33 спрайта kubernetes | ✅ **Сделано** |
| 6.1q | **Проверено: спрайты kubernetes рисуются.** Цепочка замкнута целиком: `!include` → разбор → распаковка → раскладка → SVG с прямоугольниками палитры | проверено | ✅ **Сделано** |

| 6.1r | **Подстановка спрайтов в подписи реализована.** `<$имя>` разбивает строку на текст до, растр и текст после — как PlantUML. Спрайты передаются в рендерер через свойство `sprites` элемента; растр рисуется теми же прямоугольниками палитры. Проверено: `class "A <$s>"` даёт в выводе цвета спрайта `#E2E2E2` и `#121212` | работает | ✅ **Сделано** |
| 6.1s | **Вставка спрайта работает только в имени класса.** Формы `class A { текст <$s> }` и `A : <$s>` не разбираются: содержимое тела класса и строки-описания не принимают угловые скобки. Нужно расширить грамматику class — сейчас это ограничение разбора, а не отрисовки | зафиксировано | ⏳ **Открыто** |

| 6.1t | **Спрайты вставляются во все три формы подписи (закрывает 6.1s).** Добавлены правила грамматики: `class_description` (`A : текст` — раньше НЕ разбиралось вовсе) и `free_line` (произвольная строка тела). Проверено: имя класса, описание и тело — во всех трёх спрайт рисуется | три формы | ✅ **Сделано** |
| 6.1u | **ПРОЦЕССНАЯ НАХОДКА: порядок правил в грамматике критичен.** Первая версия поставила `free_line` ПЕРЕД `field`, и тогда `-id: Long` разбиралось как свободная строка — поля ИСЧЕЗЛИ из вывода (высота класса упала со 121 до 80 против эталонных 133). Снапшот-тесты это поймали. Правило перенесено ПОСЛЕ `field` | исправлено | ✅ **Сделано** |

| 6.1v | **Kubernetes добавлен в stdlib.** Модуль `kubernetes` с 33 спрайтами из репозитория `plantuml-stdlib/plantuml-kubernetes-sprites`. Регистрируются пути `kubernetes/kubernetes` и `k8s-sprites-unlabeled-25pct`. Проверено сквозным путём: `!include <kubernetes/kubernetes>` даёт 33 спрайта, `class "A <$master>"` рисует иконку | работает | ✅ **Сделано** |
| 6.1w | **Размер вывода спрайтов уменьшен в 11 раз.** Первая версия рисовала каждый пиксель отдельным `<rect>`: спрайт 64x63 давал **10.5 МБ** на диаграмму. Добавлено объединение блоков: сначала горизонтальные пробеги (10.5 → 1.3 МБ), затем слияние одинаковых пробегов по вертикали (**1.3 → 0.95 МБ**). Корректность проверена: суммарная площадь закраски равна числу непрозрачных пикселей (тест) | 10.5 МБ → 0.95 МБ | ✅ **Сделано** |

| 6.1x | **Azure добавлен в stdlib.** Модуль `azure` с 38 спрайтами из репозитория `plantuml-stdlib/Azure-PlantUML` (каталоги `Compute` и `Containers`), 76 зарегистрированных путей. Формат тот же, что у kubernetes — `[70x70/16z]`, поэтому распаковщик подошёл без изменений. Проверено: `!include <azure/Compute/AzureAppService>` даёт спрайт, подстановка в подписи рисует иконку (119 КБ вывода) | работает | ✅ **Сделано** |

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
