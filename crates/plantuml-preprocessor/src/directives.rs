//! Обработка директив препроцессора

use crate::{PreprocessContext, PreprocessError, Result};

/// Обрабатывает `!define`.
///
/// Поддерживаются обе формы PlantUML:
///
/// - простая подстановка: `!define NAME value` — значение подставляется
///   вместо имени при упоминании;
/// - **макрос с параметрами**: `!define NAME(a, b) тело` — вызов
///   `NAME(x, y)` раскрывается в тело с подстановкой аргументов.
///
/// Вторая форма — основа стандартной библиотеки: `C4`, `tupadr3` и `office`
/// объявляют свои элементы именно так
/// (`!define Person(e_alias, e_label) rectangle "..." as e_alias`).
/// Без её поддержки содержимое stdlib подставлялось, но вызовы макросов
/// оставались в тексте как есть, и парсер падал на незнакомом синтаксисе.
pub fn handle_define(rest: &str, ctx: &mut PreprocessContext) -> Result<()> {
    if !ctx.should_output() {
        return Ok(());
    }

    let rest = rest.trim();

    // Макрос с параметрами: имя(...)  тело
    if let Some(paren_open) = rest.find('(') {
        if let Some(paren_close) = find_matching_paren(rest, paren_open) {
            let name = rest[..paren_open].trim();
            // Имя макроса не должно содержать пробелов: иначе это не макрос,
            // а обычное значение, внутри которого встретилась скобка
            if !name.is_empty() && !name.contains(char::is_whitespace) {
                let params_str = &rest[paren_open + 1..paren_close];
                let parameters: Vec<String> = if params_str.trim().is_empty() {
                    Vec::new()
                } else {
                    params_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                };
                let body = rest[paren_close + 1..].trim().to_string();

                ctx.define_macro(name, parameters, body);
                return Ok(());
            }
        }
    }

    // Простой define: !define NAME или !define NAME value
    if let Some((name, value)) = rest.split_once(' ') {
        ctx.set_variable(name.trim().to_string(), value.trim().to_string());
    } else {
        // Просто определяем как пустую строку
        ctx.set_variable(rest.to_string(), String::new());
    }

    Ok(())
}

/// Находит позицию закрывающей скобки, соответствующей открывающей.
///
/// Учитывает вложенность: в теле макроса могут быть свои скобки.
fn find_matching_paren(text: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (i, c) in text.char_indices().skip_while(|(i, _)| *i < open) {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Обрабатывает !undef
pub fn handle_undef(name: &str, ctx: &mut PreprocessContext) {
    if ctx.should_output() {
        ctx.variables.shift_remove(name);
    }
}

/// Обрабатывает !ifdef / !ifndef
pub fn handle_ifdef(name: &str, ctx: &mut PreprocessContext, is_ifdef: bool) {
    let defined = ctx.is_defined(name);
    let condition = if is_ifdef { defined } else { !defined };

    // Если мы уже внутри ложного условия, вложенное условие тоже ложно
    let effective = ctx.should_output() && condition;

    ctx.condition_stack.push(effective);
    ctx.condition_depth += 1;
}

/// Обрабатывает `!if (выражение)`.
///
/// PlantUML поддерживает не только `!ifdef`, но и полноценное условие:
///
/// ```text
/// !if ($ARCH_LOCAL == %true())
///     !include локальный.puml
/// !else
///     !include <stdlib/путь>
/// !endif
/// ```
///
/// Раньше `!if` не обрабатывался вовсе: строка попадала в вывод как
/// обычный текст, а `!endif` затем сообщал о несбалансированных
/// условиях. Именно поэтому библиотека Archimate не подключалась.
pub fn handle_if(expression: &str, ctx: &mut PreprocessContext) {
    let value = evaluate_condition(expression, ctx);
    let effective = ctx.should_output() && value;

    ctx.condition_stack.push(effective);
    ctx.condition_depth += 1;
}

/// Снимает обрамляющие скобки, только если они парные и охватывают всё.
///
/// Прежняя версия срезала первую `(` и последнюю `)` БЕЗУСЛОВНО, из-за
/// чего выражение `%variable_exists("X")` теряло закрывающую скобку и
/// разбор встроенной функции ломался. На этом падал C4: он выбирает
/// между относительным и сетевым include именно такой проверкой.
fn strip_enclosing_parens(text: &str) -> &str {
    let mut current = text;

    loop {
        let Some(inner) = current.strip_prefix('(').and_then(|r| r.strip_suffix(')')) else {
            return current;
        };

        // Скобки должны быть парными внутри, иначе внешняя не «обрамляет».
        let mut depth = 0i32;
        let mut balanced = true;
        for c in inner.chars() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth < 0 {
                        balanced = false;
                        break;
                    }
                }
                _ => {}
            }
        }

        if !balanced || depth != 0 {
            return current;
        }

        current = inner.trim();
    }
}

/// Вычисляет условие `!if` в булево значение.
///
/// Поддерживаются сравнения `==` и `!=` над переменными (`$имя`),
/// логическими литералами `%true()`/`%false()` и строками. Этого
/// достаточно для условий в стандартной библиотеке.
fn evaluate_condition(expression: &str, ctx: &PreprocessContext) -> bool {
    let text = strip_enclosing_parens(expression.trim());

    for (operator, negate) in [("==", false), ("!=", true)] {
        if let Some((left, right)) = text.split_once(operator) {
            let left = resolve_operand(left.trim(), ctx);
            let right = resolve_operand(right.trim(), ctx);
            let equal = left == right;
            return if negate { !equal } else { equal };
        }
    }

    // Без оператора значение приводится к булеву напрямую.
    resolve_operand(text, ctx) == "true"
}

/// Приводит операнд условия к строке.
///
/// Переменные подставляются из контекста, `%true()`/`%false()` — к
/// `true`/`false`, кавычки снимаются.
fn resolve_operand(operand: &str, ctx: &PreprocessContext) -> String {
    let trimmed = operand.trim().trim_matches('"');

    match trimmed {
        "%true()" | "true" => return "true".to_string(),
        "%false()" | "false" => return "false".to_string(),
        _ => {}
    }

    // `%function_exists("ИМЯ")` — объявлена ли такая функция или процедура.
    // Библиотека C4 выбирает поведение по наличию необязательных функций.
    if let Some(inner) = trimmed
        .strip_prefix("%function_exists(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let name = inner.trim().trim_matches('"');
        let known =
            ctx.get_callable(name).is_some() || ctx.get_callable(&format!("${name}")).is_some();
        return if known { "true" } else { "false" }.to_string();
    }

    // `%variable_exists("ИМЯ")` — встроенная проверка наличия переменной.
    // Нужна стандартной библиотеке: C4 выбирает относительный include
    // именно так.
    if let Some(inner) = trimmed
        .strip_prefix("%variable_exists(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let name = inner.trim().trim_matches('"');
        let exists = ctx.variables.contains_key(name)
            || ctx.variables.contains_key(&format!("${name}"))
            || ctx.is_defined(name);
        return if exists { "true" } else { "false" }.to_string();
    }

    if trimmed.starts_with('$') {
        // Переменные хранятся ВМЕСТЕ со знаком `$` — так их записывает
        // `handle_variable_assignment`. Искать по имени без `$` было
        // ошибкой: условие всегда получалось ложным.
        if let Some(value) = ctx.get_variable(trimmed) {
            let value = value.trim().trim_matches('"');
            return match value {
                "%true()" | "true" => "true".to_string(),
                "%false()" | "false" => "false".to_string(),
                other => other.to_string(),
            };
        }
    }

    trimmed.to_string()
}

/// Обрабатывает !else
pub fn handle_else(ctx: &mut PreprocessContext) -> Result<()> {
    if ctx.condition_stack.is_empty() {
        return Err(PreprocessError::UnbalancedCondition);
    }

    // Проверяем родительское условие до мутации
    let len = ctx.condition_stack.len();
    let parent_ok = len <= 1 || ctx.condition_stack[..len - 1].iter().all(|&b| b);

    // Инвертируем текущее условие если родитель true
    if parent_ok {
        if let Some(last) = ctx.condition_stack.last_mut() {
            *last = !*last;
        }
    }

    Ok(())
}

/// Обрабатывает !endif
pub fn handle_endif(ctx: &mut PreprocessContext) -> Result<()> {
    if ctx.condition_stack.is_empty() {
        return Err(PreprocessError::UnbalancedCondition);
    }

    ctx.condition_stack.pop();
    ctx.condition_depth = ctx.condition_depth.saturating_sub(1);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Макрос с параметрами раскрывается с подстановкой аргументов.
    ///
    /// На этой форме держится вся стандартная библиотека: C4, tupadr3 и
    /// office объявляют свои элементы как
    /// `!define Person(e_alias, e_label) rectangle "..." as e_alias`.
    /// Раньше `!define` с параметрами трактовался как обычная переменная,
    /// поэтому вызовы `Person(user, "Имя")` не раскрывались.
    #[test]
    fn test_define_macro_with_parameters() {
        let mut ctx = PreprocessContext::new();
        handle_define("GREET(name) participant name", &mut ctx).unwrap();

        let def = ctx.macros.get("GREET").expect("макрос не объявлен");
        assert_eq!(def.parameters, vec!["name"]);
        assert_eq!(def.body, "participant name");

        assert_eq!(ctx.expand_macros("GREET(Alice)"), "participant Alice");
    }

    /// Несколько параметров подставляются по порядку.
    #[test]
    fn test_macro_expands_multiple_parameters() {
        let mut ctx = PreprocessContext::new();
        handle_define("BOX(a, b) rectangle \"a\" as b", &mut ctx).unwrap();
        assert_eq!(ctx.expand_macros("BOX(X, Y)"), "rectangle \"X\" as Y");
    }

    /// Параметр сразу после литерала `\n` подставляется.
    ///
    /// В C4-макросах параметр описания идёт как `...\\n\\ne_descr`, и буква
    /// `n` не должна считаться частью имени.
    #[test]
    fn test_macro_parameter_after_escape() {
        let mut ctx = PreprocessContext::new();
        handle_define("M(a, b) rect \"a\\nb\"", &mut ctx).unwrap();
        assert_eq!(ctx.expand_macros("M(X, Z)"), "rect \"X\\nZ\"");
    }

    /// Отсутствующий аргумент подставляется пустой строкой.
    ///
    /// PlantUML допускает вызов макроса с меньшим числом аргументов.
    #[test]
    fn test_macro_missing_argument_is_empty() {
        let mut ctx = PreprocessContext::new();
        handle_define("M(a, b) [a|b]", &mut ctx).unwrap();
        assert_eq!(ctx.expand_macros("M(X)"), "[X|]");
    }

    /// Запятая внутри кавычек не разрывает аргумент.
    ///
    /// Сами кавычки при подстановке СНИМАЮТСЯ — проверено на plantuml.com:
    /// `!define M(a) class "PRE a POST"` даёт `PRE Имя POST` и при вызове
    /// `M("Имя")`, и при `M(Имя)`. Раньше тест ожидал сохранения кавычек,
    /// что фиксировало неверное поведение и мешало работать стандартной
    /// библиотеке C4.
    #[test]
    fn test_macro_args_respect_quotes() {
        let mut ctx = PreprocessContext::new();
        handle_define("M(a, b) a + b", &mut ctx).unwrap();
        assert_eq!(ctx.expand_macros("M(\"x, y\", Z)"), "x, y + Z");
        // Аргумент без кавычек передаётся как есть
        assert_eq!(ctx.expand_macros("M(x, Z)"), "x + Z");
    }

    /// Имя параметра не заменяется внутри более длинного имени.
    #[test]
    fn test_macro_word_boundaries() {
        let mut ctx = PreprocessContext::new();
        handle_define("M(a) [a] and [abc] and [a_x]", &mut ctx).unwrap();
        let out = ctx.expand_macros("M(V)");
        assert_eq!(out, "[V] and [abc] and [a_x]");
    }

    /// Простой `!define` без скобок работает как раньше.
    #[test]
    fn test_simple_define_still_works() {
        let mut ctx = PreprocessContext::new();
        handle_define("DEBUG true", &mut ctx).unwrap();
        assert_eq!(ctx.get_variable("DEBUG"), Some(&"true".to_string()));

        let mut ctx2 = PreprocessContext::new();
        handle_define("FLAG", &mut ctx2).unwrap();
        assert_eq!(ctx2.get_variable("FLAG"), Some(&String::new()));
    }

    /// Взаимная рекурсия макросов не зацикливает препроцессор.
    #[test]
    fn test_macro_recursion_is_bounded() {
        let mut ctx = PreprocessContext::new();
        handle_define("A(x) B(x)", &mut ctx).unwrap();
        handle_define("B(x) A(x)", &mut ctx).unwrap();
        // Главное — не зависнуть и вернуть строку
        let out = ctx.expand_macros("A(V)");
        assert!(out.contains('V') || !out.is_empty());
    }

    #[test]
    fn test_define() {
        let mut ctx = PreprocessContext::new();
        handle_define("DEBUG true", &mut ctx).unwrap();
        assert_eq!(ctx.get_variable("DEBUG"), Some(&"true".to_string()));
    }

    #[test]
    fn test_ifdef_defined() {
        let mut ctx = PreprocessContext::new();
        ctx.set_variable("DEBUG", "");

        handle_ifdef("DEBUG", &mut ctx, true);
        assert!(ctx.should_output());

        handle_endif(&mut ctx).unwrap();
        assert!(ctx.should_output());
    }

    #[test]
    fn test_ifdef_not_defined() {
        let mut ctx = PreprocessContext::new();

        handle_ifdef("DEBUG", &mut ctx, true);
        assert!(!ctx.should_output());

        handle_endif(&mut ctx).unwrap();
        assert!(ctx.should_output());
    }

    #[test]
    fn test_else() {
        let mut ctx = PreprocessContext::new();

        handle_ifdef("DEBUG", &mut ctx, true); // false, DEBUG не определён
        assert!(!ctx.should_output());

        handle_else(&mut ctx).unwrap();
        assert!(ctx.should_output()); // Теперь true

        handle_endif(&mut ctx).unwrap();
    }
}
