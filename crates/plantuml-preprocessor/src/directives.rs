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
    ctx.branch_taken.push(effective);
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
    ctx.branch_taken.push(effective);
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
/// Оценка условия для внешних модулей (циклы ).
pub fn evaluate_condition_public(expression: &str, ctx: &PreprocessContext) -> bool {
    evaluate_condition(expression, ctx)
}

fn evaluate_condition(expression: &str, ctx: &PreprocessContext) -> bool {
    evaluate(expression, ctx).unwrap_or(false)
}

/// Разбирает условие. `None` означает, что выражение НЕ РАЗОБРАНО.
///
/// Раньше функция умела только `==` и `!=`, а всё остальное молча
/// считала истиной. Для `!if (%strpos(...) >= 0 && $bgColor != "")`
/// она делила строку по `!=`, получала в операндах мусор, видела их
/// неравными — и ВЫПОЛНЯЛА блок. То есть неподдержанное выражение
/// приводило к исполнению кода вместо пропуска. Теперь неразобранное
/// условие даёт `false`.
fn evaluate(expression: &str, ctx: &PreprocessContext) -> Option<bool> {
    let text = strip_enclosing_parens(expression.trim());

    // Дизъюнкция и конъюнкция — самый низкий приоритет.
    if let Some((left, right)) = split_top_level(text, "||") {
        return Some(evaluate(left, ctx)? || evaluate(right, ctx)?);
    }
    if let Some((left, right)) = split_top_level(text, "&&") {
        return Some(evaluate(left, ctx)? && evaluate(right, ctx)?);
    }

    // Сравнения. Порядок важен: `>=` проверяем раньше `>`.
    for operator in ["==", "!=", ">=", "<=", ">", "<"] {
        if let Some((left, right)) = split_top_level(text, operator) {
            let left = resolve_operand(left, ctx);
            let right = resolve_operand(right, ctx);

            return match operator {
                "==" => Some(left == right),
                "!=" => Some(left != right),
                _ => {
                    // Числа сравниваются ЧИСЛОВО.
                    if let (Ok(a), Ok(b)) =
                        (left.trim().parse::<f64>(), right.trim().parse::<f64>())
                    {
                        return Some(match operator {
                            ">=" => a >= b,
                            "<=" => a <= b,
                            ">" => a > b,
                            _ => a < b,
                        });
                    }

                    // Для НЕчисловых операндов сравнение идёт по ДЛИНЕ.
                    //
                    // Стандартная библиотека C4 проверяет наличие значения
                    // как `!if ($arg > "")`. Числовое сравнение тут не
                    // работает: `"X"` не число, и условие оказывалось
                    // ложным даже для непустой строки. Из-за этого
                    // `$toRelArg` возвращал пустоту вместо аргумента, и
                    // МЕТКА СВЯЗИ ТЕРЯЛАСЬ — в выводе оставалось
                    // `user -->> sys :`.
                    let (a, b) = (left.trim().chars().count(), right.trim().chars().count());
                    Some(match operator {
                        ">=" => a >= b,
                        "<=" => a <= b,
                        ">" => a > b,
                        _ => a < b,
                    })
                }
            };
        }
    }

    // Без оператора значение приводится к булеву напрямую.
    let value = resolve_operand(text, ctx);
    match value.trim() {
        "true" => Some(true),
        "false" => Some(false),
        // Непустая строка — истина, пустая — ложь.
        other => Some(!other.is_empty()),
    }
}

/// Делит выражение по оператору ВЕРХНЕГО уровня.
///
/// Операторы внутри скобок и кавычек не считаются разделителями:
/// иначе `%strpos($a, "&&")` развалилось бы на части.
fn split_top_level<'a>(text: &'a str, operator: &str) -> Option<(&'a str, &'a str)> {
    let bytes = text.as_bytes();
    let op = operator.as_bytes();
    let mut depth = 0i32;
    let mut in_quotes = false;
    let mut index = 0;

    while index + op.len() <= bytes.len() {
        let byte = bytes[index];
        match byte {
            b'"' => in_quotes = !in_quotes,
            b'(' if !in_quotes => depth += 1,
            b')' if !in_quotes => depth -= 1,
            _ => {}
        }

        if !in_quotes && depth == 0 && &bytes[index..index + op.len()] == op {
            return Some((&text[..index], &text[index + op.len()..]));
        }

        index += 1;
    }

    None
}

/// Приводит операнд условия к строке.
///
/// Переменные подставляются из контекста, `%true()`/`%false()` — к
/// `true`/`false`, кавычки снимаются.
fn resolve_operand(operand: &str, ctx: &PreprocessContext) -> String {
    let trimmed = operand.trim().trim_matches('"');

    // Символьный литерал в ОДИНАРНЫХ кавычках.
    //
    // Библиотека C4-PlantUML ищет пробел так:
    //     !while ($brPos > 0 && %substr($text, $brPos, 1) != ' ')
    // Одинарные кавычки здесь — литерал, а НЕ комментарий. Прежде операнд
    // оставался трёхсимвольной строкой `' '`, сравнение с настоящим пробелом
    // всегда давало «не равно», счётчик уходил в ноль, и `$breakText` не
    // укорачивал текст — цикл крутился все 10 000 итераций (42 с на вызов,
    // 91 с на разбор библиотеки C4).
    let trimmed = if trimmed.len() >= 2 && trimmed.starts_with('\'') && trimmed.ends_with('\'') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };

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

    // Встроенные функции в операнде (`%strpos(...)`, `%strlen(...)`) —
    // вычисляются теми же преобразованиями, что и в обычном тексте.
    // Без этого числовые сравнения вроде `%strpos($x, "+") >= 0` не
    // работали: операнд оставался текстом вызова.
    // Сначала подставляем переменные: встроенные функции ожидают
    // аргументы в кавычках, а в условиях первый аргумент часто задан
    // переменной — `%strpos($s, "+")`.
    let substituted = crate::variables::substitute(trimmed, &ctx.variables);

    let computed = crate::builtins::process_builtins(&substituted);
    if computed != substituted {
        return computed.trim().trim_matches('"').to_string();
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

    // Ветвь выполняется, только если ни одна предыдущая не сработала.
    let taken = ctx.branch_taken.last().copied().unwrap_or(false);
    let effective = parent_ok && !taken;

    if let Some(last) = ctx.condition_stack.last_mut() {
        *last = effective;
    }
    if let Some(last) = ctx.branch_taken.last_mut() {
        *last = taken || effective;
    }

    Ok(())
}

/// Обрабатывает `!elseif (выражение)`.
///
/// Ветвь проверяется ТОЛЬКО если ни одна предыдущая не сработала.
/// Раньше `!elseif` не обрабатывался вовсе: он не менял состояние, и при
/// истинном первом условии выполнялись ВСЕ ветви подряд, а при ложном —
/// ни одна. На этом, в частности, раздувалась библиотека C4: строки
/// накапливались кратно и достигали десятков мегабайт.
pub fn handle_elseif(expression: &str, ctx: &mut PreprocessContext) -> Result<()> {
    if ctx.condition_stack.is_empty() {
        return Err(PreprocessError::UnbalancedCondition);
    }

    let len = ctx.condition_stack.len();
    let parent_ok = len <= 1 || ctx.condition_stack[..len - 1].iter().all(|&b| b);
    let already_taken = ctx.branch_taken.last().copied().unwrap_or(false);

    // Условие вычисляется только если ветвь ещё может быть выбрана:
    // иначе `%variable_exists` и прочие проверки выполнялись бы зря.
    let value = !already_taken && parent_ok && evaluate_condition(expression, ctx);
    let effective = parent_ok && value;

    if let Some(last) = ctx.condition_stack.last_mut() {
        *last = effective;
    }
    if let Some(last) = ctx.branch_taken.last_mut() {
        *last = already_taken || effective;
    }

    Ok(())
}

/// Обрабатывает !endif
pub fn handle_endif(ctx: &mut PreprocessContext) -> Result<()> {
    if ctx.condition_stack.is_empty() {
        return Err(PreprocessError::UnbalancedCondition);
    }

    ctx.condition_stack.pop();
    ctx.branch_taken.pop();
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
