//! AST типы для Class Diagrams (диаграмм классов).

use serde::{Deserialize, Serialize};

use crate::common::{Color, DiagramMetadata, Direction, Identifier, LineStyle, Note, Stereotype};

/// Диаграмма классов
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClassDiagram {
    /// Метаданные диаграммы
    pub metadata: DiagramMetadata,
    /// Классы и интерфейсы
    pub classifiers: Vec<Classifier>,
    /// Отношения между классами
    pub relationships: Vec<Relationship>,
    /// Пакеты
    pub packages: Vec<Package>,
    /// Заметки
    pub notes: Vec<Note>,
    /// Спрайты: `sprite $имя [ШxВ/цветов] { ...hex... }`.
    ///
    /// Раньше грамматика их принимала, но парсер отбрасывал, и в AST они
    /// не попадали — то есть содержимое библиотек иконок (logos, office,
    /// tupadr3) разбиралось и молча терялось.
    pub sprites: Vec<Sprite>,
}

/// Растровый спрайт из стандартной библиотеки.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sprite {
    /// Имя без ведущего `$`.
    pub name: String,
    /// Строки пикселей в шестнадцатеричном виде (`0`..`f` на пиксель).
    ///
    /// Пусто для векторных спрайтов — у них заполнен [`Sprite::svg`].
    pub rows: Vec<String>,
    /// Ширина в пикселях (из заголовка `[ШxВ/цветов]`).
    pub width: usize,
    /// Высота в пикселях.
    pub height: usize,
    /// Палитра в порядке индексов `0`..`f`.
    pub palette: Vec<String>,
    /// Векторное тело для формы `sprite имя <svg ...>...</svg>`.
    pub svg: Option<SvgSprite>,
}

/// Векторный спрайт — встроенный SVG.
///
/// PlantUML поддерживает лишь небольшое подмножество SVG: примитивы
/// `path`, `circle`, `ellipse`, `rect` и группировку `g` с `transform`.
/// Библиотека Archimate использует именно эту форму.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SvgSprite {
    /// Атрибуты открывающего тега `<svg ...>`; нужен `viewBox`.
    pub attrs: String,
    /// Содержимое между `<svg ...>` и `</svg>`.
    pub body: String,
}

impl ClassDiagram {
    /// Создаёт новую пустую диаграмму
    pub fn new() -> Self {
        Self::default()
    }

    /// Добавляет класс
    pub fn add_class(&mut self, class: Classifier) {
        self.classifiers.push(class);
    }

    /// Добавляет отношение
    pub fn add_relationship(&mut self, rel: Relationship) {
        self.relationships.push(rel);
    }
}

/// Тип классификатора
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ClassifierType {
    #[default]
    Class,
    Interface,
    AbstractClass,
    Enum,
    Annotation,
    Entity,
    Circle,
    Diamond,
}

impl ClassifierType {
    /// Парсит тип из строки
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "class" => Some(Self::Class),
            "interface" => Some(Self::Interface),
            "abstract" | "abstract class" => Some(Self::AbstractClass),
            "enum" => Some(Self::Enum),
            "annotation" => Some(Self::Annotation),
            "entity" => Some(Self::Entity),
            "circle" => Some(Self::Circle),
            "diamond" => Some(Self::Diamond),
            _ => None,
        }
    }
}

/// Классификатор (класс, интерфейс, enum, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classifier {
    /// Идентификатор
    pub id: Identifier,
    /// Тип классификатора
    pub classifier_type: ClassifierType,
    /// Поля
    pub fields: Vec<Member>,
    /// Методы
    pub methods: Vec<Member>,
    /// Стереотип
    pub stereotype: Option<Stereotype>,
    /// Цвет фона
    pub background_color: Option<Color>,
    /// Цвет границы
    pub border_color: Option<Color>,
    /// Обобщённые параметры (generics)
    pub generics: Option<String>,
}

impl Classifier {
    /// Создаёт новый класс
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Identifier::new(name),
            classifier_type: ClassifierType::Class,
            fields: Vec::new(),
            methods: Vec::new(),
            stereotype: None,
            background_color: None,
            border_color: None,
            generics: None,
        }
    }

    /// Создаёт интерфейс
    pub fn interface(name: impl Into<String>) -> Self {
        Self {
            classifier_type: ClassifierType::Interface,
            ..Self::new(name)
        }
    }

    /// Создаёт абстрактный класс
    pub fn abstract_class(name: impl Into<String>) -> Self {
        Self {
            classifier_type: ClassifierType::AbstractClass,
            ..Self::new(name)
        }
    }

    /// Создаёт enum
    pub fn enumeration(name: impl Into<String>) -> Self {
        Self {
            classifier_type: ClassifierType::Enum,
            ..Self::new(name)
        }
    }

    /// Добавляет поле
    pub fn add_field(&mut self, field: Member) {
        self.fields.push(field);
    }

    /// Добавляет метод
    pub fn add_method(&mut self, method: Member) {
        self.methods.push(method);
    }
}

/// Модификатор видимости
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Visibility {
    #[default]
    /// + public
    Public,
    /// - private
    Private,
    /// # protected
    Protected,
    /// ~ package
    Package,
}

impl Visibility {
    /// Парсит видимость из символа
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '+' => Some(Self::Public),
            '-' => Some(Self::Private),
            '#' => Some(Self::Protected),
            '~' => Some(Self::Package),
            _ => None,
        }
    }

    /// Возвращает символ UML
    pub fn to_char(&self) -> char {
        match self {
            Self::Public => '+',
            Self::Private => '-',
            Self::Protected => '#',
            Self::Package => '~',
        }
    }
}

/// Член класса (поле или метод)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    /// Имя
    pub name: String,
    /// Тип (опционально)
    pub member_type: Option<String>,
    /// Видимость
    pub visibility: Visibility,
    /// Статический член
    pub is_static: bool,
    /// Абстрактный член
    pub is_abstract: bool,
    /// Параметры (для методов)
    pub parameters: Vec<Parameter>,
}

impl Member {
    /// Создаёт новое поле
    pub fn field(name: impl Into<String>, member_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            member_type: Some(member_type.into()),
            visibility: Visibility::Private,
            is_static: false,
            is_abstract: false,
            parameters: Vec::new(),
        }
    }

    /// Создаёт новый метод
    pub fn method(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            member_type: None,
            visibility: Visibility::Public,
            is_static: false,
            is_abstract: false,
            parameters: Vec::new(),
        }
    }

    /// Устанавливает видимость
    pub fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Делает статическим
    pub fn make_static(mut self) -> Self {
        self.is_static = true;
        self
    }

    /// Делает абстрактным
    pub fn make_abstract(mut self) -> Self {
        self.is_abstract = true;
        self
    }
}

/// Параметр метода
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Имя параметра
    pub name: String,
    /// Тип параметра
    pub param_type: String,
}

/// Тип отношения
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipType {
    /// <|-- наследование (extension)
    Inheritance,
    /// <|.. реализация (implementation)
    Realization,
    /// *-- композиция
    Composition,
    /// o-- агрегация
    Aggregation,
    /// <-- ассоциация (направленная)
    Association,
    /// -- связь (ненаправленная)
    Link,
    /// <.. зависимость
    Dependency,
}

impl RelationshipType {
    /// Парсит тип отношения из строки
    pub fn from_arrow(arrow: &str) -> Option<Self> {
        match arrow {
            "<|--" | "--|>" => Some(Self::Inheritance),
            "<|.." | "..|>" => Some(Self::Realization),
            "*--" | "--*" => Some(Self::Composition),
            "o--" | "--o" => Some(Self::Aggregation),
            "<--" | "-->" => Some(Self::Association),
            "--" => Some(Self::Link),
            "<.." | "..>" => Some(Self::Dependency),
            ".." => Some(Self::Dependency),
            _ => None,
        }
    }
}

/// Отношение между классами
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// Источник отношения
    pub from: String,
    /// Цель отношения
    pub to: String,
    /// Тип отношения
    pub relationship_type: RelationshipType,
    /// Метка отношения
    pub label: Option<String>,
    /// Множественность источника
    pub from_cardinality: Option<String>,
    /// Множественность цели
    pub to_cardinality: Option<String>,
    /// Стиль линии
    pub line_style: LineStyle,
    /// Направление
    pub direction: Option<Direction>,
    /// Связь задана КЛЮЧЕВЫМ СЛОВОМ (`extends`/`implements`), а не стрелкой.
    ///
    /// PlantUML раскладывает эти две записи ПРОТИВОПОЛОЖНО. Проверено на
    /// сервере минимальными диаграммами:
    ///   `Dog --|> Animal`          — сверху Dog (ребёнок),
    ///   `class Dog extends Animal` — сверху Animal (родитель).
    /// Треугольник наследования в обоих случаях стоит у РОДИТЕЛЯ, то есть
    /// направление стрелки одинаково и `from`/`to` менять нельзя. Значит
    /// различие нужно хранить отдельным признаком, а разворачивать только
    /// порядок слоёв в построителе графа.
    #[serde(default)]
    pub declared_by_keyword: bool,
}

impl Relationship {
    /// Создаёт новое отношение
    pub fn new(
        from: impl Into<String>,
        to: impl Into<String>,
        relationship_type: RelationshipType,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            relationship_type,
            label: None,
            from_cardinality: None,
            to_cardinality: None,
            line_style: LineStyle::Solid,
            direction: None,
            declared_by_keyword: false,
        }
    }

    /// Помечает связь как заданную ключевым словом (`extends`/`implements`).
    ///
    /// Такие связи PlantUML раскладывает ПРОТИВОПОЛОЖНО стрелочным: родитель
    /// оказывается сверху. Направление самой стрелки при этом не меняется —
    /// треугольник в обоих случаях стоит у родителя.
    pub fn declared_by_keyword(mut self) -> Self {
        self.declared_by_keyword = true;
        self
    }

    /// Наследование
    pub fn inheritance(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::new(from, to, RelationshipType::Inheritance)
    }

    /// Реализация интерфейса
    pub fn realization(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::new(from, to, RelationshipType::Realization)
    }

    /// Композиция
    pub fn composition(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::new(from, to, RelationshipType::Composition)
    }

    /// Устанавливает метку
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Устанавливает множественность
    pub fn with_cardinality(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.from_cardinality = Some(from.into());
        self.to_cardinality = Some(to.into());
        self
    }
}

/// Пакет
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    /// Имя пакета
    pub name: String,
    /// Стереотип
    pub stereotype: Option<Stereotype>,
    /// Вложенные классификаторы
    pub classifiers: Vec<Classifier>,
    /// Вложенные пакеты
    pub packages: Vec<Package>,
    /// Цвет фона
    pub background_color: Option<Color>,
}

impl Package {
    /// Создаёт новый пакет
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            stereotype: None,
            classifiers: Vec::new(),
            packages: Vec::new(),
            background_color: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_class() {
        let mut class = Classifier::new("User");
        class.add_field(Member::field("id", "Long").with_visibility(Visibility::Private));
        class.add_field(Member::field("name", "String").with_visibility(Visibility::Private));
        class.add_method(Member::method("getId").with_visibility(Visibility::Public));

        assert_eq!(class.id.name, "User");
        assert_eq!(class.fields.len(), 2);
        assert_eq!(class.methods.len(), 1);
    }

    #[test]
    fn test_visibility() {
        assert_eq!(Visibility::from_char('+'), Some(Visibility::Public));
        assert_eq!(Visibility::from_char('-'), Some(Visibility::Private));
        assert_eq!(Visibility::from_char('#'), Some(Visibility::Protected));
        assert_eq!(Visibility::from_char('~'), Some(Visibility::Package));
        assert_eq!(Visibility::Private.to_char(), '-');
    }

    #[test]
    fn test_relationship() {
        let rel = Relationship::inheritance("Dog", "Animal").with_label("extends");

        assert_eq!(rel.from, "Dog");
        assert_eq!(rel.to, "Animal");
        assert_eq!(rel.relationship_type, RelationshipType::Inheritance);
    }
}
