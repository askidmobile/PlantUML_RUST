//! Встроенные функции препроцессора PlantUML
//!
//! Поддерживаемые функции:
//! - Дата/время: `%date()`, `%time()`
//! - Метаданные: `%version()`, `%filename()`, `%dirpath()`
//! - Логические: `%true()`, `%false()`, `%not(expr)`
//! - Строковые: `%strlen(s)`, `%substr(s, start, len)`, `%upper(s)`, `%lower(s)`,
//!   `%strpos(s, needle)`, `%string(x)`, `%newline()`
//! - Числовые: `%intval(s)`, `%floor(x)`, `%ceil(x)`, `%abs(x)`

// ВАЖНО: аргументы могут быть заданы ПЕРЕМЕННОЙ, а не литералом.
//
// PlantUML хранит значение без кавычек (`!$s = "abc"` даёт `abc`,
// проверено на сервере: `%strlen($s)` равно 3), поэтому после
// подстановки вызов выглядит как `%strpos(a+b, "+")`.

/// Обрабатывает builtin функции в строке
pub fn process_builtins(line: &str) -> String {
    // Быстрый выход: без знака `%` встроенных функций быть не может.
    //
    // Ниже около двадцати регулярных выражений, и каждое сканирует строку
    // целиком. Строк же в библиотеке C4-PlantUML десятки тысяч (каждая
    // строка тела каждого макроса на каждом уровне вложенности), поэтому
    // проверка одного символа экономит львиную долю времени разбора.
    if !line.as_bytes().contains(&b'%') {
        return line.to_string();
    }

    let mut result = line.to_string();

    // === Простые функции без аргументов ===

    // %date()
    if result.contains("%date()") {
        let date = get_current_date();
        result = result.replace("%date()", &date);
    }

    // %time()
    if result.contains("%time()") {
        let time = get_current_time();
        result = result.replace("%time()", &time);
    }

    // %version()
    if result.contains("%version()") {
        result = result.replace("%version()", env!("CARGO_PKG_VERSION"));
    }

    // %true()
    result = result.replace("%true()", "true");

    // %false()
    result = result.replace("%false()", "false");

    // %newline()
    result = result.replace("%newline()", "\n");

    //  — функция, пришедшая на смену  в
    // PlantUML v1.2025.1beta6. Стандартная библиотека C4 выбирает её
    // через .
    result = result.replace("%breakline()", "\n");

    // %tab()
    result = result.replace("%tab()", "\t");

    // === Строковые функции ===

    // %strlen("string")
    // Проверка подстроки ДЕШЕВЛЕ регулярного выражения: ниже в каждой
    // функции своя регулярка, и она сканирует строку целиком.
    // Вложенные вызовы раскрываются сканером: регулярные выражения
    // скобки в аргументах не разбирают.
    result = expand_nested_calls(&result);

    result
}

// === Строковые функции ===

// === Сканер вызовов с поддержкой ВЛОЖЕННОСТИ ===

/// Раскрывает вызовы встроенных функций, ПОДДЕРЖИВАЯ ВЛОЖЕННЫЕ вызовы.
///
/// Прежде каждая функция раскрывалась своим регулярным выражением, а
/// шаблоны запрещали скобки в аргументах (`[^",)]*?`). Из-за этого вызов
/// вида `%substr(%substr($t, 1), 2)` НЕ раскрывался, и в библиотеке
/// C4-PlantUML значение переменной накапливалось как
/// `%substr(%substr(%substr(...` — доросло до 1.3 МБ, а разбор занимал
/// 87 секунд. Регулярным выражением вложенные скобки не разобрать
/// (нужен счётчик глубины), поэтому здесь рукописный сканер.
fn expand_nested_calls(text: &str) -> String {
    expand_nested_calls_at(text, 0)
}

fn expand_nested_calls_at(text: &str, depth: usize) -> String {
    // Предел глубины: защита от патологической вложенности.
    if depth > 32 {
        return text.to_string();
    }

    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] != b'%' {
            let ch = text[i..].chars().next().unwrap_or(' ');
            result.push(ch);
            i += ch.len_utf8();
            continue;
        }

        let Some((name, open)) = parse_call_name(text, i) else {
            result.push('%');
            i += 1;
            continue;
        };
        let Some(close) = find_matching_paren(text, open) else {
            result.push('%');
            i += 1;
            continue;
        };

        // Сначала раскрываем ВЛОЖЕННЫЕ вызовы в аргументах.
        let inner = expand_nested_calls_at(&text[open + 1..close], depth + 1);
        let args = split_top_level_args(&inner);

        match apply_builtin(&name, &args) {
            Some(value) => result.push_str(&value),
            None => {
                // Неизвестная функция: оставляем как есть, но с уже
                // раскрытыми аргументами — её обработает следующий проход.
                result.push('%');
                result.push_str(&name);
                result.push('(');
                result.push_str(&inner);
                result.push(')');
            }
        }
        i = close + 1;
    }

    result
}

/// Читает имя функции после `%` и позицию открывающей скобки.
fn parse_call_name(text: &str, start: usize) -> Option<(String, usize)> {
    let rest = text.get(start + 1..)?;
    let mut name = String::new();
    for ch in rest.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            name.push(ch);
        } else {
            break;
        }
    }
    if name.is_empty() {
        return None;
    }
    let open = start + 1 + name.len();
    if text.as_bytes().get(open) != Some(&b'(') {
        return None;
    }
    Some((name, open))
}

/// Индекс закрывающей скобки для открывающей на `open`.
fn find_matching_paren(text: &str, open: usize) -> Option<usize> {
    let mut depth = 0i32;
    for (offset, byte) in text.bytes().enumerate().skip(open) {
        match byte {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(offset);
                }
            }
            _ => {}
        }
    }
    None
}

/// Делит аргументы по запятым ВЕРХНЕГО уровня, не трогая кавычки и скобки.
fn split_top_level_args(inner: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut depth = 0i32;
    let mut quote: Option<char> = None;

    for ch in inner.chars() {
        match ch {
            '"' | '\'' if quote.is_none() => {
                quote = Some(ch);
                current.push(ch);
            }
            c if Some(c) == quote => {
                quote = None;
                current.push(ch);
            }
            '(' if quote.is_none() => {
                depth += 1;
                current.push(ch);
            }
            ')' if quote.is_none() => {
                depth -= 1;
                current.push(ch);
            }
            ',' if quote.is_none() && depth == 0 => {
                args.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if !current.trim().is_empty() || !args.is_empty() {
        args.push(current.trim().to_string());
    }
    args
}

/// Снимает парные кавычки — и двойные, и одинарные.
fn unquote(argument: &str) -> &str {
    let trimmed = argument.trim();
    for quote in ['"', '\''] {
        if trimmed.len() >= 2 && trimmed.starts_with(quote) && trimmed.ends_with(quote) {
            return &trimmed[1..trimmed.len() - 1];
        }
    }
    trimmed
}

/// Применяет встроенную функцию. `None` — функция не наша.
fn apply_builtin(name: &str, args: &[String]) -> Option<String> {
    let first = args.first().map(String::as_str).unwrap_or_default();
    let second = args.get(1).map(String::as_str).unwrap_or_default();
    let third = args.get(2).map(String::as_str).unwrap_or_default();

    match name {
        "strlen" => Some(unquote(first).chars().count().to_string()),
        "upper" => Some(unquote(first).to_uppercase()),
        "lower" => Some(unquote(first).to_lowercase()),
        "substr" => {
            let source = unquote(first);
            let start = eval_index(second).unwrap_or(0).max(0) as usize;
            let chars: Vec<char> = source.chars().collect();
            if start >= chars.len() {
                return Some(String::new());
            }
            Some(match eval_index(third) {
                Some(length) => chars
                    .iter()
                    .skip(start)
                    .take(length.max(0) as usize)
                    .collect(),
                None => chars.iter().skip(start).collect(),
            })
        }
        "strpos" => Some(match unquote(first).find(unquote(second)) {
            Some(position) => position.to_string(),
            None => "-1".to_string(),
        }),
        "string" => Some(format!("\"{}\"", unquote(first))),
        "intval" => Some(unquote(first).parse::<i64>().unwrap_or(0).to_string()),
        "floor" => Some(
            unquote(first)
                .parse::<f64>()
                .map(|value| value.floor() as i64)
                .unwrap_or(0)
                .to_string(),
        ),
        "ceil" => Some(
            unquote(first)
                .parse::<f64>()
                .map(|value| value.ceil() as i64)
                .unwrap_or(0)
                .to_string(),
        ),
        "abs" => Some(
            unquote(first)
                .parse::<i64>()
                .map(|value| value.abs().to_string())
                .or_else(|_| {
                    unquote(first)
                        .parse::<f64>()
                        .map(|value| value.abs().to_string())
                })
                .unwrap_or_else(|_| "0".to_string()),
        ),
        "not" => Some(match unquote(first).to_lowercase().as_str() {
            "true" | "1" => "false".to_string(),
            _ => "true".to_string(),
        }),
        _ => None,
    }
}

/// Вычисляет простое арифметическое выражение индекса.
///
/// Поддерживает `+`, `-`, `*`, `/` и скобки не требует: PlantUML в
/// аргументах встроенных функций пишет именно такие выражения
/// (`$brPos + 1`, `$width - 2`). Возвращает `None`, если разобрать не
/// удалось — тогда вызывающий код оставляет вызов как есть.
pub(crate) fn eval_index(expression: &str) -> Option<i64> {
    let expression = expression.trim();
    if expression.is_empty() {
        return None;
    }
    if let Ok(value) = expression.parse::<i64>() {
        return Some(value);
    }
    // Сложение и вычитание — самый низкий приоритет, ищем ПОСЛЕДНИЙ знак.
    for operators in [['+', '-'], ['*', '/']] {
        for (index, ch) in expression.char_indices().rev() {
            if index == 0 || !operators.contains(&ch) {
                continue;
            }
            let (left, right) = (&expression[..index], &expression[index + 1..]);
            let (Some(left), Some(right)) = (eval_index(left), eval_index(right)) else {
                continue;
            };
            return match ch {
                '+' => Some(left + right),
                '-' => Some(left - right),
                '*' => Some(left * right),
                '/' if right != 0 => Some(left / right),
                _ => None,
            };
        }
    }
    None
}

// === Числовые функции ===

// === Логические функции ===

// === Вспомогательные функции ===

/// Возвращает текущую дату в формате YYYY-MM-DD
fn get_current_date() -> String {
    // Для WASM-совместимости используем cfg
    #[cfg(target_arch = "wasm32")]
    {
        // В WASM возвращаем placeholder (можно заменить на js_sys::Date)
        "2024-01-01".to_string()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::SystemTime;

        // Получаем время с эпохи Unix
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();

        // Простой расчёт даты (без учёта часовых поясов)
        let days = now.as_secs() / 86400;
        let (year, month, day) = days_to_ymd(days);

        format!("{:04}-{:02}-{:02}", year, month, day)
    }
}

/// Возвращает текущее время в формате HH:MM:SS
fn get_current_time() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        "12:00:00".to_string()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::SystemTime;

        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();

        let secs = now.as_secs() % 86400;
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;

        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

/// Преобразует количество дней с эпохи Unix в (year, month, day)
#[cfg(not(target_arch = "wasm32"))]
fn days_to_ymd(days: u64) -> (u32, u32, u32) {
    // Алгоритм из Howard Hinnant's date algorithms
    let z = days as i64 + 719468;
    let era = if z >= 0 {
        z / 146097
    } else {
        (z - 146096) / 146097
    };
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    (y as u32, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let result = process_builtins("version: %version()");
        // Проверяем что версия содержит формат X.Y.Z
        assert!(result.contains("version:"));
        assert!(result.contains('.'));
    }

    #[test]
    fn test_true_false() {
        let result = process_builtins("value = %true()");
        assert_eq!(result, "value = true");

        let result = process_builtins("value = %false()");
        assert_eq!(result, "value = false");
    }

    #[test]
    fn test_strlen() {
        let result = process_builtins(r#"len = %strlen("hello")"#);
        assert_eq!(result, "len = 5");

        let result = process_builtins(r#"len = %strlen("")"#);
        assert_eq!(result, "len = 0");

        // Юникод
        let result = process_builtins(r#"len = %strlen("привет")"#);
        assert_eq!(result, "len = 6");
    }

    #[test]
    fn test_upper_lower() {
        let result = process_builtins(r#"upper = %upper("hello")"#);
        assert_eq!(result, "upper = HELLO");

        let result = process_builtins(r#"lower = %lower("WORLD")"#);
        assert_eq!(result, "lower = world");
    }

    #[test]
    fn test_substr() {
        let result = process_builtins(r#"sub = %substr("hello", 1, 3)"#);
        assert_eq!(result, "sub = ell");

        let result = process_builtins(r#"sub = %substr("hello", 2)"#);
        assert_eq!(result, "sub = llo");

        let result = process_builtins(r#"sub = %substr("hello", 10)"#);
        assert_eq!(result, "sub = ");
    }

    /// Управляющие последовательности остаются ДВУМЯ символами.
    ///
    /// Проверено на сервере: `!$t = "ab\ncd"` даёт `%strlen($t)` = 6 и
    /// `%strpos($t, "\n")` = 2. Прежде мы заменяли `\n` настоящим
    /// переводом строки (длина 5), и вся арифметика индексов в
    /// `$breakText` из C4-PlantUML съезжала: текст не укорачивался,
    /// `$multiLine` дорос до 20 МБ, разбор занимал 87 секунд.
    #[test]
    fn test_escape_sequences_stay_two_chars() {
        let preprocessor = crate::Preprocessor::new();
        let source = "@startuml\n!$t = \"ab\\ncd\"\nA: [%strlen($t)]\nB: [%strpos($t, \"\\n\")]\nC: [%substr($t, 0, 2)]\n@enduml\n";
        let result = preprocessor.process(source).expect("разбор должен пройти");
        assert!(result.contains("A: [6]"), "длина не 6: {result}");
        assert!(result.contains("B: [2]"), "позиция не 2: {result}");
        assert!(result.contains("C: [ab]"), "подстрока не ab: {result}");
    }

    /// ВЛОЖЕННЫЕ вызовы в первом аргументе раскрываются.
    ///
    /// Регрессия: шаблоны регулярных выражений запрещали скобки в
    /// аргументах, поэтому `%substr(%substr($t, 1), 2)` не раскрывался.
    /// В библиотеке C4-PlantUML значение переменной накапливалось как
    /// `%substr(%substr(%substr(...`, доросло до 1.3 МБ, и разбор занимал
    /// 87 секунд. Проверено на сервере: `%substr(%substr("abcdef", 1), 2)`
    /// равно `def`, `%strlen(%upper("abc"))` равно 3.
    #[test]
    fn test_nested_builtin_calls() {
        assert_eq!(
            process_builtins(r#"x = %substr(%substr("abcdef", 1), 2)"#),
            "x = def"
        );
        assert_eq!(process_builtins(r#"x = %strlen(%upper("abc"))"#), "x = 3");
        assert_eq!(
            process_builtins(r#"x = %strpos(%lower("ABC"), "b")"#),
            "x = 1"
        );
    }

    /// Арифметика в присваивании с ПЕРЕМЕННОЙ в операнде.
    ///
    /// Регрессия: `!$p = 5`, затем `!$p = $p - 1` сохраняло текст «5 - 1».
    /// Из-за этого `!$brPos = $brPos - 1` в `$breakText` из C4-PlantUML не
    /// уменьшал счётчик, и внутренний `!while` крутился 10 000 итераций.
    /// Проверено на сервере: результат равен 4.
    #[test]
    fn test_assignment_arithmetic_with_variable() {
        let preprocessor = crate::Preprocessor::new();
        let source = "@startuml\n!$p = 5\n!$p = $p - 1\nA: [$p]\n!$q = $p * 2\nB: [$q]\n@enduml\n";
        let result = preprocessor.process(source).expect("разбор должен пройти");
        assert!(
            result.contains("A: [4]"),
            "вычитание не вычислилось: {result}"
        );
        assert!(
            result.contains("B: [8]"),
            "умножение не вычислилось: {result}"
        );
    }

    /// Индекс может быть АРИФМЕТИЧЕСКИМ ВЫРАЖЕНИЕМ.
    ///
    /// Регрессия: библиотека C4-PlantUML пишет `%substr($text, $brPos + 1)`.
    /// Прежний шаблон принимал только `\d+`, вызов оставался текстом,
    /// переменная `$text` в `$breakText` не укорачивалась — цикл `!while`
    /// крутился все 10 000 итераций (42 секунды на вызов).
    #[test]
    fn test_substr_with_expression_index() {
        // Проверено на сервере: %substr("abcdef", 2 + 1) = "def".
        assert_eq!(
            process_builtins(r#"x = %substr("abcdef", 2 + 1)"#),
            "x = def"
        );
        assert_eq!(
            process_builtins(r#"x = %substr("abcdef", 4 - 1)"#),
            "x = def"
        );
        assert_eq!(
            process_builtins(r#"x = %substr("abcdef", 1 + 1, 2 * 2)"#),
            "x = cdef"
        );
    }

    #[test]
    fn test_strpos() {
        let result = process_builtins(r#"pos = %strpos("hello world", "wor")"#);
        assert_eq!(result, "pos = 6");

        let result = process_builtins(r#"pos = %strpos("hello", "xyz")"#);
        assert_eq!(result, "pos = -1");
    }

    #[test]
    fn test_intval() {
        let result = process_builtins(r#"val = %intval("42")"#);
        assert_eq!(result, "val = 42");

        let result = process_builtins(r#"val = %intval("-17")"#);
        assert_eq!(result, "val = -17");

        let result = process_builtins(r#"val = %intval("abc")"#);
        assert_eq!(result, "val = 0");
    }

    #[test]
    fn test_floor_ceil() {
        let result = process_builtins("val = %floor(3.7)");
        assert_eq!(result, "val = 3");

        let result = process_builtins("val = %ceil(3.2)");
        assert_eq!(result, "val = 4");

        let result = process_builtins("val = %floor(-2.3)");
        assert_eq!(result, "val = -3");
    }

    #[test]
    fn test_abs() {
        let result = process_builtins("val = %abs(-5)");
        assert_eq!(result, "val = 5");

        let result = process_builtins("val = %abs(3.14)");
        assert_eq!(result, "val = 3.14");

        let result = process_builtins("val = %abs(-2.5)");
        assert_eq!(result, "val = 2.5");
    }

    #[test]
    fn test_not() {
        let result = process_builtins("val = %not(true)");
        assert_eq!(result, "val = false");

        let result = process_builtins("val = %not(false)");
        assert_eq!(result, "val = true");

        let result = process_builtins("val = %not(1)");
        assert_eq!(result, "val = false");

        let result = process_builtins("val = %not(0)");
        assert_eq!(result, "val = true");
    }

    #[test]
    fn test_tab_newline() {
        let result = process_builtins("a%tab()b");
        assert_eq!(result, "a\tb");

        let result = process_builtins("a%newline()b");
        assert_eq!(result, "a\nb");
    }

    #[test]
    fn test_date_format() {
        let result = process_builtins("today = %date()");
        // Проверяем формат YYYY-MM-DD
        assert!(result.contains("today = "));
        let date_part = result.strip_prefix("today = ").unwrap();
        assert!(date_part.len() == 10);
        assert!(date_part.chars().nth(4) == Some('-'));
        assert!(date_part.chars().nth(7) == Some('-'));
    }

    #[test]
    fn test_time_format() {
        let result = process_builtins("now = %time()");
        // Проверяем формат HH:MM:SS
        let time_part = result.strip_prefix("now = ").unwrap();
        assert!(time_part.len() == 8);
        assert!(time_part.chars().nth(2) == Some(':'));
        assert!(time_part.chars().nth(5) == Some(':'));
    }

    #[test]
    fn test_combined_builtins() {
        let result = process_builtins(r#"result = %upper("test") + %strlen("hello")"#);
        assert_eq!(result, "result = TEST + 5");
    }
}
