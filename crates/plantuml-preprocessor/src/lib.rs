//! # plantuml-preprocessor
//!
//! Препроцессор PlantUML для обработки директив:
//! - `!include` / `!include_once`
//! - `!define` / `!undef`
//! - `!ifdef` / `!ifndef` / `!else` / `!endif`
//! - `!$variable = value`
//! - `!function` / `!procedure`
//! - `!theme`
//! - `%date()`, `%version()` и другие builtin функции

mod builtins;
mod directives;
mod error;
mod fs_resolver;
mod functions;
mod variables;

pub use error::PreprocessError;
pub use fs_resolver::FsFileResolver;
pub use functions::{CallableKind, UserCallable};
pub use plantuml_themes::{SkinParams, Theme};

use indexmap::IndexMap;

/// Обрабатывает PlantUML исходный код (без поддержки !include)
///
/// Это удобная обёртка над `Preprocessor::new().process(source)`.
///
/// # Пример
///
/// ```rust
/// use plantuml_preprocessor::preprocess;
///
/// let source = "!define DEBUG\n!ifdef DEBUG\nDebug mode\n!endif";
/// let result = preprocess(source).unwrap();
/// assert!(result.contains("Debug mode"));
/// ```
pub fn preprocess(source: &str) -> Result<String> {
    Preprocessor::new().process(source)
}

/// Результат препроцессинга
pub type Result<T> = std::result::Result<T, PreprocessError>;

/// Трейт для разрешения путей файлов
pub trait FileResolver {
    /// Читает содержимое файла по пути
    fn read_file(&self, path: &str) -> Result<String>;

    /// Проверяет существование файла
    fn file_exists(&self, path: &str) -> bool;
}

/// Заглушка для FileResolver (не поддерживает !include)
#[derive(Debug, Default)]
pub struct NoopFileResolver;

impl FileResolver for NoopFileResolver {
    fn read_file(&self, path: &str) -> Result<String> {
        Err(PreprocessError::IncludeNotSupported(path.to_string()))
    }

    fn file_exists(&self, _path: &str) -> bool {
        false
    }
}

/// Состояние определения функции/процедуры
#[derive(Debug, Clone)]
enum DefiningCallable {
    /// Не определяем функцию/процедуру
    None,
    /// Определяем функцию
    Function(functions::UserCallable),
    /// Определяем процедуру
    Procedure(functions::UserCallable),
}

/// Контекст препроцессора
#[derive(Debug)]
pub struct PreprocessContext {
    /// Переменные
    pub variables: IndexMap<String, String>,
    /// Уже включённые файлы (для !include_once)
    pub included_files: Vec<String>,
    /// Текущий уровень вложенности условий
    pub condition_depth: usize,
    /// Активные условия (true = выполнять код)
    pub condition_stack: Vec<bool>,
    /// Пользовательские функции и процедуры
    pub callables: IndexMap<String, functions::UserCallable>,
    /// Текущее определение функции/процедуры
    defining: DefiningCallable,
    /// Текущая тема
    pub theme: Theme,
    /// SkinParam параметры
    pub skin_params: SkinParams,
}

impl Default for PreprocessContext {
    fn default() -> Self {
        Self {
            variables: IndexMap::new(),
            included_files: Vec::new(),
            condition_depth: 0,
            condition_stack: Vec::new(),
            callables: IndexMap::new(),
            defining: DefiningCallable::None,
            theme: Theme::default(),
            skin_params: SkinParams::new(),
        }
    }
}

impl PreprocessContext {
    /// Создаёт новый контекст
    pub fn new() -> Self {
        Self::default()
    }

    /// Устанавливает переменную
    pub fn set_variable(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(name.into(), value.into());
    }

    /// Получает значение переменной
    pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.variables.get(name)
    }

    /// Проверяет, определена ли переменная
    pub fn is_defined(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// Проверяет, нужно ли выполнять текущий код
    pub fn should_output(&self) -> bool {
        self.condition_stack.iter().all(|&b| b)
    }

    /// Проверяет, определяем ли мы сейчас функцию/процедуру
    pub fn is_defining_callable(&self) -> bool {
        !matches!(self.defining, DefiningCallable::None)
    }

    /// Регистрирует функцию или процедуру
    pub fn register_callable(&mut self, callable: functions::UserCallable) {
        self.callables.insert(callable.name.clone(), callable);
    }

    /// Получает функцию или процедуру по имени
    pub fn get_callable(&self, name: &str) -> Option<&functions::UserCallable> {
        self.callables.get(name)
    }

    /// Устанавливает тему по имени
    pub fn set_theme(&mut self, name: &str) -> bool {
        if let Some(theme) = Theme::by_name(name) {
            self.theme = theme;
            true
        } else {
            false
        }
    }

    /// Устанавливает skinparam
    pub fn set_skin_param(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.skin_params.set(key, value);
    }

    /// Применяет skinparams к теме
    pub fn apply_skin_params(&mut self) {
        self.skin_params.apply_to(&mut self.theme);
    }
}

/// Препроцессор PlantUML
pub struct Preprocessor<R: FileResolver = NoopFileResolver> {
    resolver: R,
}

impl Preprocessor<NoopFileResolver> {
    /// Создаёт препроцессор без поддержки !include
    pub fn new() -> Self {
        Self {
            resolver: NoopFileResolver,
        }
    }
}

impl Default for Preprocessor<NoopFileResolver> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: FileResolver> Preprocessor<R> {
    /// Создаёт препроцессор с заданным resolver'ом
    pub fn with_resolver(resolver: R) -> Self {
        Self { resolver }
    }

    /// Обрабатывает исходный код PlantUML
    pub fn process(&self, source: &str) -> Result<String> {
        let mut ctx = PreprocessContext::new();
        self.process_with_context(source, &mut ctx)
    }

    /// Обрабатывает исходный код с заданным контекстом
    pub fn process_with_context(
        &self,
        source: &str,
        ctx: &mut PreprocessContext,
    ) -> Result<String> {
        let mut output = String::new();

        for line in source.lines() {
            let trimmed = line.trim();

            // Если мы определяем функцию/процедуру, собираем тело
            if ctx.is_defining_callable() {
                if trimmed == "!endfunction" || trimmed == "!endprocedure" {
                    // Завершаем определение
                    self.finish_callable_definition(ctx)?;
                } else {
                    // Добавляем строку в тело
                    self.add_line_to_callable(line, ctx);
                }
                continue;
            }

            // Обработка директив препроцессора
            if trimmed.starts_with('!') {
                let included_content = self.process_directive_with_output(trimmed, ctx)?;
                if let Some(content) = included_content {
                    output.push_str(&content);
                }
                continue;
            }

            // Пропускаем строки, если мы внутри ложного условия
            if !ctx.should_output() {
                continue;
            }

            // Обработка skinparam
            if trimmed.starts_with("skinparam ") {
                self.handle_skinparam(trimmed, ctx);
                // Пропускаем строку (она применена к теме)
                continue;
            }

            // Подстановка переменных
            let processed = self.substitute_variables(line, ctx);

            // Обработка вызовов пользовательских функций
            let processed = self.process_function_calls(&processed, ctx);

            // Обработка builtin функций
            let processed = builtins::process_builtins(&processed);

            output.push_str(&processed);
            output.push('\n');
        }

        Ok(output)
    }

    /// Обрабатывает директиву препроцессора (без возврата контента)
    #[allow(dead_code)]
    fn process_directive(&self, line: &str, ctx: &mut PreprocessContext) -> Result<()> {
        self.process_directive_with_output(line, ctx)?;
        Ok(())
    }

    /// Обрабатывает директиву препроцессора и возвращает включённый контент (если есть)
    fn process_directive_with_output(
        &self,
        line: &str,
        ctx: &mut PreprocessContext,
    ) -> Result<Option<String>> {
        let directive = &line[1..]; // Убираем '!'

        if let Some(rest) = directive.strip_prefix("define ") {
            directives::handle_define(rest, ctx)?;
        } else if let Some(rest) = directive.strip_prefix("undef ") {
            directives::handle_undef(rest.trim(), ctx);
        } else if let Some(rest) = directive.strip_prefix("ifdef ") {
            directives::handle_ifdef(rest.trim(), ctx, true);
        } else if let Some(rest) = directive.strip_prefix("ifndef ") {
            directives::handle_ifdef(rest.trim(), ctx, false);
        } else if directive == "else" {
            directives::handle_else(ctx)?;
        } else if directive == "endif" {
            directives::handle_endif(ctx)?;
        } else if let Some(rest) = directive.strip_prefix("include ") {
            return self.handle_include(rest.trim(), ctx, false);
        } else if let Some(rest) = directive.strip_prefix("include_once ") {
            return self.handle_include(rest.trim(), ctx, true);
        } else if let Some(rest) = directive.strip_prefix("function ") {
            self.start_function_definition(rest.trim(), ctx)?;
        } else if let Some(rest) = directive.strip_prefix("procedure ") {
            self.start_procedure_definition(rest.trim(), ctx)?;
        } else if let Some(rest) = directive.strip_prefix("theme ") {
            self.handle_theme(rest.trim(), ctx)?;
        } else if directive.starts_with('$') {
            // Переменная: !$var = value
            variables::handle_variable_assignment(directive, ctx)?;
        }

        Ok(None)
    }

    /// Обрабатывает !include и возвращает обработанный контент
    fn handle_include(
        &self,
        path: &str,
        ctx: &mut PreprocessContext,
        once: bool,
    ) -> Result<Option<String>> {
        if !ctx.should_output() {
            return Ok(None);
        }

        let path = path.trim_matches(|c| c == '<' || c == '>' || c == '"');

        if once && ctx.included_files.contains(&path.to_string()) {
            return Ok(None);
        }

        let content = self.resolver.read_file(path)?;
        ctx.included_files.push(path.to_string());

        // Рекурсивная обработка включённого файла
        let processed = self.process_with_context(&content, ctx)?;

        Ok(Some(processed))
    }

    /// Подставляет переменные в строку
    fn substitute_variables(&self, line: &str, ctx: &PreprocessContext) -> String {
        variables::substitute(line, &ctx.variables)
    }

    /// Обрабатывает !theme
    fn handle_theme(&self, theme_spec: &str, ctx: &mut PreprocessContext) -> Result<()> {
        if !ctx.should_output() {
            return Ok(());
        }

        // Формат: !theme <name> [from <url>]
        // Пока поддерживаем только локальные темы
        let theme_name = theme_spec.split_whitespace().next().unwrap_or(theme_spec);

        if !ctx.set_theme(theme_name) {
            // Неизвестная тема - можно предупредить или проигнорировать
            // Пока просто игнорируем (как PlantUML)
        }

        Ok(())
    }

    /// Обрабатывает skinparam
    fn handle_skinparam(&self, line: &str, ctx: &mut PreprocessContext) {
        // Формат: skinparam <key> <value>
        let rest = line.strip_prefix("skinparam ").unwrap_or("");
        let parts: Vec<&str> = rest.splitn(2, ' ').collect();

        if parts.len() == 2 {
            let key = parts[0].trim();
            let value = parts[1].trim();
            ctx.set_skin_param(key, value);
            ctx.apply_skin_params();
        }
    }

    /// Начинает определение функции
    fn start_function_definition(&self, def: &str, ctx: &mut PreprocessContext) -> Result<()> {
        if !ctx.should_output() {
            return Ok(());
        }

        let (name, params) = functions::parse_callable_definition(def).ok_or_else(|| {
            PreprocessError::SyntaxError(format!("неверный формат определения функции: {}", def))
        })?;

        let callable = functions::UserCallable::function(name, params);
        ctx.defining = DefiningCallable::Function(callable);

        Ok(())
    }

    /// Начинает определение процедуры
    fn start_procedure_definition(&self, def: &str, ctx: &mut PreprocessContext) -> Result<()> {
        if !ctx.should_output() {
            return Ok(());
        }

        let (name, params) = functions::parse_callable_definition(def).ok_or_else(|| {
            PreprocessError::SyntaxError(format!("неверный формат определения процедуры: {}", def))
        })?;

        let callable = functions::UserCallable::procedure(name, params);
        ctx.defining = DefiningCallable::Procedure(callable);

        Ok(())
    }

    /// Добавляет строку в тело текущей функции/процедуры
    fn add_line_to_callable(&self, line: &str, ctx: &mut PreprocessContext) {
        match &mut ctx.defining {
            DefiningCallable::Function(ref mut callable)
            | DefiningCallable::Procedure(ref mut callable) => {
                callable.add_line(line);
            }
            DefiningCallable::None => {}
        }
    }

    /// Завершает определение функции/процедуры
    fn finish_callable_definition(&self, ctx: &mut PreprocessContext) -> Result<()> {
        let callable = std::mem::replace(&mut ctx.defining, DefiningCallable::None);

        match callable {
            DefiningCallable::Function(c) | DefiningCallable::Procedure(c) => {
                ctx.register_callable(c);
            }
            DefiningCallable::None => {
                return Err(PreprocessError::SyntaxError(
                    "!endfunction/!endprocedure без соответствующего !function/!procedure"
                        .to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Обрабатывает вызовы пользовательских функций в строке
    fn process_function_calls(&self, line: &str, ctx: &PreprocessContext) -> String {
        let calls = functions::find_function_calls(line);

        if calls.is_empty() {
            return line.to_string();
        }

        let mut result = line.to_string();

        // Обрабатываем вызовы в обратном порядке (чтобы не сбивались индексы)
        for (start, end, name, args) in calls.into_iter().rev() {
            if let Some(callable) = ctx.get_callable(&name) {
                let (output_lines, return_value) = callable.call(&args);

                let replacement = match callable.kind {
                    functions::CallableKind::Function => {
                        // Функция: подставляем возвращённое значение
                        return_value.unwrap_or_default()
                    }
                    functions::CallableKind::Procedure => {
                        // Процедура: подставляем вывод
                        output_lines.join("\n")
                    }
                };

                result = format!("{}{}{}", &result[..start], replacement, &result[end..]);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_preprocess() {
        let preprocessor = Preprocessor::new();
        let source = "@startuml\nAlice -> Bob\n@enduml";
        let result = preprocessor.process(source).unwrap();
        assert!(result.contains("Alice -> Bob"));
    }

    #[test]
    fn test_variable_substitution() {
        let preprocessor = Preprocessor::new();
        let source = "!$name = \"Alice\"\nparticipant $name";
        let result = preprocessor.process(source).unwrap();
        assert!(result.contains("participant Alice"));
    }

    #[test]
    fn test_ifdef() {
        let preprocessor = Preprocessor::new();
        let source = r#"
!define DEBUG
!ifdef DEBUG
debug message
!endif
"#;
        let result = preprocessor.process(source).unwrap();
        assert!(result.contains("debug message"));
    }

    #[test]
    fn test_ifndef() {
        let preprocessor = Preprocessor::new();
        let source = r#"
!ifndef RELEASE
debug mode
!endif
"#;
        let result = preprocessor.process(source).unwrap();
        assert!(result.contains("debug mode"));
    }

    #[test]
    fn test_function_definition_and_call() {
        let preprocessor = Preprocessor::new();
        let source = r#"
!function $greet($name)
!return Hello_$name
!endfunction
result: $greet("World")
"#;
        let result = preprocessor.process(source).unwrap();
        assert!(result.contains("result: Hello_World"));
    }

    #[test]
    fn test_procedure_definition_and_call() {
        let preprocessor = Preprocessor::new();
        let source = r#"
!procedure $box($text)
rectangle "$text" {
}
!endprocedure
$box("MyBox")
"#;
        let result = preprocessor.process(source).unwrap();
        assert!(result.contains("rectangle \"MyBox\""));
    }

    #[test]
    fn test_function_with_multiple_params() {
        let preprocessor = Preprocessor::new();
        let source = r#"
!function $format($prefix, $value, $suffix)
!return $prefix$value$suffix
!endfunction
output: $format("[", "test", "]")
"#;
        let result = preprocessor.process(source).unwrap();
        assert!(result.contains("output: [test]"));
    }

    #[test]
    fn test_theme_directive() {
        let preprocessor = Preprocessor::new();
        let mut ctx = PreprocessContext::new();

        let source = r#"
!theme dark
@startuml
Alice -> Bob
@enduml
"#;
        preprocessor.process_with_context(source, &mut ctx).unwrap();

        assert_eq!(ctx.theme.name, "dark");
    }

    #[test]
    fn test_theme_unknown() {
        let preprocessor = Preprocessor::new();
        let mut ctx = PreprocessContext::new();

        let source = r#"
!theme nonexistent
@startuml
@enduml
"#;
        // Неизвестная тема не должна вызывать ошибку
        let result = preprocessor.process_with_context(source, &mut ctx);
        assert!(result.is_ok());
        // Тема остаётся default
        assert_eq!(ctx.theme.name, "default");
    }

    #[test]
    fn test_skinparam() {
        let preprocessor = Preprocessor::new();
        let mut ctx = PreprocessContext::new();

        let source = r#"
skinparam backgroundColor #FF0000
@startuml
@enduml
"#;
        preprocessor.process_with_context(source, &mut ctx).unwrap();

        assert_eq!(ctx.theme.background_color.to_css(), "#FF0000");
    }

    #[test]
    fn test_theme_with_skinparam_override() {
        let preprocessor = Preprocessor::new();
        let mut ctx = PreprocessContext::new();

        let source = r#"
!theme dark
skinparam backgroundColor #00FF00
@startuml
@enduml
"#;
        preprocessor.process_with_context(source, &mut ctx).unwrap();

        // Тема dark, но backgroundColor переопределён
        assert_eq!(ctx.theme.name, "dark");
        assert_eq!(ctx.theme.background_color.to_css(), "#00FF00");
    }

    #[test]
    fn test_include_with_fs_resolver() {
        use std::io::Write;
        use tempfile::TempDir;

        // Создаём временную директорию с файлами
        let temp_dir = TempDir::new().unwrap();

        // Создаём файл для включения
        let include_path = temp_dir.path().join("common.puml");
        let mut include_file = std::fs::File::create(&include_path).unwrap();
        writeln!(include_file, "' Common definitions").unwrap();
        writeln!(include_file, "!$COLOR = \"#FF0000\"").unwrap();

        // Основной файл
        let source = r#"
!include "common.puml"
participant Alice $COLOR
"#;

        // Используем FsFileResolver
        let resolver = FsFileResolver::new(temp_dir.path());
        let preprocessor = Preprocessor::with_resolver(resolver);
        let result = preprocessor.process(source).unwrap();

        // Переменная из включённого файла должна быть подставлена
        assert!(result.contains("participant Alice #FF0000"));
    }

    #[test]
    fn test_include_once() {
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Файл, который будет включён дважды
        let include_path = temp_dir.path().join("header.puml");
        let mut include_file = std::fs::File::create(&include_path).unwrap();
        writeln!(include_file, "HEADER_LINE").unwrap();

        let source = r#"
!include_once "header.puml"
!include_once "header.puml"
BODY
"#;

        let resolver = FsFileResolver::new(temp_dir.path());
        let preprocessor = Preprocessor::with_resolver(resolver);
        let result = preprocessor.process(source).unwrap();

        // HEADER_LINE должен появиться только один раз
        let count = result.matches("HEADER_LINE").count();
        assert_eq!(
            count, 1,
            "!include_once должен включать файл только один раз"
        );
    }

    #[test]
    fn test_include_not_found() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        let source = r#"
!include "nonexistent.puml"
"#;

        let resolver = FsFileResolver::new(temp_dir.path());
        let preprocessor = Preprocessor::with_resolver(resolver);
        let result = preprocessor.process(source);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, PreprocessError::FileNotFound(_)));
    }

    #[test]
    fn test_nested_include() {
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // level2.puml
        let level2_path = temp_dir.path().join("level2.puml");
        let mut level2_file = std::fs::File::create(&level2_path).unwrap();
        writeln!(level2_file, "LEVEL2_CONTENT").unwrap();

        // level1.puml включает level2.puml
        let level1_path = temp_dir.path().join("level1.puml");
        let mut level1_file = std::fs::File::create(&level1_path).unwrap();
        writeln!(level1_file, "LEVEL1_START").unwrap();
        writeln!(level1_file, "!include \"level2.puml\"").unwrap();
        writeln!(level1_file, "LEVEL1_END").unwrap();

        let source = r#"
MAIN_START
!include "level1.puml"
MAIN_END
"#;

        let resolver = FsFileResolver::new(temp_dir.path());
        let preprocessor = Preprocessor::with_resolver(resolver);
        let result = preprocessor.process(source).unwrap();

        // Проверяем что все уровни включены
        assert!(result.contains("MAIN_START"));
        assert!(result.contains("LEVEL1_START"));
        assert!(result.contains("LEVEL2_CONTENT"));
        assert!(result.contains("LEVEL1_END"));
        assert!(result.contains("MAIN_END"));
    }
}
