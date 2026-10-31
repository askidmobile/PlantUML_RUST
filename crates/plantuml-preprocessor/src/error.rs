//! Ошибки препроцессора

use thiserror::Error;

/// Ошибки препроцессора PlantUML
#[derive(Error, Debug)]
pub enum PreprocessError {
    /// Ошибка чтения файла
    #[error("failed to read file: {0}")]
    FileReadError(String),

    /// !include не поддерживается
    #[error("!include is not available here: {0}")]
    IncludeNotSupported(String),

    /// Файл не найден
    #[error("file not found: {0}")]
    FileNotFound(String),

    /// Синтаксическая ошибка
    #[error("syntax error in directive: {0}")]
    SyntaxError(String),

    /// Несбалансированные !ifdef/!endif
    #[error("unbalanced !ifdef/!endif")]
    UnbalancedCondition,

    /// Неизвестная переменная
    #[error("unknown variable: {0}")]
    UnknownVariable(String),

    /// Рекурсивное включение
    #[error("recursive include: {0}")]
    RecursiveInclude(String),

    /// Ошибка вычисления выражения
    #[error("failed to evaluate expression: {0}")]
    ExpressionError(String),

    /// Исчерпан бюджет раскрытия макросов.
    ///
    /// Жёсткий предел нужен из-за инцидента 2026-07-31: разбор библиотеки
    /// C4-PlantUML разрастался до 8.5 ГБ и выедал память машины. Причина —
    /// цикл `!while`, который не сходился: каждая итерация подставляла в
    /// тело уже накопленный текст, и объём рос лавиной. Ошибка вместо
    /// роста памяти позволяет показать пользователю внятное сообщение, а
    /// не уронить процесс.
    #[error("macro expansion limit exceeded ({limit} MiB): a !while loop probably never ends")]
    ExpansionLimit {
        /// Предел в байтах.
        limit: usize,
    },
}
