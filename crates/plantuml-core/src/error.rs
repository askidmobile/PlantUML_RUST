//! Типы ошибок для plantuml-core

use thiserror::Error;

/// Тип результата для plantuml-core
pub type Result<T> = std::result::Result<T, Error>;

/// Ошибки библиотеки plantuml-core
#[derive(Error, Debug)]
pub enum Error {
    /// Ошибка препроцессора
    #[error("preprocessor error: {0}")]
    Preprocess(String),

    /// Ошибка парсинга
    #[error("{0}")]
    Parse(String),

    /// Ошибка layout
    #[error("layout error: {0}")]
    Layout(String),

    /// Ошибка рендеринга
    #[error("render error: {0}")]
    Render(String),

    /// Неподдерживаемый тип диаграммы
    #[error("unsupported diagram type: {0}")]
    UnsupportedDiagram(String),

    /// Неизвестная тема
    #[error("unknown theme: {0}")]
    UnknownTheme(String),

    /// Пустой исходный код
    #[error("empty source")]
    EmptySource,
}
