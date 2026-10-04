//! Общие типы, используемые во всех типах диаграмм.

use serde::{Deserialize, Serialize};

/// Позиция в исходном коде
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// Начальная позиция (байт)
    pub start: usize,
    /// Конечная позиция (байт)
    pub end: usize,
    /// Номер строки (1-indexed)
    pub line: usize,
    /// Номер колонки (1-indexed)
    pub column: usize,
}

impl Span {
    /// Создаёт новый Span
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }

    /// Создаёт пустой Span (для сгенерированных узлов)
    pub fn empty() -> Self {
        Self {
            start: 0,
            end: 0,
            line: 0,
            column: 0,
        }
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::empty()
    }
}

/// Идентификатор элемента диаграммы
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Identifier {
    /// Имя идентификатора
    pub name: String,
    /// Алиас (опционально)
    pub alias: Option<String>,
}

impl Identifier {
    /// Создаёт новый идентификатор
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            alias: None,
        }
    }

    /// Создаёт идентификатор с алиасом
    pub fn with_alias(name: impl Into<String>, alias: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            alias: Some(alias.into()),
        }
    }

    /// Возвращает отображаемое имя (алиас или имя)
    pub fn display_name(&self) -> &str {
        self.alias.as_deref().unwrap_or(&self.name)
    }
}

impl From<&str> for Identifier {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for Identifier {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

/// Цвет в различных форматах
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Color {
    /// Именованный цвет (red, blue, etc.)
    Named(String),
    /// HEX формат (#RRGGBB или #RGB)
    Hex(String),
    /// RGB формат
    Rgb { r: u8, g: u8, b: u8 },
    /// RGBA формат
    Rgba { r: u8, g: u8, b: u8, a: u8 },
}

impl Color {
    /// Создаёт цвет из HEX строки
    pub fn from_hex(hex: impl Into<String>) -> Self {
        Self::Hex(hex.into())
    }

    /// Создаёт именованный цвет
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
    }

    /// Парсит цвет из строки (умно определяет тип)
    /// Принимает: "#FF0000", "#F00", "FF0000", "red", "#red", "#LightBlue"
    pub fn parse(s: impl Into<String>) -> Self {
        let s = s.into();
        let trimmed = s.trim_start_matches('#');

        // Если после удаления # остались только hex символы и длина 3, 6 или 8 - это hex
        let is_hex = trimmed.len() >= 3
            && trimmed.len() <= 8
            && trimmed.chars().all(|c| c.is_ascii_hexdigit());

        if is_hex {
            Self::Hex(trimmed.to_string())
        } else {
            // Это именованный цвет
            Self::Named(trimmed.to_string())
        }
    }

    /// Преобразует в CSS строку
    pub fn to_css(&self) -> String {
        match self {
            Color::Named(name) => name.clone(),
            Color::Hex(hex) => {
                if hex.starts_with('#') {
                    hex.clone()
                } else {
                    format!("#{}", hex)
                }
            }
            Color::Rgb { r, g, b } => format!("rgb({}, {}, {})", r, g, b),
            Color::Rgba { r, g, b, a } => {
                format!("rgba({}, {}, {}, {})", r, g, b, *a as f32 / 255.0)
            }
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::Named("black".to_string())
    }
}

/// Стереотип элемента <<stereotype>>
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stereotype {
    /// Имена стереотипов
    pub names: Vec<String>,
    /// Цвет фона (опционально)
    pub background_color: Option<Color>,
}

impl Stereotype {
    /// Создаёт новый стереотип
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            names: vec![name.into()],
            background_color: None,
        }
    }

    /// Создаёт стереотип с несколькими именами
    pub fn multiple(names: Vec<String>) -> Self {
        Self {
            names,
            background_color: None,
        }
    }
}

/// Заметка (note)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    /// Позиция заметки
    pub position: NotePosition,
    /// Текст заметки
    pub text: String,
    /// Привязка к элементам (опционально)
    pub anchors: Vec<String>,
    /// Цвет фона
    pub background_color: Option<Color>,
}

/// Позиция заметки
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotePosition {
    Left,
    Right,
    Top,
    Bottom,
    Over,
}

/// Направление диаграммы
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Direction {
    #[default]
    TopToBottom,
    BottomToTop,
    LeftToRight,
    RightToLeft,
}

/// Стиль линии
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LineStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
    Bold,
}

/// Метаданные диаграммы
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DiagramMetadata {
    /// Заголовок диаграммы
    pub title: Option<String>,
    /// Подпись
    pub caption: Option<String>,
    /// Легенда
    pub legend: Option<String>,
    /// Верхний колонтитул
    pub header: Option<String>,
    /// Нижний колонтитул
    pub footer: Option<String>,
    /// Масштаб
    pub scale: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_display_name() {
        let id = Identifier::new("Alice");
        assert_eq!(id.display_name(), "Alice");

        let id_with_alias = Identifier::with_alias("Alice", "A");
        assert_eq!(id_with_alias.display_name(), "A");
    }

    #[test]
    fn test_color_to_css() {
        assert_eq!(Color::named("red").to_css(), "red");
        assert_eq!(Color::from_hex("#FF0000").to_css(), "#FF0000");
        assert_eq!(Color::from_hex("FF0000").to_css(), "#FF0000");
        assert_eq!(Color::Rgb { r: 255, g: 0, b: 0 }.to_css(), "rgb(255, 0, 0)");
    }

    #[test]
    fn test_color_parse() {
        // Hex цвета
        assert_eq!(Color::parse("#FF0000").to_css(), "#FF0000");
        assert_eq!(Color::parse("FF0000").to_css(), "#FF0000");
        assert_eq!(Color::parse("#FFF").to_css(), "#FFF");
        assert_eq!(Color::parse("ABC").to_css(), "#ABC");
        assert_eq!(Color::parse("#AABBCCDD").to_css(), "#AABBCCDD");

        // Именованные цвета
        assert_eq!(Color::parse("red").to_css(), "red");
        assert_eq!(Color::parse("#red").to_css(), "red");
        assert_eq!(Color::parse("LightBlue").to_css(), "LightBlue");
        assert_eq!(Color::parse("#LightBlue").to_css(), "LightBlue");
        assert_eq!(Color::parse("DarkGreen").to_css(), "DarkGreen");
    }
}
