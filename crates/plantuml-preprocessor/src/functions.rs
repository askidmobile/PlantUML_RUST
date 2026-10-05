//! Пользовательские функции и процедуры препроцессора.
//!
//! Поддержка:
//! - `!function $name($args)` ... `!endfunction`
//! - `!procedure $name($args)` ... `!endprocedure`
//! - `!return value`

use indexmap::IndexMap;

/// Тип callable: функция или процедура
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallableKind {
    /// Функция возвращает значение через !return
    Function,
    /// Процедура выводит текст напрямую
    Procedure,
}

/// Определение функции или процедуры
#[derive(Debug, Clone)]
pub struct UserCallable {
    /// Имя функции/процедуры (с $)
    pub name: String,
    /// Тип: функция или процедура
    pub kind: CallableKind,
    /// Список параметров (имена с $)
    pub parameters: Vec<String>,
    /// Тело функции (строки между !function и !endfunction)
    pub body: Vec<String>,
    /// Объявлена как `!unquoted`: кавычки вокруг аргументов снимаются.
    ///
    /// PlantUML использует это в стандартной библиотеке: вызов
    /// `Person(a, "Имя")` подставляет аргумент внутрь уже закавыченной
    /// строки макроса, и без снятия кавычек получается `"=="Имя""`.
    pub unquoted: bool,
}

impl UserCallable {
    /// Создаёт новую функцию
    pub fn function(name: impl Into<String>, parameters: Vec<String>) -> Self {
        Self {
            name: name.into(),
            kind: CallableKind::Function,
            parameters,
            body: Vec::new(),
            unquoted: false,
        }
    }

    /// Создаёт новую процедуру
    pub fn procedure(name: impl Into<String>, parameters: Vec<String>) -> Self {
        Self {
            name: name.into(),
            kind: CallableKind::Procedure,
            parameters,
            body: Vec::new(),
            unquoted: false,
        }
    }

    /// Добавляет строку в тело
    pub fn add_line(&mut self, line: impl Into<String>) {
        self.body.push(line.into());
    }

    /// Вызывает функцию/процедуру с аргументами
    ///
    /// Возвращает (output_lines, return_value)
    pub fn call(&self, args: &[String]) -> (Vec<String>, Option<String>) {
        // Создаём локальный контекст переменных
        let mut local_vars: IndexMap<String, String> = IndexMap::new();

        // Связываем параметры с аргументами
        for (i, param) in self.parameters.iter().enumerate() {
            let mut value = args.get(i).cloned().unwrap_or_default();
            // `!unquoted`: снимаем обрамляющие кавычки, иначе подстановка
            // внутрь закавыченной строки макроса даёт `"=="Имя""`.
            if self.unquoted {
                let trimmed = value.trim();
                if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
                    value = trimmed[1..trimmed.len() - 1].to_string();
                }
            }
            local_vars.insert(param.clone(), value);
        }

        let mut output = Vec::new();
        let mut return_value: Option<String> = None;

        for line in &self.body {
            let trimmed = line.trim();

            // Обработка !return
            if let Some(rest) = trimmed.strip_prefix("!return ") {
                let value = substitute_local(rest.trim(), &local_vars);
                return_value = Some(value);
                break; // !return завершает выполнение
            }

            // Обработка локального присваивания !$var = value
            if trimmed.starts_with("!$") {
                if let Some((var, val)) = trimmed[1..].split_once('=') {
                    let var_name = var.trim().to_string();
                    let val_str = substitute_local(val.trim().trim_matches('"'), &local_vars);
                    local_vars.insert(var_name, val_str);
                    continue;
                }
            }

            // Подстановка переменных и вывод
            let processed = substitute_local(line, &local_vars);
            output.push(processed);
        }

        (output, return_value)
    }
}

/// Подставляет локальные переменные в строку
fn substitute_local(line: &str, vars: &IndexMap<String, String>) -> String {
    let mut result = line.to_string();

    for (name, value) in vars {
        // Подстановка $name
        result = result.replace(name, value);

        // Подстановка ${name}
        let braced = format!("${{{}}}", name.trim_start_matches('$'));
        result = result.replace(&braced, value);
    }

    result
}

/// Парсит определение функции/процедуры
///
/// Формат: `$name($arg1, $arg2, ...)` или `$name()`
pub fn parse_callable_definition(def: &str) -> Option<(String, Vec<String>)> {
    let def = def.trim();

    // Находим имя и список параметров
    let paren_start = def.find('(')?;
    let paren_end = def.rfind(')')?;

    if paren_start >= paren_end {
        return None;
    }

    let name = def[..paren_start].trim().to_string();
    let params_str = &def[paren_start + 1..paren_end];

    let parameters: Vec<String> = if params_str.trim().is_empty() {
        Vec::new()
    } else {
        params_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };

    Some((name, parameters))
}

/// Парсит вызов функции/процедуры
///
/// Формат: `$name(arg1, arg2, ...)` или `$name()`
pub fn parse_callable_call(call: &str) -> Option<(String, Vec<String>)> {
    let call = call.trim();

    // Должно начинаться с $
    if !call.starts_with('$') {
        return None;
    }

    let paren_start = call.find('(')?;
    let paren_end = call.rfind(')')?;

    if paren_start >= paren_end {
        return None;
    }

    let name = call[..paren_start].trim().to_string();
    let args_str = &call[paren_start + 1..paren_end];

    let args: Vec<String> = if args_str.trim().is_empty() {
        Vec::new()
    } else {
        // Простой парсинг - по запятым (не учитывает вложенные вызовы)
        args_str
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .collect()
    };

    Some((name, args))
}

/// Ищет вызовы функций в строке и возвращает позиции.
///
/// Позиции — БАЙТОВЫЕ, а не в символах: вызывающий код режет строку через
/// `&line[..start]`. Раньше здесь собирался `Vec<char>` и наружу отдавались
/// индексы в символах, поэтому любая кириллица ПЕРЕД вызовом процедуры
/// сдвигала границы и приводила к панике «byte index N is not a char
/// boundary».
pub fn find_function_calls(line: &str) -> Vec<(usize, usize, String, Vec<String>)> {
    let mut calls = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    // Байтовое смещение каждого символа: chars[k] начинается с byte_offsets[k]
    let byte_offsets: Vec<usize> = line.char_indices().map(|(b, _)| b).collect();
    let to_byte =
        |char_idx: usize| -> usize { byte_offsets.get(char_idx).copied().unwrap_or(line.len()) };
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '$' {
            // Потенциальный вызов функции
            let start = i;
            i += 1;

            // Читаем имя
            let mut name = String::from("$");
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                name.push(chars[i]);
                i += 1;
            }

            // Проверяем наличие (
            if i < chars.len() && chars[i] == '(' {
                let mut depth = 1;
                i += 1;

                // Ищем закрывающую скобку
                while i < chars.len() && depth > 0 {
                    if chars[i] == '(' {
                        depth += 1;
                    } else if chars[i] == ')' {
                        depth -= 1;
                    }
                    i += 1;
                }

                if depth == 0 {
                    let call_str: String = chars[start..i].iter().collect();
                    if let Some((_, args)) = parse_callable_call(&call_str) {
                        calls.push((to_byte(start), to_byte(i), name, args));
                    }
                }
            }
        } else {
            i += 1;
        }
    }

    calls
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_callable_definition() {
        let (name, params) = parse_callable_definition("$add($a, $b)").unwrap();
        assert_eq!(name, "$add");
        assert_eq!(params, vec!["$a", "$b"]);
    }

    #[test]
    fn test_parse_callable_definition_no_params() {
        let (name, params) = parse_callable_definition("$greet()").unwrap();
        assert_eq!(name, "$greet");
        assert!(params.is_empty());
    }

    #[test]
    fn test_parse_callable_call() {
        let (name, args) = parse_callable_call("$add(1, 2)").unwrap();
        assert_eq!(name, "$add");
        assert_eq!(args, vec!["1", "2"]);
    }

    #[test]
    fn test_function_call() {
        let mut func = UserCallable::function("$add", vec!["$a".to_string(), "$b".to_string()]);
        func.add_line("!return $a + $b");

        let (output, ret) = func.call(&["10".to_string(), "20".to_string()]);
        assert!(output.is_empty());
        assert_eq!(ret, Some("10 + 20".to_string()));
    }

    #[test]
    fn test_procedure_call() {
        let mut proc = UserCallable::procedure("$box", vec!["$text".to_string()]);
        proc.add_line("rectangle \"$text\" {");
        proc.add_line("}");

        let (output, ret) = proc.call(&["Hello".to_string()]);
        assert_eq!(output.len(), 2);
        assert_eq!(output[0], "rectangle \"Hello\" {");
        assert!(ret.is_none());
    }

    #[test]
    fn test_find_function_calls() {
        let calls = find_function_calls("result = $add(1, 2) + $mul(3, 4)");
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].2, "$add");
        assert_eq!(calls[1].2, "$mul");
    }

    #[test]
    fn test_local_variable_in_function() {
        let mut func = UserCallable::function("$double", vec!["$x".to_string()]);
        func.add_line("!$result = $x$x");
        func.add_line("!return $result");

        let (_, ret) = func.call(&["5".to_string()]);
        assert_eq!(ret, Some("55".to_string()));
    }
}
