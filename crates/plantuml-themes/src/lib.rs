//! # plantuml-themes
//!
//! Темы и skinparam для стилизации диаграмм PlantUML.

use serde::{Deserialize, Serialize};

/// Цвет
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color(String);

impl Color {
    /// Создаёт цвет из строки
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Возвращает CSS представление
    pub fn to_css(&self) -> String {
        // Возвращаем значение as-is: hex (#fff), rgb(...), или именованные цвета
        self.0.clone()
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::new("#000000")
    }
}

impl From<&str> for Color {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Тема оформления
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Название темы
    pub name: String,

    /// Цвет фона диаграммы
    pub background_color: Color,

    /// Цвет фона узлов
    pub node_background: Color,

    /// Цвет границы узлов
    pub node_border: Color,

    /// Цвет текста
    pub text_color: Color,

    /// Цвет стрелок
    pub arrow_color: Color,

    /// Семейство шрифтов
    pub font_family: String,

    /// Размер шрифта
    pub font_size: f64,

    /// Толщина линий
    pub line_width: f64,

    /// Радиус скругления углов
    pub corner_radius: f64,

    /// Тень
    pub shadow: bool,

    /// Рукописный стиль
    pub handwritten: bool,
}

impl Default for Theme {
    fn default() -> Self {
        // Стиль по умолчанию как в PlantUML (серо-голубые прямоугольники)
        Self {
            name: "default".to_string(),
            background_color: Color::new("#FFFFFF"),
            node_background: Color::new("#E2E2F0"), // Серо-голубой фон как в PlantUML
            node_border: Color::new("#181818"),     // Тёмная граница
            text_color: Color::new("#000000"),
            arrow_color: Color::new("#181818"),
            font_family: "sans-serif".to_string(),
            font_size: 14.0, // PlantUML использует 14 для участников
            line_width: 1.0,
            corner_radius: 2.5, // PlantUML использует rx/ry = 2.5
            shadow: false,
            handwritten: false,
        }
    }
}

impl Theme {
    /// Тема по умолчанию (monochrome)
    pub fn default_theme() -> Self {
        Self::default()
    }

    /// Классическая цветная тема PlantUML (жёлто-красная)
    pub fn classic() -> Self {
        Self {
            name: "classic".to_string(),
            background_color: Color::new("#FFFFFF"),
            node_background: Color::new("#FEFECE"),
            node_border: Color::new("#A80036"),
            text_color: Color::new("#000000"),
            arrow_color: Color::new("#A80036"),
            font_family: "sans-serif".to_string(),
            font_size: 13.0,
            line_width: 1.0,
            corner_radius: 5.0,
            shadow: true,
            handwritten: false,
        }
    }

    /// Минималистичная тема
    pub fn minimal() -> Self {
        Self {
            name: "minimal".to_string(),
            background_color: Color::new("#FFFFFF"),
            node_background: Color::new("#FFFFFF"),
            node_border: Color::new("#333333"),
            text_color: Color::new("#333333"),
            arrow_color: Color::new("#333333"),
            font_family: "Helvetica, Arial, sans-serif".to_string(),
            font_size: 12.0,
            line_width: 1.0,
            corner_radius: 0.0,
            shadow: false,
            handwritten: false,
        }
    }

    /// Тёмная тема
    pub fn dark() -> Self {
        Self {
            name: "dark".to_string(),
            background_color: Color::new("#1E1E1E"),
            node_background: Color::new("#2D2D2D"),
            node_border: Color::new("#569CD6"),
            text_color: Color::new("#D4D4D4"),
            arrow_color: Color::new("#569CD6"),
            font_family: "Consolas, monospace".to_string(),
            font_size: 13.0,
            line_width: 1.0,
            corner_radius: 3.0,
            shadow: false,
            handwritten: false,
        }
    }

    /// Рукописный стиль
    pub fn sketchy() -> Self {
        Self {
            name: "sketchy".to_string(),
            background_color: Color::new("#FFFFF0"),
            node_background: Color::new("#FFFACD"),
            node_border: Color::new("#2F4F4F"),
            text_color: Color::new("#2F4F4F"),
            arrow_color: Color::new("#2F4F4F"),
            font_family: "Comic Sans MS, cursive".to_string(),
            font_size: 14.0,
            line_width: 2.0,
            corner_radius: 8.0,
            shadow: false,
            handwritten: true,
        }
    }

    /// Cerulean (голубая)
    pub fn cerulean() -> Self {
        Self {
            name: "cerulean".to_string(),
            background_color: Color::new("#FFFFFF"),
            node_background: Color::new("#E3F2FD"),
            node_border: Color::new("#1976D2"),
            text_color: Color::new("#0D47A1"),
            arrow_color: Color::new("#1976D2"),
            font_family: "Segoe UI, Arial, sans-serif".to_string(),
            font_size: 13.0,
            line_width: 1.5,
            corner_radius: 4.0,
            shadow: true,
            handwritten: false,
        }
    }

    /// Загружает тему по имени
    pub fn by_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "default" => Some(Self::default_theme()),
            "classic" | "plantuml" => Some(Self::classic()),
            "minimal" => Some(Self::minimal()),
            "dark" => Some(Self::dark()),
            "sketchy" | "sketchy-outline" => Some(Self::sketchy()),
            "cerulean" => Some(Self::cerulean()),
            _ => None,
        }
    }
}

/// SkinParam параметры
///
/// Хранит настройки внешнего вида из `skinparam`, включая блочную форму
/// (`skinparam rectangle { ... }`). Параметры внутри блока сохраняются с
/// префиксом `<тип>.<ключ>` — так же, как их адресует PlantUML.
#[derive(Debug, Clone, Default)]
pub struct SkinParams {
    params: std::collections::HashMap<String, String>,
    /// Тип текущего блока (`rectangle`, `sequence`, ...) либо None
    current_block: Option<String>,
}

impl SkinParams {
    /// Создаёт пустые параметры
    pub fn new() -> Self {
        Self::default()
    }

    /// Устанавливает параметр
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.params.insert(key.into(), value.into());
    }

    /// Начинает блок `skinparam <section> {`
    pub fn begin_block(&mut self, section: impl Into<String>) {
        self.current_block = Some(section.into());
    }

    /// Завершает текущий блок
    pub fn end_block(&mut self) {
        self.current_block = None;
    }

    /// Открыт ли сейчас блок
    pub fn in_block(&self) -> bool {
        self.current_block.is_some()
    }

    /// Устанавливает параметр внутри блока.
    ///
    /// Сохраняется и с префиксом (`sequence.autonumber`), и без него —
    /// чтобы `apply_to` находил настройки независимо от того, заданы они
    /// в блоке или одной строкой.
    pub fn set_in_block(&mut self, key: &str, value: &str) {
        if let Some(block) = self.current_block.clone() {
            self.params
                .insert(format!("{block}.{key}"), value.to_string());
        }
        self.params.insert(key.to_string(), value.to_string());
    }

    /// Получает параметр
    pub fn get(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }

    /// Число сохранённых параметров
    pub fn len(&self) -> usize {
        self.params.len()
    }

    /// Нет ли ни одного параметра
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// Применяет параметры к теме.
    ///
    /// Поддерживаемые ключи (в скобках — форма внутри блока):
    /// `backgroundColor`, `defaultFontName`, `defaultFontSize`,
    /// `handwritten`, `shadowing`, `monochrome`,
    /// `fontColor`/`FontColor`, `linetype`, `roundCorner`/`roundcorner`,
    /// `nodesep`, `ranksep`.
    pub fn apply_to(&self, theme: &mut Theme) {
        if let Some(v) = self.get("backgroundColor") {
            theme.background_color = Color::new(v);
        }
        if let Some(v) = self.get("defaultFontName") {
            theme.font_family = v.clone();
        }
        if let Some(v) = self.get("defaultFontSize") {
            if let Ok(size) = v.parse() {
                theme.font_size = size;
            }
        }
        if let Some(v) = self.get("handwritten") {
            theme.handwritten = v == "true";
        }
        if let Some(v) = self.get("shadowing") {
            theme.shadow = v == "true";
        }
        // Монохромный режим PlantUML: фон белый, границы чёрные
        if self.get("monochrome").is_some_and(|v| v == "true") {
            theme.background_color = Color::new("#FFFFFF");
            theme.node_background = Color::new("#FFFFFF");
            theme.node_border = Color::new("#000000");
            theme.text_color = Color::new("#000000");
            theme.arrow_color = Color::new("#000000");
        }
        // Толщина линий: sequenceArrowThickness / linetype
        if let Some(v) = self
            .get("sequenceArrowThickness")
            .or_else(|| self.get("arrowThickness"))
        {
            if let Ok(w) = v.parse() {
                theme.line_width = w;
            }
        }
        // Радиус скругления углов
        if let Some(v) = self.get("roundCorner").or_else(|| self.get("roundcorner")) {
            if let Ok(r) = v.parse() {
                theme.corner_radius = r;
            }
        }
        // Цвет текста. Внутри блока ключ пишется с заглавной: FontColor
        if let Some(v) = self.get("FontColor").or_else(|| self.get("fontColor")) {
            theme.text_color = Color::new(v);
        }
        // Цвет границ внутри блока: BorderColor
        if let Some(v) = self.get("BorderColor").or_else(|| self.get("borderColor")) {
            theme.node_border = Color::new(v);
        }
        // Цвет заливки узлов внутри блока: BackgroundColor
        if let Some(v) = self
            .get("BackgroundColor")
            .or_else(|| self.get("nodeBackgroundColor"))
        {
            theme.node_background = Color::new(v);
        }
        // Толщина линий внутри блока: LineThickness
        if let Some(v) = self.get("LineThickness") {
            if let Ok(w) = v.parse() {
                theme.line_width = w;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.name, "default");
    }

    #[test]
    fn test_theme_by_name() {
        assert!(Theme::by_name("dark").is_some());
        assert!(Theme::by_name("unknown").is_none());
    }

    #[test]
    fn test_skin_params() {
        let mut params = SkinParams::new();
        params.set("backgroundColor", "#FF0000");

        let mut theme = Theme::default();
        params.apply_to(&mut theme);

        assert_eq!(theme.background_color.to_css(), "#FF0000");
    }
}
