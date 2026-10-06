//! Актуальная библиотека C4-PlantUML из репозитория plantuml-stdlib.
//!
//! Файлы лежат в `assets/c4/` и вкомпилированы через `include_str!`.
//! Регистрируется ПОВЕРХ рукописного [`crate::c4`]: тот остаётся
//! запасным вариантом на случай, если набор файлов урезан.
//!
//! # Почему это заработало не сразу
//!
//! Библиотека целиком — 136 КБ, и первая попытка подключить её в июле
//! закончилась пиком памяти 8.5 ГБ. Причин было несколько, и каждая
//! чинилась отдельно:
//!
//! * несходящийся `!while` раздувал переменную до мегабайтов — добавлен
//!   бюджет раскрытия (`PreprocessError::ExpansionLimit`);
//! * арифметика с переменной (`!$i = $i - 1`) не вычислялась;
//! * вложенный `!while` исполнялся один раз при сборе тела внешнего;
//! * `\n` раскрывался в настоящий перевод строки, из-за чего съезжала
//!   арифметика индексов в `$breakText`;
//! * вложенные вызовы в аргументах не раскрывались регулярными
//!   выражениями — заменены рукописным сканером;
//! * переменная верхнего уровня не перекрывала одноимённую из области
//!   макросов, и подпись связи терялась.
//!
//! Итог: разбор библиотеки занимает около 300 мс, все 13 тестов
//! стандартной библиотеки проходят.

/// Файлы библиотеки: путь включения и содержимое.
const FILES: &[(&str, &str)] = &[
    ("C4/C4", include_str!("../assets/c4/C4.puml")),
    (
        "C4/C4_Context",
        include_str!("../assets/c4/C4_Context.puml"),
    ),
    (
        "C4/C4_Container",
        include_str!("../assets/c4/C4_Container.puml"),
    ),
    (
        "C4/C4_Component",
        include_str!("../assets/c4/C4_Component.puml"),
    ),
    (
        "C4/C4_Deployment",
        include_str!("../assets/c4/C4_Deployment.puml"),
    ),
    (
        "C4/C4_Dynamic",
        include_str!("../assets/c4/C4_Dynamic.puml"),
    ),
    (
        "C4/C4_Sequence",
        include_str!("../assets/c4/C4_Sequence.puml"),
    ),
];

/// Регистрирует включения библиотеки C4-PlantUML.
pub fn register(registry: &mut std::collections::HashMap<&'static str, &'static str>) {
    for (path, content) in FILES {
        registry.insert(path, content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Все файлы библиотеки зарегистрированы и непусты.
    #[test]
    fn test_files_registered() {
        let mut registry = std::collections::HashMap::new();
        register(&mut registry);
        for (path, content) in FILES {
            assert_eq!(registry.get(path), Some(content), "нет включения {path}");
            assert!(!content.is_empty(), "пустое включение {path}");
        }
        assert_eq!(registry.get("C4/C4").map(|c| c.len()), Some(68301));
    }
}
