//! Ошибки парсера

use thiserror::Error;

/// Преобразует ошибку pest в [`ParseError::SyntaxError`] с корректной позицией.
///
/// # Почему прежний код не работал
///
/// Каждый парсер извлекал номер строки так:
///
/// ```ignore
/// line: e.line().to_string().parse().unwrap_or(0),
/// ```
///
/// Но `pest::error::Error::line()` возвращает **текст строки**, а не её номер
/// (`pub fn line(&self) -> &str`). Разбор текста в `usize` всегда терпел
/// неудачу, и `unwrap_or(0)` давал ноль. Поэтому поле `line` в
/// `SyntaxError` **всегда** было нулевым, хотя в тексте сообщения pest
/// печатало правильный номер: программа, читающая поле, а не сообщение,
/// не могла узнать настоящую строку.
///
/// Настоящая позиция лежит в `Error::line_col` в виде
/// `LineColLocation::Pos((строка, колонка))`.
pub fn syntax_error_from_pest(
    error: &pest::error::Error<impl pest::RuleType>,
    _source: &str,
) -> ParseError {
    let line = match error.line_col {
        pest::error::LineColLocation::Pos((line, _)) => line,
        pest::error::LineColLocation::Span((line, _), _) => line,
    };

    // Строка 0 означает, что позиция неизвестна (пустой вход)
    let line = if line == 0 { 1 } else { line };

    ParseError::SyntaxError {
        line,
        message: error.to_string(),
    }
}

/// Ошибки парсинга PlantUML
#[derive(Error, Debug)]
pub enum ParseError {
    /// Неожиданный токен
    #[error("неожиданный токен '{token}' в строке {line}, позиция {column}")]
    UnexpectedToken {
        token: String,
        line: usize,
        column: usize,
    },

    /// Синтаксическая ошибка
    #[error("синтаксическая ошибка в строке {line}: {message}")]
    SyntaxError { line: usize, message: String },

    /// Неизвестный тип диаграммы
    #[error("не удалось определить тип диаграммы")]
    UnknownDiagramType,

    /// Отсутствует @startuml
    #[error("отсутствует @startuml")]
    MissingStartTag,

    /// Отсутствует @enduml
    #[error("отсутствует @enduml")]
    MissingEndTag,

    /// Неизвестный участник
    #[error("неизвестный участник: {0}")]
    UnknownParticipant(String),

    /// Ошибка лексера
    #[error("ошибка лексера: {0}")]
    LexerError(String),

    /// Ошибка грамматики
    #[error("ошибка грамматики: {0}")]
    GrammarError(String),
}
