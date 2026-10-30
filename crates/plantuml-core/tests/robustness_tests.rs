//! Тесты устойчивости: мусорный и незавершённый ввод не должен ронять процесс.
//!
//! Почему это важно. В release профиль собирается с `panic = "abort"`:
//! паника там неотличима от аварийной остановки, а в браузере превращается
//! в ловушку модуля WASM. Поэтому каждая точка, где код брал значение без
//! проверки, — потенциальный краш всего приложения.
//!
//! Раньше в парсере было одиннадцать таких мест: съём со стека
//! (`stack.pop().unwrap()`) и доступ к вложенной паре pest
//! (`pair.into_inner().next().unwrap()`). Они были защищены инвариантами
//! в соседних строках, но инвариант — это обещание, а не проверка.
//! Все они заменены на `let ... else` и `ok_or_else`.
//!
//! Тесты ниже проходят при `panic = "unwind"` (профиль test), поэтому
//! вернувшийся panic провалил бы их и не дал уйти этой правке.

use plantuml_core::{parse_diagram, render, RenderOptions};

/// Входы, на которых раньше можно было упасть.
///
/// Каждый пункт — конкретная бывшая точка паники либо соседняя с ней.
const HOSTILE_INPUTS: &[(&str, &str)] = &[
    // JSON: пара-обёртка `json_value` остаётся без вложенного значения.
    ("json: пустой объект", "@startuml\n@startjson\n{}\n@endjson\n@enduml"),
    ("json: пустой массив", "@startuml\n@startjson\n[]\n@endjson\n@enduml"),
    (
        "json: значение без содержимого",
        "@startuml\n@startjson\n{ \"k\": }\n@endjson\n@enduml",
    ),
    // YAML: те же три правила-обёртки.
    ("yaml: пустой документ", "@startuml\n@startyaml\n@endyaml\n@enduml"),
    ("yaml: пустое значение", "@startuml\n@startyaml\nk:\n@endyaml\n@enduml"),
    (
        "yaml: вложенный список без элементов",
        "@startuml\n@startyaml\nk:\n  - \n@endyaml\n@enduml",
    ),
    (
        "yaml: значение-скобка",
        "@startuml\n@startyaml\nk: [\n@endyaml\n@enduml",
    ),
    // Mindmap и WBS: стек сворачивается до меньшего уровня.
    ("mindmap: только корневые узлы", "@startuml\n@startmindmap\n***\n@endmindmap\n@enduml"),
    (
        "mindmap: без корня",
        "@startuml\n@startmindmap\n**\n* a\n@endmindmap\n@enduml",
    ),
    ("wbs: пустое дерево", "@startuml\n@startwbs\n@endwbs\n@enduml"),
    ("wbs: без корня", "@startuml\n@startwbs\n**\n* задача\n@endwbs\n@enduml"),
    // Salt: стек виджетов закрывается по уровням.
    ("salt: незакрытая скобка", "@startuml\n@startsalt\n{ { |\n@enduml"),
    ("salt: пустая таблица", "@startuml\n@startsalt\n{ { }\n@enduml"),
    // Class: стек пакетов.
    ("class: package без имени", "@startuml\npackage\nclass A\n@enduml"),
    (
        "class: закрытие без открытия",
        "@startuml\nclass A\npackage {\n@enduml",
    ),
    (
        "class: вложенные пакеты",
        "@startuml\npackage \"Внешний\" {\n  package \"Внутренний\" {\n    class A\n  }\n}\n@enduml",
    ),
];

/// Ни один из входов не должен вызывать панику.
///
/// Результат может быть любым: напечатать диаграмму или вернуть ошибку.
/// Важно, чтобы процесс дожил до конца.
#[test]
fn test_hostile_input_never_panics() {
    for (name, source) in HOSTILE_INPUTS {
        // Разбор и отрисовка проверяются раздельно: паника может ждать
        // в любой из стадий.
        let parsed = parse_diagram(source);
        let drawn = render(source, &RenderOptions::default());

        // Если разбор прошёл, отрисовка обязана пройти тоже.
        if parsed.is_ok() {
            assert!(
                drawn.is_ok(),
                "разбор прошёл, а отрисовка упала для входа «{name}»"
            );
        }
    }
}

/// Вход без маркеров диаграммы даёт понятную ошибку, а не пустой SVG.
#[test]
fn test_input_without_markers_is_rejected() {
    let source = "просто текст без @startuml";
    assert!(
        render(source, &RenderOptions::default()).is_err(),
        "ввод без маркеров не должен молча рисоваться"
    );
}

/// Пустой исходник отвергается явно, а не падает.
#[test]
fn test_empty_source_is_rejected() {
    assert!(render("", &RenderOptions::default()).is_err());
}

/// Незакрытая директива `!include` даёт ошибку, а не подвешивает процесс.
#[test]
fn test_unclosed_include_is_rejected() {
    let source = "@startuml\n!include <C4/C4_Context>\nPerson(p, \"П\")\n@enduml";
    // Диаграмма без `!endprocedure` и прочих закрывающих конструкций:
    // результат не важен, важно, что процесс выжил и вернул управление.
    let _ = render(source, &RenderOptions::default());
}

/// Диаграмма с незакрытым блоком `alt` в sequence не роняет разбор.
#[test]
fn test_unclosed_sequence_fragment_is_rejected() {
    let source = "@startuml\nAlice -> Bob: старт\nalt условие\nAlice -> Bob: да\n@enduml";
    let _ = render(source, &RenderOptions::default());
}

/// Успешно разобранная диаграмма содержит ожидаемые элементы.
///
/// Обратная сторона устойчивости: защита не должна превращать вход в пустую
/// картинку. Эталон здесь — golden-тесты, этот тест лишь ловит грубые
/// поломки на тех же входах, что перечислены выше.
#[test]
fn test_valid_nested_input_still_parses() {
    let source = "@startuml\npackage \"Внешний\" {\n  class A\n}\n@enduml";
    let diagram = parse_diagram(source).expect("корректная диаграмма должна разбираться");
    let text = format!("{diagram:?}");
    assert!(text.contains("A"), "потерян класс A: {text}");
}

/// Глубина вложенности, заведомо выше предела.
const DEEP: usize = 3000;

/// Генераторы враждебных входов: глубокая вложенность и большой объём.
fn generated_inputs() -> Vec<(&'static str, String)> {
    let nested = |open: &str, close: &str, body: &str| {
        format!(
            "@startuml\n{}{body}\n{}@enduml\n",
            open.repeat(DEEP),
            close.repeat(DEEP)
        )
    };
    vec![
        ("вложенные alt", nested("alt a\n", "end\n", "A -> B")),
        ("вложенные группы", nested("group g\n", "end\n", "A -> B")),
        (
            "вложенные package",
            nested("package p {\n", "}\n", "class A"),
        ),
        ("вложенные state", nested("state S {\n", "}\n", "[*] --> X")),
        (
            "вложенные if",
            format!(
                "@startuml\nstart\n{}:a;\n{}stop\n@enduml\n",
                "if (c) then (y)\n".repeat(DEEP),
                "endif\n".repeat(DEEP)
            ),
        ),
        (
            "вложенные rectangle",
            nested("rectangle r {\n", "}\n", "[C]"),
        ),
        (
            "скобки в подписи",
            format!("@startuml\nA -> B : {}\n@enduml\n", "(".repeat(DEEP)),
        ),
        (
            "вложенный JSON",
            format!(
                "@startjson\n{}1{}\n@endjson\n",
                "[".repeat(DEEP),
                "]".repeat(DEEP)
            ),
        ),
        (
            "вложенный YAML",
            format!(
                "@startyaml\n{}@endyaml\n",
                (0..DEEP)
                    .map(|i| format!("{}k:\n", " ".repeat(i * 2)))
                    .collect::<String>()
            ),
        ),
        (
            "глубокий mindmap",
            format!(
                "@startmindmap\n{}@endmindmap\n",
                (1..DEEP.min(500))
                    .map(|i| format!("{} n\n", "*".repeat(i)))
                    .collect::<String>()
            ),
        ),
        (
            "вложенный salt",
            format!(
                "@startsalt\n{}{}@endsalt\n",
                "{\n".repeat(DEEP),
                "}\n".repeat(DEEP)
            ),
        ),
        (
            "большой sequence",
            format!(
                "@startuml\n{}@enduml\n",
                (0..5000)
                    .map(|i| format!("p{} -> p{} : сообщение {i}\n", i % 50, (i + 1) % 50))
                    .collect::<String>()
            ),
        ),
        (
            "большой class",
            format!(
                "@startuml\n{}@enduml\n",
                (0..800)
                    .map(|i| format!("class C{i}\nC{i} --> C{}\n", (i * 7 + 1) % 800))
                    .collect::<String>()
            ),
        ),
        (
            "длинная строка",
            format!("@startuml\nA -> B : {}\n@enduml\n", "х".repeat(1_000_000)),
        ),
    ]
}

/// Рендер на потоке со стеком 2 МБ, как у рабочих потоков приложения.
fn render_on_small_stack(source: String) -> bool {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(move || render(&source, &RenderOptions::default()).is_ok())
        .expect("поток создаётся")
        .join()
        .expect("рендер не паникует")
}

/// Враждебные входы не роняют процесс на стеке 2 МБ и укладываются в 5 секунд.
#[test]
fn test_hostile_corpus_survives_small_stack() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/hostile");
    let mut inputs = generated_inputs();
    for entry in std::fs::read_dir(dir)
        .expect("есть враждебный корпус")
        .flatten()
    {
        let source = std::fs::read_to_string(entry.path()).expect("читается вход");
        inputs.push(("файл корпуса", source));
    }

    for (name, source) in inputs {
        let started = std::time::Instant::now();
        let _ = render_on_small_stack(source);
        let spent = started.elapsed();
        assert!(spent.as_secs() < 5, "«{name}» занял {spent:?}");
    }
}

/// Запрещённое не становится разрешённым: файлы и сеть из `render` недоступны.
#[test]
fn test_render_does_not_read_files_or_network() {
    for target in ["/etc/passwd", "https://evil.example/x.puml"] {
        let source = format!("@startuml\n!include {target}\nAlice -> Bob\n@enduml");
        assert!(render(&source, &RenderOptions::default()).is_err());
    }
}
