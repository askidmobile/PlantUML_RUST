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
    let value = value_part.trim().trim_matches('"');
    let key = format!("${}", name);

    if default_only && ctx.variables.contains_key(&key) {
        return Ok(());
    }

    ctx.set_variable(key, value.to_string());

    Ok(())
}

/// Подставляет переменные в строку
pub fn substitute(line: &str, variables: &IndexMap<String, String>) -> String {
    let mut result = line.to_string();

    for (name, value) in variables {
        // Подстановка $name и ${name}
        result = result.replace(name, value);

        // Также подстановка ${name}
        let braced = format!("${{{}}}", name.trim_start_matches('$'));
        result = result.replace(&braced, value);
    }

    result
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
