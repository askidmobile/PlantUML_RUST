//! AST типы для Component и Deployment Diagrams.

use serde::{Deserialize, Serialize};

use crate::common::{Color, DiagramMetadata, Note, Stereotype};

/// Диаграмма компонентов/развёртывания
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentDiagram {
    /// Метаданные диаграммы
    pub metadata: DiagramMetadata,
    /// Компоненты
    pub components: Vec<Component>,
    /// Связи
    pub connections: Vec<Connection>,
    /// Пакеты/контейнеры
    pub packages: Vec<ComponentPackage>,
    /// Заметки
    pub notes: Vec<Note>,
}

impl ComponentDiagram {
    /// Создаёт новую пустую диаграмму
    pub fn new() -> Self {
        Self::default()
    }
}

/// Тип компонента
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ComponentType {
    #[default]
    /// [Component]
    Component,
    /// Интерфейс ()
    Interface,
    /// База данных
    Database,
    /// Очередь
    Queue,
    /// Папка
    Folder,
    /// Фрейм
    Frame,
    /// Облако
    Cloud,
    /// Узел (node)
    Node,
    /// Прямоугольник
    Rectangle,
    /// Actor
    Actor,
    /// Артефакт
    Artifact,
    /// Файл
    File,
    /// Storage
    Storage,
    /// Card
    Card,
    /// Hexagon
    Hexagon,
    /// Stack
    Stack,
    /// Port
    Port,
    /// Device (физическое устройство для Deployment Diagram)
    Device,
    /// Agent (исполняемый агент для Deployment Diagram)
    Agent,
    /// Control (управляющий компонент)
    Control,
    /// Boundary (граница системы)
    Boundary,
    /// Entity (сущность)
    Entity,
    /// Collections (коллекция)
    Collections,
}

impl ComponentType {
    /// Парсит тип из строки
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "component" => Some(Self::Component),
            "interface" => Some(Self::Interface),
            "database" => Some(Self::Database),
            "queue" => Some(Self::Queue),
            "folder" => Some(Self::Folder),
            "frame" => Some(Self::Frame),
            "cloud" => Some(Self::Cloud),
            "node" => Some(Self::Node),
            "rectangle" => Some(Self::Rectangle),
            "actor" => Some(Self::Actor),
            "artifact" => Some(Self::Artifact),
            "file" => Some(Self::File),
            "storage" => Some(Self::Storage),
            "card" => Some(Self::Card),
            "hexagon" => Some(Self::Hexagon),
            "stack" => Some(Self::Stack),
            "port" => Some(Self::Port),
            "device" => Some(Self::Device),
            "agent" => Some(Self::Agent),
            "control" => Some(Self::Control),
            "boundary" => Some(Self::Boundary),
            "entity" => Some(Self::Entity),
            "collections" => Some(Self::Collections),
            _ => None,
        }
    }
}

/// Компонент
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    /// Имя компонента
    pub name: String,
    /// Алиас
    pub alias: Option<String>,
    /// Тип компонента
    pub component_type: ComponentType,
    /// Стереотип
    pub stereotype: Option<Stereotype>,
    /// Цвет
    pub color: Option<Color>,
    /// Вложенные компоненты
    pub children: Vec<Component>,
    /// Интерфейсы
    pub interfaces: Vec<ComponentInterface>,
    /// Порты
    pub ports: Vec<Port>,
}

impl Component {
    /// Создаёт новый компонент
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            alias: None,
            component_type: ComponentType::Component,
            stereotype: None,
            color: None,
            children: Vec::new(),
            interfaces: Vec::new(),
            ports: Vec::new(),
        }
    }

    /// Создаёт базу данных
    pub fn database(name: impl Into<String>) -> Self {
        Self {
            component_type: ComponentType::Database,
            ..Self::new(name)
        }
    }

    /// Создаёт узел
    pub fn node(name: impl Into<String>) -> Self {
        Self {
            component_type: ComponentType::Node,
            ..Self::new(name)
        }
    }

    /// Создаёт облако
    pub fn cloud(name: impl Into<String>) -> Self {
        Self {
            component_type: ComponentType::Cloud,
            ..Self::new(name)
        }
    }
}

/// Интерфейс компонента
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInterface {
    /// Имя интерфейса
    pub name: String,
    /// Алиас
    pub alias: Option<String>,
    /// Предоставляемый (true) или требуемый (false)
    pub provided: bool,
}

impl ComponentInterface {
    /// Создаёт предоставляемый интерфейс
    pub fn provided(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            alias: None,
            provided: true,
        }
    }

    /// Создаёт требуемый интерфейс
    pub fn required(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            alias: None,
            provided: false,
        }
    }
}

/// Порт
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    /// Имя порта
    pub name: String,
}

/// Тип связи
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConnectionType {
    #[default]
    /// Простая связь
    Simple,
    /// Зависимость
    Dependency,
    /// Использование
    Use,
    /// Включение
    Include,
}

/// Связь между компонентами
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    /// Источник
    pub from: String,
    /// Цель
    pub to: String,
    /// Тип связи
    pub connection_type: ConnectionType,
    /// Метка
    pub label: Option<String>,
    /// Цвет
    pub color: Option<Color>,
    /// Пунктирная линия
    pub dashed: bool,
}

impl Connection {
    /// Создаёт новую связь
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            connection_type: ConnectionType::Simple,
            label: None,
            color: None,
            dashed: false,
        }
    }

    /// Устанавливает метку
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Делает пунктирной
    pub fn dashed(mut self) -> Self {
        self.dashed = true;
        self
    }
}

/// Пакет/контейнер для компонентов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentPackage {
    /// Имя пакета
    pub name: String,
    /// Тип контейнера
    pub package_type: PackageType,
    /// Стереотип
    pub stereotype: Option<Stereotype>,
    /// Цвет
    pub color: Option<Color>,
    /// Вложенные компоненты
    pub components: Vec<Component>,
    /// Вложенные пакеты
    pub packages: Vec<ComponentPackage>,
}

/// Тип контейнера
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PackageType {
    #[default]
    Package,
    Node,
    Folder,
    Frame,
    Cloud,
    Rectangle,
    /// База данных (`database "Имя" { ... }`).
    ///
    /// Измерено по эталону `Component: Простой`: переход в `database`
    /// даёт шаг 137 против 122 при переходе в обычный `package`.
    Database,
}

impl ComponentPackage {
    /// Создаёт новый пакет
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            package_type: PackageType::Package,
            stereotype: None,
            color: None,
            components: Vec::new(),
            packages: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_component() {
        let comp = Component::new("API Gateway");
        assert_eq!(comp.name, "API Gateway");
        assert_eq!(comp.component_type, ComponentType::Component);

        let db = Component::database("PostgreSQL");
        assert_eq!(db.component_type, ComponentType::Database);
    }

    #[test]
    fn test_create_connection() {
        let conn = Connection::new("App", "Database").with_label("uses");

        assert_eq!(conn.from, "App");
        assert_eq!(conn.to, "Database");
        assert_eq!(conn.label, Some("uses".to_string()));
    }
}
