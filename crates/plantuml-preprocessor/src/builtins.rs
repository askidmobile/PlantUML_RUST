//! Встроенные функции препроцессора PlantUML
//!
//! Поддерживаемые функции:
//! - Дата/время: `%date()`, `%time()`
//! - Метаданные: `%version()`, `%filename()`, `%dirpath()`
//! - Логические: `%true()`, `%false()`, `%not(expr)`
//! - Строковые: `%strlen(s)`, `%substr(s, start, len)`, `%upper(s)`, `%lower(s)`,
//!              `%strpos(s, needle)`, `%string(x)`, `%newline()`
//! - Числовые: `%intval(s)`, `%floor(x)`, `%ceil(x)`, `%abs(x)`

use regex::Regex;
use std::sync::LazyLock;

/// Регулярные выражения для парсинга функций с аргументами
static RE_STRLEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"%strlen\("([^"]*)"\)"#).unwrap());
static RE_UPPER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"%upper\("([^"]*)"\)"#).unwrap());
static RE_LOWER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"%lower\("([^"]*)"\)"#).unwrap());
static RE_SUBSTR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"%substr\("([^"]*)",\s*(\d+)(?:,\s*(\d+))?\)"#).unwrap());
static RE_STRPOS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"%strpos\("([^"]*)",\s*"([^"]*)"\)"#).unwrap());
static RE_STRING: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"%string\(([^)]+)\)").unwrap());
static RE_INTVAL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"%intval\("?([^")]+)"?\)"#).unwrap());
static RE_FLOOR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"%floor\(([^)]+)\)").unwrap());
static RE_CEIL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"%ceil\(([^)]+)\)").unwrap());
static RE_ABS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"%abs\(([^)]+)\)").unwrap());
static RE_NOT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"%not\(([^)]+)\)").unwrap());

/// Обрабатывает builtin функции в строке
pub fn process_builtins(line: &str) -> String {
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

    // %tab()
    result = result.replace("%tab()", "\t");

    // === Строковые функции ===

    // %strlen("string")
    result = process_strlen(&result);

    // %upper("string")
    result = process_upper(&result);

    // %lower("string")
    result = process_lower(&result);

    // %substr("string", start, len)
    result = process_substr(&result);

    // %strpos("string", "needle")
    result = process_strpos(&result);

    // %string(value)
    result = process_string(&result);

    // === Числовые функции ===

    // %intval("42")
    result = process_intval(&result);

    // %floor(3.7)
    result = process_floor(&result);

    // %ceil(3.2)
    result = process_ceil(&result);

    // %abs(-5)
    result = process_abs(&result);

    // === Логические функции ===

    // %not(expr)
    result = process_not(&result);

    result
}

// === Строковые функции ===

/// %strlen("string") -> длина строки
fn process_strlen(input: &str) -> String {
    RE_STRLEN
        .replace_all(input, |caps: &regex::Captures| {
            let s = &caps[1];
            s.chars().count().to_string()
        })
        .to_string()
}

/// %upper("string") -> СТРОКА В ВЕРХНЕМ РЕГИСТРЕ
fn process_upper(input: &str) -> String {
    RE_UPPER
        .replace_all(input, |caps: &regex::Captures| caps[1].to_uppercase())
        .to_string()
}

/// %lower("STRING") -> строка в нижнем регистре
fn process_lower(input: &str) -> String {
    RE_LOWER
        .replace_all(input, |caps: &regex::Captures| caps[1].to_lowercase())
        .to_string()
}

/// %substr("string", start, len) -> подстрока
fn process_substr(input: &str) -> String {
    RE_SUBSTR
        .replace_all(input, |caps: &regex::Captures| {
            let s = &caps[1];
            let start: usize = caps[2].parse().unwrap_or(0);
            let len: Option<usize> = caps.get(3).and_then(|m| m.as_str().parse().ok());

            let chars: Vec<char> = s.chars().collect();
            if start >= chars.len() {
                return String::new();
            }

            match len {
                Some(l) => chars.iter().skip(start).take(l).collect(),
                None => chars.iter().skip(start).collect(),
            }
        })
        .to_string()
}

/// %strpos("haystack", "needle") -> позиция или -1
fn process_strpos(input: &str) -> String {
    RE_STRPOS
        .replace_all(input, |caps: &regex::Captures| {
            let haystack = &caps[1];
            let needle = &caps[2];

            match haystack.find(needle) {
                Some(pos) => pos.to_string(),
                None => "-1".to_string(),
            }
        })
        .to_string()
}

/// %string(value) -> преобразование в строку
fn process_string(input: &str) -> String {
    RE_STRING
        .replace_all(input, |caps: &regex::Captures| {
            let value = caps[1].trim();
            // Убираем кавычки если есть
            let value = value.trim_matches('"');
            format!("\"{}\"", value)
        })
        .to_string()
}

// === Числовые функции ===

/// %intval("42") -> 42
fn process_intval(input: &str) -> String {
    RE_INTVAL
        .replace_all(input, |caps: &regex::Captures| {
            let value = caps[1].trim().trim_matches('"');
            value.parse::<i64>().unwrap_or(0).to_string()
        })
        .to_string()
}

/// %floor(3.7) -> 3
fn process_floor(input: &str) -> String {
    RE_FLOOR
        .replace_all(input, |caps: &regex::Captures| {
            let value = caps[1].trim();
            value
                .parse::<f64>()
                .map(|v| v.floor() as i64)
                .unwrap_or(0)
                .to_string()
        })
        .to_string()
}

/// %ceil(3.2) -> 4
fn process_ceil(input: &str) -> String {
    RE_CEIL
        .replace_all(input, |caps: &regex::Captures| {
            let value = caps[1].trim();
            value
                .parse::<f64>()
                .map(|v| v.ceil() as i64)
                .unwrap_or(0)
                .to_string()
        })
        .to_string()
}

/// %abs(-5) -> 5
fn process_abs(input: &str) -> String {
    RE_ABS
        .replace_all(input, |caps: &regex::Captures| {
            let value = caps[1].trim();
            // Пробуем как целое, потом как дробное
            if let Ok(v) = value.parse::<i64>() {
                v.abs().to_string()
            } else if let Ok(v) = value.parse::<f64>() {
                v.abs().to_string()
            } else {
                "0".to_string()
            }
        })
        .to_string()
}

// === Логические функции ===

/// %not(expr) -> инвертирование логического значения
fn process_not(input: &str) -> String {
    RE_NOT
        .replace_all(input, |caps: &regex::Captures| {
            let value = caps[1].trim().to_lowercase();
            match value.as_str() {
                "true" | "1" => "false".to_string(),
                "false" | "0" | "" => "true".to_string(),
                _ => "false".to_string(), // непустые значения считаются true
            }
        })
        .to_string()
}

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
