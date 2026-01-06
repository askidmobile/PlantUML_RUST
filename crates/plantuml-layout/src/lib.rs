//! # plantuml-layout
//!
//! Layout engines для автоматического размещения элементов диаграмм.

pub mod activity;
pub mod class;
pub mod component;
pub mod config;
pub mod er;
pub mod gantt;
pub mod json;
pub mod mindmap;
pub mod network;
pub mod object;
pub mod salt;
pub mod sequence;
pub mod state;
pub mod timing;
pub mod traits;
pub mod usecase;
pub mod wbs;
pub mod yaml;

pub use activity::{ActivityLayoutConfig, ActivityLayoutEngine};
pub use class::{ClassLayoutConfig, ClassLayoutEngine};
pub use component::{ComponentLayoutConfig, ComponentLayoutEngine};
pub use config::LayoutConfig;
pub use er::{ErLayoutConfig, ErLayoutEngine};
pub use gantt::{GanttLayoutConfig, GanttLayoutEngine};
pub use json::{JsonLayoutConfig, JsonLayoutEngine};
pub use mindmap::{MindMapLayoutConfig, MindMapLayoutEngine};
pub use network::{NetworkLayoutConfig, NetworkLayoutEngine};
pub use object::{ObjectLayoutConfig, ObjectLayoutEngine};
pub use plantuml_model::{Point, Rect, Size};
pub use salt::{SaltLayoutConfig, SaltLayoutEngine};
pub use sequence::{SequenceLayoutConfig, SequenceLayoutEngine};
pub use state::{StateLayoutConfig, StateLayoutEngine};
pub use timing::{TimingLayoutConfig, TimingLayoutEngine};
pub use traits::{LayoutEngine, LayoutResult};
pub use usecase::{UseCaseLayoutConfig, UseCaseLayoutEngine};
pub use wbs::{WbsLayoutConfig, WbsLayoutEngine};
pub use yaml::{YamlLayoutConfig, YamlLayoutEngine};

/// Слои z-order для элементов диаграммы
/// Элементы с меньшим номером рендерятся ПОД элементами с большим номером
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum ZLayer {
    /// Фоновые элементы (box группировки участников)
    Background = 0,
    /// Lifelines (пунктирные вертикальные линии)
    Lifeline = 1,
    /// Activation boxes (белые прямоугольники на lifeline)
    Activation = 2,
    /// Стрелки сообщений
    #[default]
    Message = 3,
    /// Фреймы фрагментов (alt, opt, loop) — рамки
    FragmentFrame = 4,
    /// Labels фрагментов и текст сообщений
    FragmentLabel = 5,
    /// Заметки (notes)
    Note = 6,
    /// Участники (header/footer блоки) — самые верхние
    Participant = 7,
}

impl ZLayer {
    /// Определяет z-layer по типу элемента и его id
    pub fn from_element(element: &LayoutElement) -> Self {
        match &element.element_type {
            ElementType::ParticipantBox => ZLayer::Background,
            ElementType::Activation => ZLayer::Activation,
            // Lifeline: пунктирная линия без стрелок
            ElementType::Edge { dashed: true, arrow_end: false, arrow_start: false, .. } 
                if element.id.starts_with("lifeline_") => ZLayer::Lifeline,
            // Обычные сообщения (edges)
            ElementType::Edge { .. } => ZLayer::Message,
            // Fragment
            ElementType::Fragment { .. } => ZLayer::FragmentFrame,
            // Участники (header и footer)
            ElementType::Rectangle { .. } 
                if element.id.starts_with("participant_") || element.id.starts_with("footer_") 
                => ZLayer::Participant,
            // Заметки
            ElementType::Rectangle { .. } if element.id.starts_with("note_") => ZLayer::Note,
            // Текст
            ElementType::Text { .. } => ZLayer::FragmentLabel,
            // Всё остальное — дефолт
            _ => ZLayer::Message,
        }
    }
}

/// Элемент результата layout
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutElement {
    /// Уникальный ID элемента
    pub id: String,
    /// Bounding box элемента
    pub bounds: Rect,
    /// Тип элемента (для рендеринга)
    pub element_type: ElementType,
    /// Текст элемента (опционально)
    pub text: Option<String>,
    /// Дополнительные свойства
    pub properties: std::collections::HashMap<String, String>,
}

impl LayoutElement {
    /// Создаёт новый элемент layout
    pub fn new(id: impl Into<String>, bounds: Rect, element_type: ElementType) -> Self {
        Self {
            id: id.into(),
            bounds,
            element_type,
            text: None,
            properties: std::collections::HashMap::new(),
        }
    }

    /// Добавляет текст к элементу
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Добавляет свойство
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }
}

impl Default for ElementType {
    fn default() -> Self {
        Self::Rectangle {
            label: String::new(),
            corner_radius: 0.0,
        }
    }
}

/// Тип связи для Edge (определяет форму маркеров)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EdgeType {
    /// Обычная связь (сплошная линия, заполненная стрелка)
    #[default]
    Association,
    /// Наследование (сплошная линия, пустой треугольник) --|>
    Inheritance,
    /// Реализация (пунктирная линия, пустой треугольник) ..|>
    Realization,
    /// Композиция (сплошная линия, закрашенный ромб) *--
    Composition,
    /// Агрегация (сплошная линия, пустой ромб) o--
    Aggregation,
    /// Зависимость (пунктирная линия, открытая стрелка) ..>
    Dependency,
    /// Простая линия без маркеров --
    Link,
}

/// Тип элемента layout
#[derive(Debug, Clone, PartialEq)]
pub enum ElementType {
    /// Прямоугольник (класс, участник, etc.)
    Rectangle { label: String, corner_radius: f64 },
    /// Прямоугольник со скруглёнными углами (MindMap узлы)
    RoundedRectangle,
    /// Эллипс (начальное/конечное состояние)
    Ellipse { label: Option<String> },
    /// UML Initial State (чёрный заполненный круг)
    InitialState,
    /// UML Final State (bullseye: круг в круге)
    FinalState,
    /// UML State (прямоугольник со скруглёнными углами и разделителем)
    State {
        /// Название состояния
        name: String,
        /// Описание (entry/exit/do actions)
        description: Option<String>,
    },
    /// UML Composite State (контейнер с заголовком и вложенными состояниями)
    CompositeState {
        /// Название состояния
        name: String,
        /// Высота заголовка
        header_height: f64,
    },
    /// Актёр (stick figure) для UseCase диаграмм
    Actor { label: String },
    /// Система/пакет для UseCase диаграмм (прямоугольник с заголовком сверху)
    System { title: String },
    /// Линия/стрелка
    Edge {
        points: Vec<Point>,
        label: Option<String>,
        arrow_start: bool,
        arrow_end: bool,
        /// Пунктирная линия (для lifelines и realization/dependency)
        dashed: bool,
        /// Тип связи (определяет форму маркеров)
        edge_type: EdgeType,
        /// Кардинальность у начальной точки (например "1")
        from_cardinality: Option<String>,
        /// Кардинальность у конечной точки (например "*")
        to_cardinality: Option<String>,
    },
    /// SVG Path (для кривых Безье, etc.)
    Path,
    /// Текст
    Text { text: String, font_size: f64 },
    /// Группа (пакет, фрагмент) — устаревший, используйте Fragment
    Group {
        label: Option<String>,
        children: Vec<LayoutElement>,
    },
    /// Combined Fragment (alt, opt, loop, etc.) для sequence diagrams
    /// Рендерится как PlantUML: сплошная рамка + пятиугольник заголовка + разделители else
    Fragment {
        /// Тип фрагмента (alt, opt, loop, etc.)
        fragment_type: String,
        /// Секции фрагмента с условиями и дочерними элементами
        sections: Vec<FragmentSection>,
    },
    /// Activation box для sequence diagrams (белый фон)
    Activation,
    /// Class/Interface/Enum box с зонами (PlantUML style)
    ClassBox {
        /// Тип классификатора (Class, Interface, Abstract, Enum, etc.)
        classifier_type: ClassifierKind,
        /// Название класса
        name: String,
        /// Стереотип (например: «interface», «abstract»)
        stereotype: Option<String>,
        /// Поля класса
        fields: Vec<ClassMember>,
        /// Методы класса
        methods: Vec<ClassMember>,
    },
    /// Participant Box для sequence diagrams (фоновая группировка)
    /// Рендерится как цветной прямоугольник с заголовком сверху
    ParticipantBox,
}

/// Тип классификатора для ClassBox
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClassifierKind {
    #[default]
    Class,
    Interface,
    AbstractClass,
    Enum,
    Annotation,
    Entity,
}

/// Член класса (поле или метод) с видимостью
#[derive(Debug, Clone, PartialEq)]
pub struct ClassMember {
    /// Видимость (+, -, #, ~)
    pub visibility: MemberVisibility,
    /// Текст члена (например: "name: String" или "getName()")
    pub text: String,
    /// Статический член (подчёркивание)
    pub is_static: bool,
    /// Абстрактный член (курсив)
    pub is_abstract: bool,
}

impl ClassMember {
    pub fn new(visibility: MemberVisibility, text: impl Into<String>) -> Self {
        Self {
            visibility,
            text: text.into(),
            is_static: false,
            is_abstract: false,
        }
    }
}

/// Видимость члена класса
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MemberVisibility {
    #[default]
    Public,      // +
    Private,     // -
    Protected,   // #
    Package,     // ~
}

impl MemberVisibility {
    /// Возвращает символ видимости
    pub fn symbol(&self) -> char {
        match self {
            Self::Public => '+',
            Self::Private => '-',
            Self::Protected => '#',
            Self::Package => '~',
        }
    }
}

/// Секция фрагмента (например: условие "Успешная авторизация" + элементы внутри)
#[derive(Debug, Clone, PartialEq)]
pub struct FragmentSection {
    /// Условие секции (например: "Успешная авторизация")
    pub condition: Option<String>,
    /// Y-координата начала секции
    pub start_y: f64,
    /// Y-координата конца секции
    pub end_y: f64,
    /// Дочерние элементы
    pub children: Vec<LayoutElement>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_element() {
        let elem = LayoutElement::new(
            "test",
            Rect::new(0.0, 0.0, 100.0, 50.0),
            ElementType::Rectangle {
                label: "Test".to_string(),
                corner_radius: 5.0,
            },
        );

        assert_eq!(elem.id, "test");
    }
}
