//! AST типы для ER (Entity-Relationship) диаграмм
//!
//! ER диаграммы описывают структуру базы данных: сущности, атрибуты и связи.
//!
//! Синтаксис PlantUML (IE notation):
//! ```text
//! @startuml
//! entity User {
//!   * id : int <<PK>>
//!   --
//!   name : varchar
//!   email : varchar
//! }
//!
//! entity Order {
//!   * id : int <<PK>>
//!   --
//!   * user_id : int <<FK>>
//!   total : decimal
//! }
//!
//! User ||--o{ Order : places
//! @enduml
//! ```

use serde::{Deserialize, Serialize};

use crate::common::{Color, DiagramMetadata, Identifier, Note, Stereotype};

/// ER диаграмма
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErDiagram {
    /// Метаданные диаграммы
    pub metadata: DiagramMetadata,
    /// Сущности
    pub entities: Vec<Entity>,
    /// Связи между сущностями
    pub relationships: Vec<ErRelationship>,
    /// Заметки
    pub notes: Vec<Note>,
}

impl ErDiagram {
    /// Создаёт новую пустую ER диаграмму
    pub fn new() -> Self {
        Self::default()
    }

    /// Добавляет сущность
    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    /// Добавляет связь
    pub fn add_relationship(&mut self, rel: ErRelationship) {
        self.relationships.push(rel);
    }

    /// Находит сущность по имени
    pub fn find_entity(&self, name: &str) -> Option<&Entity> {
        self.entities.iter().find(|e| e.id.name == name)
    }
}

/// Сущность (Entity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Идентификатор сущности
    pub id: Identifier,
    /// Атрибуты (поля таблицы)
    pub attributes: Vec<Attribute>,
    /// Стереотип
    pub stereotype: Option<Stereotype>,
    /// Цвет фона
    pub background_color: Option<Color>,
    /// Является ли сущность слабой (weak entity)
    pub is_weak: bool,
}

impl Entity {
    /// Создаёт новую сущность
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Identifier::new(name),
            attributes: Vec::new(),
            stereotype: None,
            background_color: None,
            is_weak: false,
        }
    }

    /// Добавляет атрибут
    pub fn add_attribute(&mut self, attr: Attribute) {
        self.attributes.push(attr);
    }

    /// Возвращает первичные ключи
    pub fn primary_keys(&self) -> Vec<&Attribute> {
        self.attributes.iter().filter(|a| a.is_primary_key).collect()
    }

    /// Возвращает внешние ключи
    pub fn foreign_keys(&self) -> Vec<&Attribute> {
        self.attributes.iter().filter(|a| a.is_foreign_key).collect()
    }

    /// Устанавливает слабую сущность
    pub fn with_weak(mut self, is_weak: bool) -> Self {
        self.is_weak = is_weak;
        self
    }
}

/// Атрибут сущности (колонка таблицы)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    /// Имя атрибута
    pub name: String,
    /// Тип данных
    pub data_type: Option<String>,
    /// Является ли первичным ключом
    pub is_primary_key: bool,
    /// Является ли внешним ключом
    pub is_foreign_key: bool,
    /// Обязательное поле (NOT NULL)
    pub is_required: bool,
    /// Уникальное значение
    pub is_unique: bool,
    /// Комментарий
    pub comment: Option<String>,
    /// Стереотип (PK, FK, UK, etc.)
    pub stereotype: Option<String>,
}

impl Attribute {
    /// Создаёт новый атрибут
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data_type: None,
            is_primary_key: false,
            is_foreign_key: false,
            is_required: false,
            is_unique: false,
            comment: None,
            stereotype: None,
        }
    }

    /// Устанавливает тип данных
    pub fn with_type(mut self, data_type: impl Into<String>) -> Self {
        self.data_type = Some(data_type.into());
        self
    }

    /// Помечает как первичный ключ
    pub fn as_primary_key(mut self) -> Self {
        self.is_primary_key = true;
        self.is_required = true;
        self.stereotype = Some("PK".to_string());
        self
    }

    /// Помечает как внешний ключ
    pub fn as_foreign_key(mut self) -> Self {
        self.is_foreign_key = true;
        self.stereotype = Some("FK".to_string());
        self
    }

    /// Помечает как обязательное
    pub fn as_required(mut self) -> Self {
        self.is_required = true;
        self
    }
}

/// Связь между сущностями
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErRelationship {
    /// Исходная сущность
    pub from: String,
    /// Целевая сущность
    pub to: String,
    /// Кардинальность со стороны from
    pub from_cardinality: Cardinality,
    /// Кардинальность со стороны to
    pub to_cardinality: Cardinality,
    /// Метка связи (название)
    pub label: Option<String>,
    /// Идентифицирующая связь (identifying relationship)
    pub is_identifying: bool,
}

impl ErRelationship {
    /// Создаёт новую связь
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            from_cardinality: Cardinality::One,
            to_cardinality: Cardinality::Many,
            label: None,
            is_identifying: false,
        }
    }

    /// Устанавливает кардинальности
    pub fn with_cardinality(mut self, from: Cardinality, to: Cardinality) -> Self {
        self.from_cardinality = from;
        self.to_cardinality = to;
        self
    }

    /// Устанавливает метку
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Устанавливает идентифицирующую связь
    pub fn as_identifying(mut self) -> Self {
        self.is_identifying = true;
        self
    }
}

/// Кардинальность связи
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Cardinality {
    /// Ровно один (||)
    #[default]
    One,
    /// Ноль или один (|o)
    ZeroOrOne,
    /// Много (}|, |{)
    Many,
    /// Ноль или много (}o, o{)
    ZeroOrMany,
    /// Один или много (}|, |{)
    OneOrMany,
}

impl Cardinality {
    /// Парсит кардинальность из символа IE нотации
    /// || = One
    /// |o или o| = ZeroOrOne
    /// }| или |{ = OneOrMany
    /// }o или o{ = ZeroOrMany
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "||" | "|" => Some(Self::One),
            "|o" | "o|" => Some(Self::ZeroOrOne),
            "}|" | "|{" => Some(Self::OneOrMany),
            "}o" | "o{" => Some(Self::ZeroOrMany),
            "}" | "{" => Some(Self::Many),
            _ => None,
        }
    }

    /// Возвращает символ для отображения
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::One => "||",
            Self::ZeroOrOne => "|o",
            Self::Many => "}|",
            Self::ZeroOrMany => "}o",
            Self::OneOrMany => "}|",
        }
    }

    /// Является ли опциональной связью
    pub fn is_optional(&self) -> bool {
        matches!(self, Self::ZeroOrOne | Self::ZeroOrMany)
    }

    /// Может ли быть много
    pub fn is_many(&self) -> bool {
        matches!(self, Self::Many | Self::ZeroOrMany | Self::OneOrMany)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_entity() {
        let mut entity = Entity::new("User");
        entity.add_attribute(Attribute::new("id").with_type("int").as_primary_key());
        entity.add_attribute(Attribute::new("name").with_type("varchar"));

        assert_eq!(entity.id.name, "User");
        assert_eq!(entity.attributes.len(), 2);
        assert_eq!(entity.primary_keys().len(), 1);
    }

    #[test]
    fn test_create_relationship() {
        let rel = ErRelationship::new("User", "Order")
            .with_cardinality(Cardinality::One, Cardinality::ZeroOrMany)
            .with_label("places");

        assert_eq!(rel.from, "User");
        assert_eq!(rel.to, "Order");
        assert_eq!(rel.label, Some("places".to_string()));
    }

    #[test]
    fn test_cardinality_parse() {
        assert_eq!(Cardinality::parse("||"), Some(Cardinality::One));
        assert_eq!(Cardinality::parse("|o"), Some(Cardinality::ZeroOrOne));
        assert_eq!(Cardinality::parse("}|"), Some(Cardinality::OneOrMany));
        assert_eq!(Cardinality::parse("}o"), Some(Cardinality::ZeroOrMany));
    }

    #[test]
    fn test_er_diagram() {
        let mut diagram = ErDiagram::new();
        diagram.add_entity(Entity::new("User"));
        diagram.add_entity(Entity::new("Order"));
        diagram.add_relationship(ErRelationship::new("User", "Order"));

        assert_eq!(diagram.entities.len(), 2);
        assert_eq!(diagram.relationships.len(), 1);
    }
}
