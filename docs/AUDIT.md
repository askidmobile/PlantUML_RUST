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
| 3.1 | **Ввести слой измерения текста.** Trait `TextMeasurer` с реализациями: (а) детерминированная таблица ширин глифов, совместимая с PlantUML-шрифтом; (б) опционально `fontdb`+`ab_glyph` на нативных платформах. Вынести из layout-движков все восемь эвристик | ✅ **Начат.** Модуль `plantuml-layout/src/text.rs`: `TextMeasurer` с ширинами глифов в долях em, откалиброванными по эталонам PlantUML (`Alice` при 14px → `textLength="33.667"`, то есть 0.481 em/символ). Учитывает узкие/широкие символы и многострочность. 7 тестов. Подключён в class-движок; остальные — далее |
| 3.2 | **Устранить байтовый счёт.** Заменить `len()` на `chars().count()` / измерение — минимум в 5 местах (`class/graph.rs:59,73,88`, `wbs/engine.rs:147`, `er/engine.rs:38,51`, `json/engine.rs:265`, `salt/engine.rs` ×11). Добавить тест на кириллицу | ✅ **Сделано.** Все байтовые измерения устранены: class (3), wbs (1), er (2), json (1), salt (11). Проверка `grep '\.len() as f64 \* [0-9]'` по layout даёт пусто. `char_width` удалён из конфигов class и wbs как ненужный. `min_class_width` снижен с 120 до 40 — он перебивал измерение. Разрыв по class_inheritance: 136 → 65px |
| 3.2a | **Обнаружено: идентификаторы в грамматиках были только ASCII.** Кириллица не работала ни в одной диаграмме вне кавычек: `class Пользователь` и `+ имя: Строка` давали ошибку парсинга, хотя проект требует русский язык | ✅ **Сделано.** 38 определений идентификаторов в 10 грамматиках переведены на `LETTER`/`NUMBER` (Unicode). Кириллица проверена во всех типах: class, sequence, state, activity, usecase, component, ER |
| 3.3 | **Пробросить тему и `skinparam` в рендерер.** Изменить `Preprocessor::process` так, чтобы он возвращал обработанный текст **и** тему (или отдельный метод `process_with_theme`); передать тему через `RenderOptions` | ✅ **Сделано.** Добавлен `Preprocessor::process_with_theme`, возвращающий `(String, Theme)`; pipeline применяет тему из исходника поверх темы из опций. Проверено: все 6 вариантов (`monochrome`, `backgroundColor`, `defaultFontName`, `FontColor`, `BorderColor`, `!theme dark`) теперь меняют вывод, базовый — нет. 4 теста |
| 3.4 | **Оживить мёртвые поля `Theme`** (`line_width`, `corner_radius`, `shadow`, `handwritten`) в рендерере или удалить их | ⚠️ **Частично.** `SkinParams::apply_to` расширен с 5 до 9 ключей (добавлены `monochrome`, `FontColor`, `BorderColor`, `BackgroundColor`, `LineThickness`, `roundCorner`). Поля `line_width`, `corner_radius`, `shadow`, `handwritten` по-прежнему не читаются рендерером — остаётся |
| 3.5 | **Исправить ложно заявленный цвет фона**: `plantuml-renderer/src/lib.rs:67` — привести комментарий и дефолт в соответствие с PlantUML (`#FFFFFF`) | ✅ **Сделано.** Ложный комментарий про `#FEFECE` убран вместе с дублирующим margin в рендерере |
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
| 5.1 | `class` | ⚠️ **Частично.** Исправлены цвета и начертание по эталону: тело класса `#F1F1F1` (было `#E2E2F0`), имя класса без `bold` (в эталоне обычное начертание), набор заливок совпал с эталоном. Ранее также: ширины стали контентными (`min_class_width` 120 → 40), измерение текста переведено на `TextMeasurer`. Остаётся: dummy-вершины для рёбер через слои, подсчёт пересечений и шаг transpose, выравнивание Brandes–Köpf, рамки `package`, направление раскладки, рёбра Безье |
| 5.2 | `state` | ⚠️ **Частично.** Оформление приведено к эталону: заливка состояния `#F1F1F1` (было `#E2E2F0`), `rx/ry` 12.5 (было 10), толщина границы 0.5 (было 1), высота заголовка 26.297 (было 25), имя без жирного начертания. Остаётся: параллельные регионы (`State.regions` не читается нигде), `entry`/`exit`/`do`, `EntryPoint`/`ExitPoint` (рисуются полноразмерным прямоугольником), заметки, переопределение конфига литералами |
| 5.3 | `activity` | ⚠️ **Частично.** Ветки `elseif` реализованы: раньше поле `elseif_branches` не читалось нигде, и конструкция `if / elseif / else` молча теряла промежуточные ветки. Теперь они размещаются каскадом вправо-вниз с метками условий; добавлен тест. Остаётся: заметки и коннекторы (отфильтрованы до обработки), `Detach`/`Kill`, цвета и стили действий, двойная диспетчеризация |
| 5.4 | `component` | ✅ **Частично.** Эмодзи-глифы убраны: `database` → `ElementType::Database` (цилиндр), `actor` → `ElementType::Actor` (стик-фигура), у остальных (component, queue, node, folder, cloud) убраны символы `⬡`/`⟿`/`⬢`/`📁`/`☁` из подписей. Остаётся: контентные размеры вместо фиксированных 140×60 и ортогональная маршрутизация связей |
| 5.5 | `usecase` | ⚠️ **Частично.** Актёры разнесены: при совпадении Y второй актёр сдвигается по X на ширину блока, наложение устранено (добавлен тест на попарное непересечение). Остаётся: `left to right direction` (переменная вычисляется и не используется), контентные размеры эллипсов |
| 5.6 | `er` | Разобраться с грамматикой (не работает вообще); убрать две разные константы ширины символа в одной функции; передавать кардинальности через поля `ElementType::Edge`, а не через `properties` |
| 5.7 | `gantt` | ⚠️ **Частично.** Исправлена грамматика: модификаторы задач, соединённые союзом `and` (`[T1] starts 2024-02-01 and lasts 3 days`), не разбирались вовсе. Ранее также исправлено раздувание высоты до ~1000px (см. 2.7). Остаётся: учесть даты в геометрии (`TaskStart::AtDate`, `TaskDuration::Until` захардкожены), день недели от реальной даты старта (сейчас `day % 7`), относительные даты `D+5`, подпись дней с переходом через месяц |
| 5.8 | `timing` | Разобраться с масштабом (сейчас `time_scale = 3.0` выдуман); `TimeValue::Named` (сейчас игнорируется); одиночное изменение состояния сейчас не рисует ничего (`windows(2)`) |
| 5.9 | `network` | Ширина полосы сети от состава сети, а не от общего числа серверов; убрать выдуманные цвета (`#FFFFCC`, `#CCE5FF`, ...); векторные формы устройств вместо прямоугольников |
| 5.10 | `mindmap`/`wbs` | Устранить копипасту (4 функции почти идентичны); убрать сырые указатели `HashMap<*const MindMapNode, f64>` как ключи; удалить `struct NodeLayout`; в `wbs` заменить байтовый счёт на символы |
| 5.11 | `json`/`yaml` | Устранить 90-строчный копипаст `layout_object`/`layout_array`; `yaml` не должен показывать JSON-синтаксис (`key: {`) |
| 5.12 | `salt` | Разобрать `layout()` (игнорирует конфиг); `Wavy`-разделитель сейчас пунктир (`4,2`) вместо волнистой линии; дерево всегда рисует префикс `├─`, последний ребёнок не получает `└─` |
| 5.13 | `object`, `deployment`, `archimate` | `archimate` не парсится вовсе; `deployment`/`archimate` не имеют своих движков (переиспользуют component) — либо реализовать, либо честно задокументировать |

### Фаза 6. Расширение совместимости (постоянно)

| # | Задача |
|---|--------|
| 6.1 | Расширить `stdlib`: сейчас 39 include — нет ни `aws`, ни `azure`, ни `kubernetes`, ни `material`. Даже `C4_Context` — подмножество (не хватает 17 макросов, включая `Rel_D/U/L/R`, `Boundary`, `UpdateElementStyle`). Решить: генерировать из первоисточников или документировать ограничение |
| 6.2 | Реализовать `skinparam`-каталог (сейчас маппится 5 ключей) |
| 6.3 | Добавить визуальную регрессию для остальных 13 типов (сейчас снапшоты только у sequence/class/state) |
| 6.4 | Ввести бенчмарки (`benches/` пуста, но README публикует цифры) или убрать цифры из README |
| 6.5 | Рассмотреть CLI (сейчас его нет, только библиотека и WASM) |

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
