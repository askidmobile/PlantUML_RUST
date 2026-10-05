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

/// Максимальное число проходов раскрытия макросов в одной строке.
///
/// Ограничивает взаимную рекурсию макросов, чтобы препроцессор не зациклился.
pub const MAX_MACRO_EXPANSIONS: usize = 32;

/// Максимальная глубина вложенности `!include`.
///
/// Защищает от бесконечной рекурсии при взаимных включениях. Значение
/// совпадает с `FsFileResolver::max_depth` по умолчанию.
pub const MAX_INCLUDE_DEPTH: usize = 10;

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

/// Резолвер, который обслуживает ТОЛЬКО встроенную стандартную библиотеку.
///
/// Нужен по умолчанию: `!include <C4/C4_Context>` не требует файловой
/// системы, данные встроены в `plantuml-stdlib`. Раньше `Preprocessor::new`
/// использовал `NoopFileResolver`, поэтому ЛЮБОЕ stdlib-включение через
/// публичный API (`render`, `parse_diagram`) падало с «!include не
/// поддерживается», и весь реестр из 48 включений был недостижим.
///
/// Файловая система здесь намеренно не используется — резолвер безопасен
/// для `wasm32-unknown-unknown`.
#[derive(Debug, Default)]
pub struct StdlibResolver;

impl FileResolver for StdlibResolver {
    fn read_file(&self, path: &str) -> Result<String> {
        // Угловые скобки — маркер стандартной библиотеки
        // (`!include <C4/C4_Context>`). Но библиотеки включают друг друга
        // ОТНОСИТЕЛЬНЫМИ путями без скобок (`!include ./C4.puml`), поэтому
        // такие пути тоже ищем в реестре.
        let stdlib_path = path.trim_matches(|c| c == '<' || c == '>');

        if let Some(content) = plantuml_stdlib::get_include(stdlib_path) {
            return Ok(content.to_string());
        }

        if path.starts_with('<') && path.ends_with('>') {
            return Err(PreprocessError::FileNotFound(format!(
                "stdlib: {stdlib_path}"
            )));
        }

        Err(PreprocessError::IncludeNotSupported(path.to_string()))
    }

    fn file_exists(&self, path: &str) -> bool {
        path.starts_with('<')
            && path.ends_with('>')
            && plantuml_stdlib::exists(path.trim_matches(|c| c == '<' || c == '>'))
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

/// Макрос PlantUML, объявленный через `!define NAME(params) тело`.
///
/// В отличие от переменной, макрос принимает аргументы: вызов
/// `NAME(x, y)` раскрывается в тело, где имена параметров заменены
/// переданными значениями.
#[derive(Debug, Clone)]
pub struct MacroDefinition {
    /// Имена параметров в порядке объявления
    pub parameters: Vec<String>,
    /// Тело макроса (то, что идёт после закрывающей скобки)
    pub body: String,
}

/// Максимальная глубина вызовов макросов.
///
/// Защищает от взаимной рекурсии процедур: без неё разбор уходит в
/// бесконечность и роняет процесс переполнением стека, а в WASM это
/// падение вкладки без возможности перехватить ошибку.
const MAX_CALL_DEPTH: usize = 64;

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
    /// Текущее определение объявлено как `!unquoted`
    defining_unquoted: bool,
    /// Текущая тема
    pub theme: Theme,
    /// SkinParam параметры
    pub skin_params: SkinParams,
    /// Макросы `!define NAME(params) тело`, определённые в исходнике.
    ///
    /// Хранятся отдельно от переменных: у макроса есть параметры и тело,
    /// а подстановка выполняется с заменой аргументов.
    pub macros: IndexMap<String, MacroDefinition>,
    /// Стек включаемых файлов для обнаружения циклов `!include`.
    ///
    /// Без него взаимные включения (a.puml → b.puml → a.puml) уходят в
    /// бесконечную рекурсию и роняют процесс переполнением стека.
    /// Для WASM это означает падение вкладки браузера без возможности
    /// перехватить ошибку.
    pub include_stack: Vec<String>,
    /// Максимальная глубина вложенности включений
    pub max_include_depth: usize,
    /// Значение, возвращённое `!return` при выполнении тела макроса.
    pub return_value: Option<String>,
    /// Признак того, что встречен `!return`: разбор тела прекращается.
    pub returning: bool,
    /// Текущая глубина вызовов макросов (защита от рекурсии).
    pub call_depth: usize,
}

impl Default for PreprocessContext {
    fn default() -> Self {
        Self {
            variables: IndexMap::new(),
            included_files: Vec::new(),
            condition_depth: 0,
            condition_stack: Vec::new(),
            callables: IndexMap::new(),
            defining_unquoted: false,
            defining: DefiningCallable::None,
            theme: Theme::default(),
            skin_params: SkinParams::new(),
            macros: IndexMap::new(),
            include_stack: Vec::new(),
            max_include_depth: MAX_INCLUDE_DEPTH,
            return_value: None,
            returning: false,
            call_depth: 0,
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

    /// Объявляет макрос `!define NAME(params) тело`.
    pub fn define_macro(
        &mut self,
        name: impl Into<String>,
        parameters: Vec<String>,
        body: impl Into<String>,
    ) {
        self.macros.insert(
            name.into(),
            MacroDefinition {
                parameters,
                body: body.into(),
            },
        );
    }

    /// Раскрывает вызовы макросов в строке.
    ///
    /// Ищет `ИМЯ(аргументы)` для каждого объявленного макроса и подставляет
    /// его тело с заменой имён параметров на переданные аргументы.
    /// Раскрытие повторяется, пока в строке есть вызовы (но не более
    /// [`MAX_MACRO_EXPANSIONS`] раз — на случай взаимной рекурсии макросов).
    pub fn expand_macros(&self, line: &str) -> String {
        if self.macros.is_empty() || !line.contains('(') {
            return line.to_string();
        }

        let mut current = line.to_string();
        for _ in 0..MAX_MACRO_EXPANSIONS {
            match self.expand_once(&current) {
                Some(next) => current = next,
                None => break,
            }
        }
        current
    }

    /// Одно прохождение раскрытия: `None`, если вызовов не найдено.
    fn expand_once(&self, line: &str) -> Option<String> {
        for (name, def) in &self.macros {
            let mut search_from = 0usize;
            while let Some(rel) = line[search_from..].find(name.as_str()) {
                let start = search_from + rel;
                let after_name = start + name.len();

                // Имя должно быть целым словом: `Person(` — да,
                // `Person_Ext(` при поиске `Person` — нет
                let boundary_ok = line[..start]
                    .chars()
                    .next_back()
                    .is_none_or(|c| !c.is_alphanumeric() && c != '_');
                if !boundary_ok {
                    search_from = after_name;
                    continue;
                }

                // Сразу за именем должна идти скобка
                let rest = &line[after_name..];
                let Some(open_rel) = rest.find('(') else {
                    break;
                };
                if open_rel != 0 {
                    search_from = after_name;
                    continue;
                }

                let open = after_name;
                let Some(close) = self::find_matching_paren_in(line, open) else {
                    search_from = after_name;
                    continue;
                };

                let args_str = &line[open + 1..close];
                let args = split_macro_args(args_str);
                let expanded = substitute_macro_args(&def.body, &def.parameters, &args);

                let mut next = String::with_capacity(line.len() + expanded.len());
                next.push_str(&line[..start]);
                next.push_str(&expanded);
                next.push_str(&line[close + 1..]);
                return Some(next);
            }
        }
        None
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
pub struct Preprocessor<R: FileResolver = StdlibResolver> {
    resolver: R,
}

impl Preprocessor<StdlibResolver> {
    /// Создаёт препроцессор со встроенной стандартной библиотекой.
    ///
    /// Файловая система не используется: доступны только включения вида
    /// `!include <C4/C4_Context>`. Для работы с файлами нужен
    /// `Preprocessor::with_resolver(FsFileResolver::new(...))`.
    pub fn new() -> Self {
        Self {
            // Стандартная библиотека встроена в бинарник, поэтому доступна
            // всегда; файловая система для этого не нужна.
            resolver: StdlibResolver,
        }
    }
}

impl Default for Preprocessor<StdlibResolver> {
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

    /// Обрабатывает исходник и возвращает текст **вместе с темой**.
    ///
    /// # Зачем отдельный метод
    ///
    /// `process` возвращает только `String`, поэтому разобранные `!theme` и
    /// `skinparam` терялись: они оседали в `PreprocessContext`, который
    /// никуда не передавался. В результате `skinparam backgroundColor #FF0000`
    /// давал байт-идентичный вывод с базовым — тема не влияла ни на что.
    ///
    /// Этот метод отдаёт тему наружу, чтобы pipeline мог применить её при
    /// рендеринге.
    ///
    /// # Пример
    ///
    /// ```
    /// use plantuml_preprocessor::Preprocessor;
    ///
    /// let pp = Preprocessor::new();
    /// let (text, theme) = pp
    ///     .process_with_theme("@startuml\nskinparam monochrome true\nA -> B\n@enduml")
    ///     .unwrap();
    /// assert!(text.contains("A -> B"));
    /// // PlantUML записывает цвета сокращённо: #000000 -> #000.
    /// assert_eq!(theme.node_border.to_css(), "#000");
    /// ```
    pub fn process_with_theme(&self, source: &str) -> Result<(String, Theme)> {
        let mut ctx = PreprocessContext::new();
        let text = self.process_with_context(source, &mut ctx)?;
        Ok((text, ctx.theme))
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

            // `!return` в теле макроса прекращает его выполнение.
            if ctx.returning {
                break;
            }

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
            //
            // Учитываем и блочную форму: строки внутри `skinparam X { ... }`
            // не начинаются со `skinparam`, но тоже относятся к настройкам
            // и не должны попадать в вывод (иначе парсер получает
            // осиротевшую `}` и падает).
            if trimmed.starts_with("skinparam ") {
                self.handle_skinparam(trimmed, ctx);
                continue;
            }
            if ctx.skin_params.in_block() {
                self.handle_skinparam(trimmed, ctx);
                continue;
            }

            // Подстановка переменных
            let processed = self.substitute_variables(line, ctx);

            // Раскрытие макросов `!define NAME(params) тело`.
            // Выполняется после подстановки переменных: тело макроса может
            // ссылаться на переменные, объявленные ранее.
            let processed = ctx.expand_macros(&processed);

            // Обработка вызовов пользовательских функций
            let processed = self.process_function_calls(&processed, ctx);

            // Встроенные функции, которым нужен контекст
            let processed = self.process_context_builtins(&processed, ctx);

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

        if let Some(rest) = directive.strip_prefix("return") {
            // `!return` завершает выполнение тела макроса. Значение
            // подставляется из переменных: полноценный разбор выражений
            // не нужен, библиотеки возвращают уже готовые строки.
            // Значение тоже ВЫРАЖЕНИЕ: `!return "[" + $x + "]"` должно
            // вернуть склеенную строку, а не текст выражения. Вызовы
            // функций внутри вычисляются РАНЬШЕ: `!return $bl()` иначе
            // возвращал текст вызова.
            let expanded = self.process_function_calls(rest.trim(), ctx);
            let value = evaluate_concat(&expanded, ctx);
            ctx.return_value = Some(value);
            ctx.returning = true;
        } else if let Some(rest) = directive.strip_prefix("global ") {
            // `!global $ИМЯ ?= значение` — объявление переменной уровня
            // библиотеки. Разбирается тем же кодом, что и присваивание.
            variables::handle_variable_assignment(rest.trim(), ctx)?;
        } else if let Some(rest) = directive.strip_prefix("define ") {
            directives::handle_define(rest, ctx)?;
        } else if let Some(rest) = directive.strip_prefix("undef ") {
            directives::handle_undef(rest.trim(), ctx);
        } else if let Some(rest) = directive.strip_prefix("if ") {
            directives::handle_if(rest.trim(), ctx);
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
        } else if let Some(rest) = directive.strip_prefix("unquoted procedure ") {
            self.start_procedure_definition(rest.trim(), ctx)?;
            ctx.defining_unquoted = true;
        } else if let Some(rest) = directive.strip_prefix("unquoted function ") {
            self.start_function_definition(rest.trim(), ctx)?;
            ctx.defining_unquoted = true;
        } else if let Some(rest) = directive.strip_prefix("procedure ") {
            self.start_procedure_definition(rest.trim(), ctx)?;
        } else if let Some(rest) = directive.strip_prefix("theme ") {
            self.handle_theme(rest.trim(), ctx)?;
        } else if directive.starts_with('$') {
            // Переменная: `!$var = выражение`.
            //
            // Вызовы функций в правой части раскрываются ДО присваивания:
            // `!$x = $x + $elementTagSkinparams("rectangle", ...)` должен
            // получить значение функции, а не её текст. Без этого в
            // переменной оседал сырой вызов, и он попадал в диаграмму —
            // именно так ломалась библиотека C4.
            let expanded = self.process_function_calls(directive, ctx);
            variables::handle_variable_assignment(&expanded, ctx)?;
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

        // ВАЖНО: угловые скобки — маркер стандартной библиотеки
        // (`!include <C4/C4_Context>`), резолвер распознаёт их сам.
        // Срезать их здесь нельзя, иначе stdlib-путь превращается в
        // обычный и никогда не находится. Кавычки снимаем: это просто
        // способ экранировать путь с пробелами.
        let path = path.trim();
        let normalized = if path.starts_with('<') && path.ends_with('>') {
            // stdlib-путь: сохраняем скобки как маркер
            path.to_string()
        } else {
            path.trim_matches('"').to_string()
        };

        // Ключ для отслеживания — без скобок и кавычек, чтобы `!include X`
        // и `!include <X>` не считались разными файлами.
        let key = normalized
            .trim_matches(|c| c == '<' || c == '>' || c == '"')
            .to_string();

        if once && ctx.included_files.contains(&key) {
            return Ok(None);
        }

        // Защита от бесконечной рекурсии при взаимных включениях
        // (a.puml -> b.puml -> a.puml). Без неё процесс падает
        // переполнением стека, а в WASM это падение вкладки.
        if ctx.include_stack.iter().any(|p| p == &key) {
            let chain = ctx
                .include_stack
                .iter()
                .map(String::as_str)
                .chain(std::iter::once(key.as_str()))
                .collect::<Vec<_>>()
                .join(" -> ");
            return Err(PreprocessError::RecursiveInclude(format!(
                "циклическое включение: {chain}"
            )));
        }

        if ctx.include_stack.len() >= ctx.max_include_depth {
            return Err(PreprocessError::RecursiveInclude(format!(
                "превышена максимальная глубина включений ({})",
                ctx.max_include_depth
            )));
        }

        // Пробуем прочитать как есть, затем — ОТНОСИТЕЛЬНО текущего файла.
        //
        // Библиотеки stdlib включают друг друга относительными путями:
        // `Archimate.puml` содержит `!include themes/shared_style.puml`.
        // Такой путь нужно разрешать внутри того же каталога stdlib.
        let content = match self.resolver.read_file(&normalized) {
            Ok(content) => content,
            Err(first_error) => {
                let parent = ctx
                    .include_stack
                    .last()
                    .and_then(|current| current.rsplit_once('/'))
                    .map(|(dir, _)| dir.to_string());

                match parent {
                    Some(dir) => {
                        let relative = normalize_include_path(&format!("{}/{}", dir, key));
                        self.resolver
                            .read_file(&relative)
                            .map_err(|_| first_error)?
                    }
                    None => return Err(first_error),
                }
            }
        };

        ctx.included_files.push(key.clone());
        ctx.include_stack.push(key);

        // Рекурсивная обработка включённого файла
        let result = self.process_with_context(&content, ctx);

        // Снимаем со стека в любом случае, иначе ошибка в глубине
        // оставит ложный след
        ctx.include_stack.pop();

        Ok(Some(result?))
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
    ///
    /// Поддерживает обе формы PlantUML:
    /// - однострочную: `skinparam monochrome true`
    /// - блочную: `skinparam rectangle { ... }`
    ///
    /// Блочная форма важна для стандартной библиотеки: C4 и другие наборы
    /// задают внешний вид именно так. Раньше блок не распознавался, его
    /// строки не поглощались, и во вход парсера попадал осиротевший `}`.
    fn handle_skinparam(&self, line: &str, ctx: &mut PreprocessContext) {
        // Внутри блока строки не начинаются со `skinparam`, поэтому
        // префикс снимаем только если он есть, иначе берём строку целиком.
        // (`strip_prefix(..).unwrap_or("")` здесь обнулил бы строку `}`.)
        let rest = match line.strip_prefix("skinparam ") {
            Some(after) => after.trim(),
            None => line.trim(),
        };

        // Конец блока — проверяем до остальных форм
        if rest == "}" {
            ctx.skin_params.end_block();
            return;
        }

        // Блочная форма: skinparam <тип> { ... }
        if let Some(before_brace) = rest.strip_suffix('{') {
            let section = before_brace.trim();
            if !section.is_empty() {
                ctx.skin_params.begin_block(section);
            }
            return;
        }

        // Строка внутри блока: `<ключ> <значение>` без префикса skinparam
        if ctx.skin_params.in_block() && !rest.is_empty() {
            let mut parts = rest.splitn(2, char::is_whitespace);
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                ctx.skin_params.set_in_block(key.trim(), value.trim());
                ctx.apply_skin_params();
            }
            return;
        }

        // Однострочная форма: skinparam <ключ> <значение>
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
            DefiningCallable::Function(mut c) | DefiningCallable::Procedure(mut c) => {
                // Флаг ставится при разборе `!unquoted` и переносится сюда:
                // само определение попадает в реестр только по !endprocedure.
                c.unquoted = ctx.defining_unquoted;
                ctx.defining_unquoted = false;
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

    /// ВЫПОЛНЯЕТ тело макроса, а не подставляет его текстом.
    ///
    /// Возвращает `(вывод, возвращённое значение)`. Важное отличие от
    /// текстовой подстановки: директивы тела (`!if`, `!$x = ...`,
    /// вложенные вызовы) обрабатываются по-настоящему. Именно этого не
    /// хватало стандартной библиотеке: например C4 строит таблицы
    /// свойств условными блоками внутри процедур.
    ///
    /// Побочные эффекты (изменённые `!$переменные`) переносятся в
    /// ВЫЗЫВАЮЩИЙ контекст: библиотеки рассчитывают на то, что переменная,
    /// установленная внутри процедуры, видна снаружи.
    fn execute_callable(
        &self,
        callable: &functions::UserCallable,
        args: &[String],
        ctx: &mut PreprocessContext,
    ) -> Result<(String, Option<String>)> {
        if ctx.call_depth >= MAX_CALL_DEPTH {
            return Err(PreprocessError::RecursiveInclude(format!(
                "превышена глубина вызовов макросов ({MAX_CALL_DEPTH})"
            )));
        }

        let mut child = PreprocessContext {
            // Переменные и объявления наследуются, но условия начинаются
            // заново: тело макроса не должно зависеть от того, внутри
            // какого `!if` он вызван.
            variables: ctx.variables.clone(),
            included_files: ctx.included_files.clone(),
            condition_depth: 0,
            condition_stack: Vec::new(),
            callables: ctx.callables.clone(),
            defining: DefiningCallable::None,
            defining_unquoted: false,
            theme: ctx.theme.clone(),
            skin_params: ctx.skin_params.clone(),
            macros: ctx.macros.clone(),
            include_stack: ctx.include_stack.clone(),
            max_include_depth: ctx.max_include_depth,
            return_value: None,
            returning: false,
            call_depth: ctx.call_depth + 1,
        };

        // Связываем параметры с аргументами. Значение по умолчанию берётся
        // из объявления (`$b = ""`), если аргумент не передан.
        for (index, parameter) in callable.parameters.iter().enumerate() {
            let (name, default) = match parameter.split_once('=') {
                Some((name, default)) => (
                    name.trim().to_string(),
                    default.trim().trim_matches('"').to_string(),
                ),
                None => (parameter.trim().to_string(), String::new()),
            };

            let mut value = match args.get(index) {
                Some(value) if !value.trim().is_empty() => value.clone(),
                _ => default,
            };

            if callable.unquoted {
                let trimmed = value.trim();
                if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
                    value = trimmed[1..trimmed.len() - 1].to_string();
                }
            }

            child.variables.insert(name, value);
        }

        let body = callable.body.join("\n");
        let output = self.process_with_context(&body, &mut child)?;

        // Побочные эффекты — наружу.
        ctx.variables = child.variables;

        Ok((output, child.return_value))
    }

    /// Обрабатывает встроенные функции, которым нужен ДОСТУП К КОНТЕКСТУ.
    ///
    /// `builtins::process_builtins` — чистые преобразования строки, они не
    /// могут менять переменные. А стандартная библиотека активно
    /// пользуется именно такими функциями:
    ///
    /// ```text
    /// %set_variable_value("$" + $tag + "_LineLegend", %true())
    /// %function_exists("%breakline")
    /// ```
    ///
    /// Первый аргумент `%set_variable_value` — КОНКАТЕНАЦИЯ через `+`,
    /// поэтому она вычисляется, а не берётся как строка.
    fn process_context_builtins(&self, line: &str, ctx: &mut PreprocessContext) -> String {
        let mut result = line.to_string();

        // %function_exists("имя") — объявлена ли такая функция/процедура
        while let Some(start) = result.find("%function_exists(") {
            let Some(close) = find_closing_paren(&result, start) else {
                break;
            };
            let inner = result[start + "%function_exists(".len()..close].to_string();
            let name = inner.trim().trim_matches('"');
            let known =
                ctx.get_callable(name).is_some() || ctx.get_callable(&format!("${name}")).is_some();
            let value = if known { "true" } else { "false" };
            result = format!("{}{}{}", &result[..start], value, &result[close + 1..]);
        }

        // %set_variable_value(ИМЯ, ЗНАЧЕНИЕ) — устанавливает переменную и
        // НЕ печатает ничего: в библиотеках это оператор ради эффекта.
        while let Some(start) = result.find("%set_variable_value(") {
            let Some(close) = find_closing_paren(&result, start) else {
                break;
            };
            let inner = result[start + "%set_variable_value(".len()..close].to_string();
            let (name, value) = split_top_level_args(&inner);

            let name = evaluate_concat(&name, ctx);
            let value = evaluate_concat(&value, ctx);

            if name.starts_with('$') {
                ctx.variables.insert(name, value);
            }

            result = format!("{}{}", &result[..start], &result[close + 1..]);
        }

        result
    }

    /// Обрабатывает вызовы пользовательских функций в строке
    fn process_function_calls(&self, line: &str, ctx: &mut PreprocessContext) -> String {
        let calls = functions::find_function_calls(line);

        if calls.is_empty() {
            return line.to_string();
        }

        let mut result = line.to_string();

        // Обрабатываем вызовы в обратном порядке (чтобы не сбивались индексы)
        for (start, end, name, args) in calls.into_iter().rev() {
            // Клонируем: дальше нужен изменяемый доступ к контексту,
            // потому что тело макроса меняет переменные.
            if let Some(callable) = ctx.get_callable(&name).cloned() {
                // При ошибке выполнения ОСТАВЛЯЕМ исходный текст вызова:
                // подставить пустоту значило бы молча потерять содержимое
                // диаграммы. Так сбой виден в выводе.
                let Ok((output_text, return_value)) = self.execute_callable(&callable, &args, ctx)
                else {
                    continue;
                };
                let output_lines: Vec<String> = output_text.lines().map(str::to_string).collect();

                // Вызов во всю строку — это ОПЕРАТОР, а не выражение.
                //
                // PlantUML выполняет такой вызов ради побочного эффекта и
                // НЕ печатает возвращённое значение. Без этого правила
                // библиотека C4 оставляла в выводе строку `""`: её функция
                // `SetPropertyHeader` вызывается на верхнем уровне только
                // ради установки переменных.
                let leading = &result[..start];
                let trailing = &result[end..];
                let is_statement = leading.trim().is_empty() && trailing.trim().is_empty();

                let replacement = match callable.kind {
                    functions::CallableKind::Function if is_statement => String::new(),
                    functions::CallableKind::Function => {
                        // Функция в выражении: подставляем возвращённое значение
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

/// Возвращает индекс закрывающей скобки для открывающей на `open`.
///
/// Учитывает вложенность: аргументы встроенных функций сами содержат
/// скобки (`%function_exists("%breakline")`).
fn find_closing_paren(text: &str, open: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0i32;

    for (offset, byte) in bytes.iter().enumerate().skip(open) {
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

/// Делит аргументы функции по запятой ВЕРХНЕГО уровня.
///
/// Запятые внутри вложенных вызовов и кавычек не считаются разделителями.
fn split_top_level_args(text: &str) -> (String, String) {
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut in_quotes = false;

    for (offset, byte) in bytes.iter().enumerate() {
        match byte {
            b'"' => in_quotes = !in_quotes,
            b'(' if !in_quotes => depth += 1,
            b')' if !in_quotes => depth -= 1,
            b',' if !in_quotes && depth == 0 => {
                return (
                    text[..offset].trim().to_string(),
                    text[offset + 1..].trim().to_string(),
                );
            }
            _ => {}
        }
    }

    (text.trim().to_string(), String::new())
}

/// Вычисляет конкатенацию вида `"$" + $tag + "_LineLegend"`.
///
/// Части соединяются через `+`; кавычки снимаются, переменные
/// подставляются из контекста.
pub(crate) fn evaluate_concat(expression: &str, ctx: &PreprocessContext) -> String {
    let mut out = String::new();
    let mut depth = 0i32;
    let mut in_quotes = false;
    let mut part = String::new();

    for ch in expression.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                part.push(ch);
            }
            '(' if !in_quotes => {
                depth += 1;
                part.push(ch);
            }
            ')' if !in_quotes => {
                depth -= 1;
                part.push(ch);
            }
            '+' if !in_quotes && depth == 0 => {
                out.push_str(&resolve_concat_part(&part, ctx));
                part.clear();
            }
            _ => part.push(ch),
        }
    }

    out.push_str(&resolve_concat_part(&part, ctx));
    out.trim().to_string()
}

/// Приводит одну часть конкатенации к значению.
fn resolve_concat_part(part: &str, ctx: &PreprocessContext) -> String {
    let trimmed = part.trim();
    let unquoted = trimmed.trim_matches('"');
    let substituted = variables::substitute(unquoted, &ctx.variables);
    substituted.trim_matches('"').to_string()
}

/// Нормализует путь включения: убирает `.` и разворачивает `..`.
///
/// Стандартная библиотека включает файлы относительными путями вида
/// `./C4.puml`, поэтому без нормализации ключ `C4/./C4.puml` не совпал бы
/// с записью реестра `C4/C4.puml`.
fn normalize_include_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();

    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }

    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Кириллица перед вызовом процедуры не вызывает панику.
    ///
    /// Регрессия: `find_function_calls` собирал `Vec<char>` и возвращал
    /// индексы в СИМВОЛАХ, а `process_function_calls` резал строку по ним
    /// как по БАЙТОВЫМ. На ASCII это совпадает, на кириллице — нет:
    /// `&result[..start]` попадал в середину символа и паниковал
    /// «byte index N is not a char boundary».
    #[test]
    fn test_cyrillic_before_procedure_call() {
        let pp = Preprocessor::new();
        let source = "@startuml\n!procedure $X($a)\nclass $a\n!endprocedure\n' Комментарий\n$X(Имя)\n@enduml";
        let out = pp
            .process(source)
            .expect("кириллица перед вызовом не должна падать");
        assert!(out.contains("class Имя"), "вызов не раскрылся: {out}");
    }

    /// Стандартная библиотека доступна без явного резолвера.
    ///
    /// Регрессия: `Preprocessor::new` использовал `NoopFileResolver`,
    /// поэтому ЛЮБОЕ `!include <...>` через публичный API падало с
    /// «!include не поддерживается», и весь реестр stdlib был недостижим.
    #[test]
    fn test_stdlib_include_without_resolver() {
        let pp = Preprocessor::new();
        let out = pp
            .process("@startuml\n!include <C4/C4_Context>\nA -> B\n@enduml")
            .expect("stdlib должен быть доступен по умолчанию");
        assert!(
            out.contains("C4_Context.puml") || out.contains("C4 Model"),
            "содержимое stdlib не подставлено"
        );
    }

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

        // PlantUML записывает цвета в сокращённой форме: `#FF0000` → `#F00`
        // (проверено на сервере для skinparam).
        assert_eq!(ctx.theme.background_color.to_css(), "#F00");
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
        assert_eq!(ctx.theme.background_color.to_css(), "#0F0");
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

    /// Имя переменной не подставляется ВНУТРИ более длинного имени.
    ///
    /// Регрессия: подстановка шла через `replace("$element", ...)`,
    /// поэтому `$elementSkin` превращался в `<значение>Skin`. На этом
    /// стандартная библиотека получала `rectangleSkin` вместо
    /// `skinparam rectangle<<...>>`.
    #[test]
    fn test_prefix_variable_name_is_not_replaced() {
        let pp = Preprocessor::new();
        let source = "@startuml\n\
            !$element = rectangle\n\
            !$elementSkin = \"skinparam \" + $element\n\
            class $elementSkin\n\
            @enduml";

        let out = pp.process(source).expect("разбор не должен падать");
        assert!(
            out.contains("class skinparam rectangle"),
            "длинное имя повреждено коротким: {out}"
        );
    }

    /// Вызов функции внутри `!return` раскрывается.
    #[test]
    fn test_function_call_inside_return() {
        let pp = Preprocessor::new();
        let source = "@startuml\n\
            !function $inner($a)\n\
            !return \"[\" + $a + \"]\"\n\
            !endfunction\n\
            !function $outer($b)\n\
            !return \"<\" + $inner($b) + \">\"\n\
            !endfunction\n\
            !$v = Внутри\n\
            class $outer($v)\n\
            @enduml";

        let out = pp.process(source).expect("разбор не должен падать");
        assert!(
            out.contains("class <[Внутри]>"),
            "вложенный вызов не раскрыт: {out}"
        );
    }

    /// Присваивание ВЫЧИСЛЯЕТ правую часть, а не хранит её текстом.
    ///
    /// Регрессия: `!$x = "a" + $v` сохраняло строку выражения, и в
    /// диаграмму попадал текст вида `"rectangle<<" + $a`. На этом
    /// ломалась библиотека C4.
    #[test]
    fn test_assignment_evaluates_expression() {
        let pp = Preprocessor::new();
        let source = "@startuml\n\
            !$v = Мир\n\
            !$x = \"Привет, \" + $v\n\
            class $x\n\
            @enduml";

        let out = pp.process(source).expect("разбор не должен падать");
        assert!(
            out.contains("class Привет, Мир"),
            "конкатенация не вычислена: {out}"
        );
    }

    /// `+` внутри кавычек не считается конкатенацией.
    #[test]
    fn test_plus_inside_quotes_is_literal() {
        let pp = Preprocessor::new();
        let source = "@startuml\n!$x = \"a+b\"\nclass $x\n@enduml";
        let out = pp.process(source).expect("разбор не должен падать");
        assert!(
            out.contains("class a+b"),
            "плюс внутри кавычек потерян: {out}"
        );
    }

    /// Вызов функции в правой части присваивания раскрывается.
    #[test]
    fn test_function_call_inside_assignment() {
        let pp = Preprocessor::new();
        let source = "@startuml\n\
            !function $f($x)\n\
            !return \"[\" + $x + \"]\"\n\
            !endfunction\n\
            !$y = \"A\" + $f(B)\n\
            class $y\n\
            @enduml";

        let out = pp.process(source).expect("разбор не должен падать");
        assert!(
            out.contains("class A[B]"),
            "вызов в присваивании не раскрыт: {out}"
        );
    }

    /// Неразобранное условие не должно ВЫПОЛНЯТЬ блок.
    ///
    /// Регрессия: прежняя версия умела только `==` и `!=`, а всё прочее
    /// молча считала истиной. Для условия с `&&` она делила строку по
    /// `!=`, получала в операндах мусор и выполняла блок.
    #[test]
    fn test_unparsed_condition_does_not_execute() {
        let pp = Preprocessor::new();
        let source = "@startuml\n\
            !$a = 1\n\
            !if ($a > 3 && $a < 10)\n\
            class Лишняя\n\
            !else\n\
            class Верно\n\
            !endif\n\
            @enduml";

        let out = pp.process(source).expect("разбор не должен падать");
        assert!(
            !out.contains("class Лишняя"),
            "выполнена ложная ветка: {out}"
        );
        assert!(
            out.contains("class Верно"),
            "не выполнена верная ветка: {out}"
        );
    }

    /// Конъюнкция, дизъюнкция и числовые сравнения в условиях.
    #[test]
    fn test_condition_operators() {
        let pp = Preprocessor::new();

        let and_true = "@startuml\n!$a = 5\n!if ($a > 3 && $a < 10)\nclass Да\n!endif\n@enduml";
        assert!(pp.process(and_true).unwrap().contains("class Да"));

        let or_true = "@startuml\n!$a = 1\n!if ($a == 9 || $a == 1)\nclass Или\n!endif\n@enduml";
        assert!(pp.process(or_true).unwrap().contains("class Или"));

        let ge_false = "@startuml\n!$a = 1\n!if ($a >= 3)\nclass Нет\n!endif\n@enduml";
        assert!(!pp.process(ge_false).unwrap().contains("class Нет"));
    }

    /// `%set_variable_value` вычисляет конкатенацию и ничего не печатает.
    ///
    /// Стандартная библиотека зовёт её как оператор:
    /// `%set_variable_value("$" + $tag + "_LineLegend", %true())`.
    #[test]
    fn test_set_variable_value() {
        let pp = Preprocessor::new();
        let source = "@startuml\n\
            !unquoted function $F()\n\
            !$tag = Важное\n\
            %set_variable_value(\"$\" + $tag + \"_Флаг\", %true())\n\
            !return \"\"\n\
            !endfunction\n\
            $F()\n\
            !if ($Важное_Флаг == true)\n\
            class Установлено\n\
            !endif\n\
            @enduml";

        let out = pp.process(source).expect("разбор не должен падать");
        assert!(
            !out.contains("%set_variable_value"),
            "вызов просочился в вывод: {out}"
        );
        assert!(
            out.contains("class Установлено"),
            "переменная не установлена: {out}"
        );
    }

    /// `%function_exists` работает и в условии `!if`.
    #[test]
    fn test_function_exists_in_condition() {
        let pp = Preprocessor::new();
        let source = "@startuml\n\
            !function $Есть()\n\
            !return 1\n\
            !endfunction\n\
            !if (%function_exists(\"$Есть\"))\n\
            class Найдена\n\
            !endif\n\
            !if (%function_exists(\"$Нет\"))\n\
            class Лишняя\n\
            !endif\n\
            @enduml";

        let out = pp.process(source).expect("разбор не должен падать");
        assert!(
            out.contains("class Найдена"),
            "известная функция не найдена: {out}"
        );
        assert!(
            !out.contains("class Лишняя"),
            "неизвестная функция найдена: {out}"
        );
    }

    /// Тело макроса ВЫПОЛНЯЕТСЯ, а не подставляется текстом.
    ///
    /// Регрессия: раньше `UserCallable::call` делал только текстовую
    /// подстановку аргументов, поэтому директивы тела (`!if`, `!$x = ...`)
    /// попадали в вывод как есть. На этом падала стандартная библиотека:
    /// C4 строит таблицы свойств условными блоками внутри процедур.
    #[test]
    fn test_macro_body_is_executed_not_substituted() {
        let pp = Preprocessor::new();
        let source = "@startuml\n\
            !procedure $P($x)\n\
            !if ($x == 1)\n\
            class Один\n\
            !else\n\
            class Другой\n\
            !endif\n\
            !endprocedure\n\
            $P(1)\n\
            @enduml";

        let out = pp.process(source).expect("разбор не должен падать");
        assert!(
            out.contains("class Один"),
            "условие в теле не сработало: {out}"
        );
        assert!(
            !out.contains("class Другой"),
            "выполнена лишняя ветка: {out}"
        );
        assert!(!out.contains("!if"), "директива просочилась в вывод: {out}");
    }

    /// Переменная, установленная внутри процедуры, видна снаружи.
    ///
    /// Библиотеки на это рассчитывают: C4 вызывает `SetPropertyHeader`
    /// ради побочного эффекта и потом читает установленные переменные.
    #[test]
    fn test_macro_variable_leaks_outward() {
        let pp = Preprocessor::new();
        let source = "@startuml\n\
            !procedure $Set()\n\
            !$глоб = Готово\n\
            !endprocedure\n\
            $Set()\n\
            class $глоб\n\
            @enduml";

        let out = pp.process(source).expect("разбор не должен падать");
        assert!(
            out.contains("class Готово"),
            "переменная не видна снаружи: {out}"
        );
    }

    /// `!return` вычисляется, параметр по умолчанию подставляется.
    #[test]
    fn test_macro_return_and_default_parameter() {
        let pp = Preprocessor::new();
        let source = "@startuml\n\
            !function $F($a, $b = Умолчание)\n\
            !if ($a == 1)\n\
            !return $b\n\
            !endif\n\
            !return Другое\n\
            !endfunction\n\
            class $F(1)\n\
            @enduml";

        let out = pp.process(source).expect("разбор не должен падать");
        assert!(
            out.contains("class Умолчание"),
            "return/умолчание не сработали: {out}"
        );
    }

    /// Взаимная рекурсия макросов не роняет процесс.
    ///
    /// В WASM переполнение стека означает падение вкладки, поэтому нужен
    /// явный предел глубины вызовов.
    #[test]
    fn test_macro_recursion_is_bounded() {
        let pp = Preprocessor::new();
        let source = "@startuml\n\
            !procedure $A()\n\
            $B()\n\
            !endprocedure\n\
            !procedure $B()\n\
            $A()\n\
            !endprocedure\n\
            $A()\n\
            @enduml";

        // Главное — не паника и не зависание: допустим и Result::Err.
        let _ = pp.process(source);
    }
}
#[cfg(test)]
mod include_tests {
    use super::*;
    use std::io::Write;

    /// Создаёт временную директорию с файлами.
    fn tmpdir(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for (name, content) in files {
            let path = dir.path().join(name);
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(content.as_bytes()).unwrap();
        }
        dir
    }

    #[test]
    fn test_include_cycle_detected() {
        let dir = tmpdir(&[
            ("a.puml", "!include \"b.puml\"\n"),
            ("b.puml", "!include \"a.puml\"\n"),
        ]);
        let pp = Preprocessor::with_resolver(FsFileResolver::new(dir.path()));
        let err = pp
            .process("@startuml\n!include \"a.puml\"\nA -> B\n@enduml")
            .expect_err("циклическое включение должно давать ошибку, а не падать");
        let msg = err.to_string();
        assert!(
            msg.contains("циклическое") || msg.contains("рекурсивное"),
            "неожиданная ошибка: {msg}"
        );
    }

    #[test]
    fn test_include_self_cycle_detected() {
        let dir = tmpdir(&[("self.puml", "!include \"self.puml\"\n")]);
        let pp = Preprocessor::with_resolver(FsFileResolver::new(dir.path()));
        assert!(pp
            .process("@startuml\n!include \"self.puml\"\n@enduml")
            .is_err());
    }

    #[test]
    fn test_include_depth_limit() {
        // Цепочка из 20 файлов при лимите 10 должна упасть по глубине
        let files: Vec<(String, String)> = (0..20)
            .map(|i| {
                let name = format!("f{i}.puml");
                let content = if i < 19 {
                    format!("!include \"f{}.puml\"\n", i + 1)
                } else {
                    String::new()
                };
                (name, content)
            })
            .collect();
        let refs: Vec<(&str, &str)> = files
            .iter()
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .collect();
        let dir = tmpdir(&refs);
        let pp = Preprocessor::with_resolver(FsFileResolver::new(dir.path()));
        let err = pp
            .process("@startuml\n!include \"f0.puml\"\n@enduml")
            .unwrap_err();
        assert!(
            err.to_string().contains("глубин"),
            "неожиданная ошибка: {err}"
        );
    }

    #[test]
    fn test_stdlib_include_through_preprocessor() {
        // Регрессия: раньше угловые скобки срезались в handle_include,
        // и stdlib-путь никогда не находился.
        //
        // Проверяем по комментариям из stdlib-файла: директивы `!define`
        // поглощаются препроцессором и в вывод не попадают.
        let dir = tempfile::tempdir().unwrap();
        let pp = Preprocessor::with_resolver(FsFileResolver::new(dir.path()));
        let out = pp
            .process("@startuml\n!include <C4/C4_Context>\nA -> B\n@enduml")
            .expect("stdlib-включение должно разрешаться");
        assert!(
            out.contains("C4_Context.puml") || out.contains("C4 Model"),
            "содержимое stdlib не подставлено, вывод:\n{out}"
        );
    }

    #[test]
    fn test_stdlib_unknown_path_is_error() {
        let dir = tempfile::tempdir().unwrap();
        let pp = Preprocessor::with_resolver(FsFileResolver::new(dir.path()));
        assert!(
            pp.process("@startuml\n!include <nope/nope>\nA -> B\n@enduml")
                .is_err(),
            "несуществующий stdlib-путь должен давать ошибку"
        );
    }

    #[test]
    fn test_skinparam_block_is_consumed() {
        // Регрессия: блочный skinparam раньше не распознавался, его строки
        // попадали в вывод, и парсер падал на осиротевшей `}`.
        let dir = tempfile::tempdir().unwrap();
        let pp = Preprocessor::with_resolver(FsFileResolver::new(dir.path()));
        let out = pp
            .process(
                "@startuml\nskinparam rectangle {\n  FontColor #FFFFFF\n  roundCorner 8\n}\nA -> B\n@enduml",
            )
            .unwrap();
        assert!(
            !out.contains('}'),
            "осиротевшая `}}` попала в вывод:\n{out}"
        );
        assert!(
            !out.contains("roundCorner"),
            "строка блока попала в вывод:\n{out}"
        );
        assert!(out.contains("A -> B"), "содержимое диаграммы потеряно");
    }

    #[test]
    fn test_include_once_not_duplicated() {
        // Проверяем по комментарию: `!define` поглощается препроцессором
        // и в выводе не появляется.
        let dir = tmpdir(&[("common.puml", "' маркер включения\n")]);
        let pp = Preprocessor::with_resolver(FsFileResolver::new(dir.path()));
        let out = pp
            .process(
                "@startuml\n!include_once \"common.puml\"\n!include_once \"common.puml\"\n@enduml",
            )
            .unwrap();
        assert_eq!(
            out.matches("маркер включения").count(),
            1,
            "!include_once включил файл дважды"
        );
    }
}

/// Находит позицию закрывающей скобки, соответствующей открывающей.
fn find_matching_paren_in(text: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (i, c) in text.char_indices().skip_while(|(i, _)| *i < open) {
        match c {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Разбивает строку аргументов вызова макроса по запятым верхнего уровня.
///
/// Учитывает вложенные скобки и кавычки: аргумент вида
/// `rectangle "a, b"` не должен разрываться по запятой внутри кавычек.
fn split_macro_args(args: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    let mut in_quotes = false;
    let mut escaped = false;

    for c in args.chars() {
        if escaped {
            current.push(c);
            escaped = false;
            continue;
        }
        match c {
            '\\' if in_quotes => {
                current.push(c);
                escaped = true;
            }
            '"' => {
                in_quotes = !in_quotes;
                current.push(c);
            }
            '(' | '[' if !in_quotes => {
                depth += 1;
                current.push(c);
            }
            ')' | ']' if !in_quotes => {
                depth = depth.saturating_sub(1);
                current.push(c);
            }
            ',' if depth == 0 && !in_quotes => {
                result.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(c),
        }
    }

    if !current.trim().is_empty() || !result.is_empty() {
        result.push(current.trim().to_string());
    }
    result
}

/// Подставляет аргументы в тело макроса.
///
/// Заменяет имена параметров на переданные значения, соблюдая границы слов:
/// параметр `e_label` не должен заменяться внутри `e_label_extra`.
///
/// Отсутствующие аргументы подставляются пустой строкой — PlantUML
/// допускает вызов макроса с меньшим числом аргументов.
fn substitute_macro_args(body: &str, params: &[String], args: &[String]) -> String {
    let mut result = body.to_string();
    for (i, param) in params.iter().enumerate() {
        let raw = args.get(i).map(String::as_str).unwrap_or("");

        // Обрамляющие кавычки снимаются — это проверено на plantuml.com:
        // `!define M(a) class "PRE a POST"` даёт одинаковый результат и при
        // вызове `M("Имя")`, и при `M(Имя)`, то есть кавычки PlantUML
        // отбрасывает.
        //
        // Без этого вызов `Person(a, "Имя")` подставлял аргумент внутрь уже
        // закавыченной строки макроса, и получалось `"=="Имя""` —
        // неразбираемая конструкция. Именно так ломалась стандартная
        // библиотека C4, объявляющая макросы через `!define`.
        let trimmed = raw.trim();
        let value = if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
            &trimmed[1..trimmed.len() - 1]
        } else {
            raw
        };

        result = replace_word(&result, param, value);
    }
    result
}

/// Заменяет вхождения целого слова (с учётом границ) на новое значение.
fn replace_word(text: &str, word: &str, replacement: &str) -> String {
    if word.is_empty() {
        return text.to_string();
    }

    let mut out = String::with_capacity(text.len());
    let mut rest = text;

    while let Some(pos) = rest.find(word) {
        // Граница слева. Отдельно обрабатываем эскейп-последовательности:
        // в телах макросов C4 параметр часто идёт сразу после литерала `\n`
        // (например `\n\ne_descr`), и буква `n` не должна считаться частью
        // имени. Поэтому если символ предварён обратным слэшем, это граница.
        let before = &rest[..pos];
        let before_ok = match before.chars().next_back() {
            None => true,
            Some(c) if !c.is_alphanumeric() && c != '_' => true,
            Some(_) => before.chars().rev().nth(1).is_some_and(|prev| prev == '\\'),
        };
        let after = &rest[pos + word.len()..];
        let after_ok = after
            .chars()
            .next()
            .is_none_or(|c| !c.is_alphanumeric() && c != '_');

        if before_ok && after_ok {
            out.push_str(&rest[..pos]);
            out.push_str(replacement);
            rest = after;
        } else {
            // Не целое слово — оставляем как есть и ищем дальше
            out.push_str(&rest[..pos + word.len()]);
            rest = after;
        }
    }
    out.push_str(rest);
    out
}
