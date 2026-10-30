//! AST типы для Use Case Diagrams (диаграмм вариантов использования).

use serde::{Deserialize, Serialize};

use crate::common::{ArrowShape, Color, DiagramMetadata, Direction, Note, Stereotype};

/// Диаграмма вариантов использования
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UseCaseDiagram {
    /// Метаданные диаграммы
    pub metadata: DiagramMetadata,
    /// Актёры
    pub actors: Vec<UseCaseActor>,
    /// Варианты использования
    pub use_cases: Vec<UseCase>,
    /// Связи
    pub relationships: Vec<UseCaseRelationship>,
    /// Пакеты/системы
    pub packages: Vec<UseCasePackage>,
    /// Заметки
    pub notes: Vec<Note>,
    /// Направление диаграммы
    pub direction: Direction,
}

impl UseCaseDiagram {
    /// Создаёт новую пустую диаграмму
    pub fn new() -> Self {
        Self::default()
    }
}

/// Актёр
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseCaseActor {
    /// Имя актёра
    pub name: String,
    /// Алиас
    pub alias: Option<String>,
    /// Стереотип
    pub stereotype: Option<Stereotype>,
    /// Цвет
    pub color: Option<Color>,
}

impl UseCaseActor {
    /// Создаёт нового актёра
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            alias: None,
            stereotype: None,
            color: None,
        }
    }
}

/// Вариант использования
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseCase {
    /// Имя
    pub name: String,
    /// Алиас
    pub alias: Option<String>,
    /// Стереотип
    pub stereotype: Option<Stereotype>,
    /// Цвет
    pub color: Option<Color>,
}

impl UseCase {
    /// Создаёт новый вариант использования
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            alias: None,
            stereotype: None,
            color: None,
        }
    }
}

/// Тип связи
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UseCaseRelationType {
    #[default]
    /// Простая ассоциация
    Association,
    /// Наследование
    Generalization,
    /// Include
    Include,
    /// Extend
    Extend,
}

/// Связь в диаграмме вариантов использования
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseCaseRelationship {
    /// Источник
    pub from: String,
    /// Цель
    pub to: String,
    /// Тип связи
    pub relation_type: UseCaseRelationType,
    /// Метка
    pub label: Option<String>,
    /// Форма стрелки
    pub arrow: ArrowShape,
}

impl UseCaseRelationship {
    /// Создаёт новую связь
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            relation_type: UseCaseRelationType::Association,
            label: None,
            arrow: ArrowShape::default(),
        }
    }

    /// Include связь
    pub fn include(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            relation_type: UseCaseRelationType::Include,
            label: Some("<<include>>".to_string()),
            ..Self::new(from, to)
        }
    }

    /// Extend связь
    pub fn extend(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            relation_type: UseCaseRelationType::Extend,
            label: Some("<<extend>>".to_string()),
            ..Self::new(from, to)
        }
    }
}

/// Пакет/система
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseCasePackage {
    /// Имя системы
    pub name: String,
    /// Варианты использования внутри
    pub use_cases: Vec<UseCase>,
    /// Цвет
    pub color: Option<Color>,
}

impl UseCasePackage {
    /// Создаёт новый пакет
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            use_cases: Vec::new(),
            color: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_actor() {
        let actor = UseCaseActor::new("User");
        assert_eq!(actor.name, "User");
    }

    #[test]
    fn test_create_use_case() {
        let uc = UseCase::new("Login");
        assert_eq!(uc.name, "Login");
    }

    #[test]
    fn test_include_relationship() {
        let rel = UseCaseRelationship::include("Login", "Authenticate");
        assert_eq!(rel.relation_type, UseCaseRelationType::Include);
    }
}
