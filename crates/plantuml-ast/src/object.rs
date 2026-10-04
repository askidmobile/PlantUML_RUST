//! AST типы для Object Diagrams (диаграмм объектов).
//!
//! Object Diagram показывает экземпляры объектов и их связи
//! в определённый момент времени.

use serde::{Deserialize, Serialize};

use crate::common::{Color, DiagramMetadata, Note, Stereotype};

/// Диаграмма объектов
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObjectDiagram {
    /// Метаданные диаграммы
    pub metadata: DiagramMetadata,
    /// Объекты
    pub objects: Vec<Object>,
    /// Связи между объектами
    pub links: Vec<ObjectLink>,
    /// Заметки
    pub notes: Vec<Note>,
}

impl ObjectDiagram {
    /// Создаёт новую пустую диаграмму
    pub fn new() -> Self {
        Self::default()
    }

    /// Добавляет объект
    pub fn add_object(&mut self, object: Object) {
        self.objects.push(object);
    }

    /// Добавляет связь
    pub fn add_link(&mut self, link: ObjectLink) {
        self.links.push(link);
    }
}

/// Объект (экземпляр класса)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    /// Имя объекта (например, "user1" или "user1 : User")
    pub name: String,
    /// Тип/класс объекта (опционально)
    pub class_name: Option<String>,
    /// Значения полей
    pub fields: Vec<ObjectField>,
    /// Стереотип
    pub stereotype: Option<Stereotype>,
    /// Цвет фона
    pub background_color: Option<Color>,
}

impl Object {
    /// Создаёт новый объект
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            class_name: None,
            fields: Vec::new(),
            stereotype: None,
            background_color: None,
        }
    }

    /// Создаёт объект с типом класса
    pub fn with_class(name: impl Into<String>, class_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            class_name: Some(class_name.into()),
            fields: Vec::new(),
            stereotype: None,
            background_color: None,
        }
    }

    /// Добавляет поле
    pub fn add_field(&mut self, field: ObjectField) {
        self.fields.push(field);
    }

    /// Возвращает полное имя для отображения (name : ClassName)
    pub fn display_name(&self) -> String {
        match &self.class_name {
            Some(class) => format!("{} : {}", self.name, class),
            None => self.name.clone(),
        }
    }
}

/// Поле объекта со значением
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectField {
    /// Имя поля
    pub name: String,
    /// Значение поля
    pub value: String,
}

impl ObjectField {
    /// Создаёт новое поле
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Связь между объектами
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectLink {
    /// Источник связи
    pub from: String,
    /// Цель связи
    pub to: String,
    /// Метка связи
    pub label: Option<String>,
    /// Тип связи
    pub link_type: ObjectLinkType,
}

impl ObjectLink {
    /// Создаёт новую связь
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            label: None,
            link_type: ObjectLinkType::Association,
        }
    }

    /// Устанавливает метку
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Устанавливает тип связи
    pub fn with_type(mut self, link_type: ObjectLinkType) -> Self {
        self.link_type = link_type;
        self
    }
}

/// Тип связи между объектами
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ObjectLinkType {
    /// --> направленная ассоциация
    #[default]
    Association,
    /// -- ненаправленная связь
    Link,
    /// ..> зависимость
    Dependency,
    /// *-- композиция
    Composition,
    /// o-- агрегация
    Aggregation,
}

impl ObjectLinkType {
    /// Парсит тип связи из строки стрелки
    pub fn from_arrow(arrow: &str) -> Self {
        match arrow {
            "-->" | "<--" => Self::Association,
            "--" => Self::Link,
            "..>" | "<.." | ".." => Self::Dependency,
            "*--" | "--*" => Self::Composition,
            "o--" | "--o" => Self::Aggregation,
            _ => Self::Association,
        }
    }

    /// Является ли связь пунктирной
    pub fn is_dashed(&self) -> bool {
        matches!(self, Self::Dependency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_object() {
        let mut obj = Object::new("user1");
        obj.add_field(ObjectField::new("name", "\"John\""));
        obj.add_field(ObjectField::new("age", "30"));

        assert_eq!(obj.name, "user1");
        assert_eq!(obj.fields.len(), 2);
        assert_eq!(obj.display_name(), "user1");
    }

    #[test]
    fn test_object_with_class() {
        let obj = Object::with_class("user1", "User");
        assert_eq!(obj.display_name(), "user1 : User");
    }

    #[test]
    fn test_object_link() {
        let link = ObjectLink::new("user1", "user2").with_label("friend");

        assert_eq!(link.from, "user1");
        assert_eq!(link.to, "user2");
        assert_eq!(link.label, Some("friend".to_string()));
    }

    #[test]
    fn test_link_type_from_arrow() {
        assert_eq!(
            ObjectLinkType::from_arrow("-->"),
            ObjectLinkType::Association
        );
        assert_eq!(ObjectLinkType::from_arrow("--"), ObjectLinkType::Link);
        assert_eq!(
            ObjectLinkType::from_arrow("..>"),
            ObjectLinkType::Dependency
        );
    }
}
