//! Обработка переменных препроцессора

use indexmap::IndexMap;

use crate::{PreprocessContext, PreprocessError, Result};

/// Обрабатывает присваивание переменной: !$var = value
pub fn handle_variable_assignment(directive: &str, ctx: &mut PreprocessContext) -> Result<()> {
    if !ctx.should_output() {
        return Ok(());
    }

    // Формат: `$name = value` либо `$name ?= value`.
    //
    // `?=` задаёт значение ТОЛЬКО если переменной ещё нет — так
    // объявляются настройки по умолчанию в стандартной библиотеке
    // (`!global $ARCH_LOCAL ?= %false()`).
    let (name_part, value_part, default_only) = match directive.split_once("?=") {
        Some((name, value)) => (name, value, true),
        None => match directive.splitn(2, '=').collect::<Vec<_>>().as_slice() {
            [name, value] => (*name, *value, false),
            _ => {
                return Err(PreprocessError::SyntaxError(format!(
                    "неверный формат присваивания: {}",
                    directive
                )))
            }
        },
    };

    let name = name_part.trim().trim_start_matches('$');

    // Правая часть — ВЫРАЖЕНИЕ, а не строка.
    //
    // PlantUML вычисляет её, включая конкатенацию через `+` и подстановку
    // переменных: `!$tagSkin = $tagSkin + "skinparam " + $name` даёт
    // склеенную строку. Прежний код сохранял текст как есть, поэтому
    // библиотеки получали в переменной сырое выражение вида
    // `"rectangle<<" + $a` — это ломало, например, C4.
    let value = crate::evaluate_concat(value_part.trim(), ctx);
    let key = format!("${}", name);

    if default_only && ctx.variables.contains_key(&key) {
        return Ok(());
    }

    ctx.set_variable(key, value.to_string());

    Ok(())
}

/// Подставляет переменные в строку
pub fn substitute(line: &str, variables: &IndexMap<String, String>) -> String {
    // ПОЧЕМУ НЕ `replace`: простая замена по подстроке портит имена,
    // которые являются префиксами других. `$element` внутри
    // `$elementSkin` превращался в `rectangleSkin`, и стандартная
    // библиотека получала мусор вместо `skinparam rectangle<<...>>`.
    //
    // Здесь имя выбирается ПО САМОМУ ДЛИННОМУ совпадению, а подстановка
    // идёт за один проход: вставленные значения повторно не сканируются.
    // Быстрый выход: без переменных и без знака `$` заменять нечего.
    if variables.is_empty() || !line.contains('$') {
        return line.to_string();
    }

    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut index = 0;
    // Буфер ключа переиспользуется: на длинных строках аллокация на
    // каждого кандидата замедляла раскрытие библиотек в разы.
    let mut key = String::with_capacity(64);

    while index < chars.len() {
        if chars[index] != '$' {
            out.push(chars[index]);
            index += 1;
            continue;
        }

        // Форма `${имя}`.
        if chars.get(index + 1) == Some(&'{') {
            if let Some(close) = (index + 2..chars.len()).find(|&k| chars[k] == '}') {
                let name: String = chars[index + 2..close].iter().collect();
                if let Some(value) = variables.get(&format!("${name}")) {
                    out.push_str(value);
                    index = close + 1;
                    continue;
                }
            }

            out.push('$');
            index += 1;
            continue;
        }

        // Форма `$имя`: берём самое длинное имя, которое есть в контексте.
        let start = index + 1;
        let mut end = start;
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        let mut replaced = false;
        for stop in (start + 1..=end).rev() {
            key.clear();
            key.push('$');
            key.extend(chars[start..stop].iter());
            if let Some(value) = variables.get(key.as_str()) {
                out.push_str(value);
                index = stop;
                replaced = true;
                break;
            }
        }

        if !replaced {
            out.push('$');
            index += 1;
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_assignment() {
        let mut ctx = PreprocessContext::new();
        handle_variable_assignment("$name = \"Alice\"", &mut ctx).unwrap();
        assert_eq!(ctx.get_variable("$name"), Some(&"Alice".to_string()));
    }

    #[test]
    fn test_substitute() {
        let mut vars = IndexMap::new();
        vars.insert("$name".to_string(), "Alice".to_string());
        vars.insert("$color".to_string(), "#FF0000".to_string());

        let result = substitute("participant $name #$color", &vars);
        assert_eq!(result, "participant Alice ##FF0000");
    }
}
