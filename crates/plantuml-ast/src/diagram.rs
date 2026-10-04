//! Главный enum Diagram, объединяющий все типы диаграмм.

use serde::{Deserialize, Serialize};

use crate::activity::ActivityDiagram;
use crate::class::ClassDiagram;
use crate::common::DiagramMetadata;
use crate::component::ComponentDiagram;
use crate::er::ErDiagram;
use crate::gantt::GanttDiagram;
use crate::json::JsonDiagram;
use crate::mindmap::MindMapDiagram;
use crate::network::NetworkDiagram;
use crate::object::ObjectDiagram;
use crate::salt::SaltDiagram;
use crate::sequence::SequenceDiagram;
use crate::state::StateDiagram;
use crate::timing::TimingDiagram;
use crate::usecase::UseCaseDiagram;
use crate::wbs::WbsDiagram;
use crate::yaml::YamlDiagram;

/// Тип диаграммы PlantUML
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiagramType {
    Sequence,
    Class,
    Activity,
    State,
    Component,
    Deployment,
    UseCase,
    Object,
    Timing,
    // Non-UML
    Gantt,
    MindMap,
    Wbs,
    Json,
    Yaml,
    Network,
    Salt,
    Er,
    Archimate,
}

/// Корневой enum для всех типов диаграмм
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Diagram {
    /// Диаграмма последовательностей
    Sequence(SequenceDiagram),
    /// Диаграмма классов
    Class(ClassDiagram),
    /// Диаграмма активностей
    Activity(ActivityDiagram),
    /// Диаграмма состояний
    State(StateDiagram),
    /// Диаграмма компонентов
    Component(ComponentDiagram),
    /// Диаграмма развёртывания (использует ComponentDiagram)
    Deployment(ComponentDiagram),
    /// Диаграмма вариантов использования
    UseCase(UseCaseDiagram),
    /// Диаграмма объектов
    Object(ObjectDiagram),
    /// Временная диаграмма
    Timing(TimingDiagram),
    /// Диаграмма Ганта
    Gantt(GanttDiagram),
    /// MindMap диаграмма
    MindMap(MindMapDiagram),
    /// WBS диаграмма
    Wbs(WbsDiagram),
    /// JSON диаграмма
    Json(JsonDiagram),
    /// YAML диаграмма
    Yaml(YamlDiagram),
    /// ER диаграмма
    Er(ErDiagram),
    /// Network диаграмма (nwdiag)
    Network(NetworkDiagram),
    /// Salt диаграмма (wireframe/UI mockup)
    Salt(SaltDiagram),
    /// Archimate диаграмма (использует ComponentDiagram)
    Archimate(ComponentDiagram),
}

impl Diagram {
    /// Возвращает тип диаграммы
    pub fn diagram_type(&self) -> DiagramType {
        match self {
            Diagram::Sequence(_) => DiagramType::Sequence,
            Diagram::Class(_) => DiagramType::Class,
            Diagram::Activity(_) => DiagramType::Activity,
            Diagram::State(_) => DiagramType::State,
            Diagram::Component(_) => DiagramType::Component,
            Diagram::Deployment(_) => DiagramType::Deployment,
            Diagram::UseCase(_) => DiagramType::UseCase,
            Diagram::Object(_) => DiagramType::Object,
            Diagram::Timing(_) => DiagramType::Timing,
            Diagram::Gantt(_) => DiagramType::Gantt,
            Diagram::MindMap(_) => DiagramType::MindMap,
            Diagram::Wbs(_) => DiagramType::Wbs,
            Diagram::Json(_) => DiagramType::Json,
            Diagram::Yaml(_) => DiagramType::Yaml,
            Diagram::Er(_) => DiagramType::Er,
            Diagram::Network(_) => DiagramType::Network,
            Diagram::Salt(_) => DiagramType::Salt,
            Diagram::Archimate(_) => DiagramType::Archimate,
        }
    }

    /// Возвращает метаданные диаграммы
    pub fn metadata(&self) -> &DiagramMetadata {
        match self {
            Diagram::Sequence(d) => &d.metadata,
            Diagram::Class(d) => &d.metadata,
            Diagram::Activity(d) => &d.metadata,
            Diagram::State(d) => &d.metadata,
            Diagram::Component(d) => &d.metadata,
            Diagram::Deployment(d) => &d.metadata,
            Diagram::UseCase(d) => &d.metadata,
            Diagram::Object(d) => &d.metadata,
            Diagram::Timing(d) => &d.metadata,
            Diagram::Gantt(d) => &d.metadata,
            Diagram::MindMap(d) => &d.metadata,
            Diagram::Wbs(d) => &d.metadata,
            Diagram::Json(d) => &d.metadata,
            Diagram::Yaml(d) => &d.metadata,
            Diagram::Er(d) => &d.metadata,
            Diagram::Network(d) => &d.metadata,
            Diagram::Salt(d) => &d.metadata,
            Diagram::Archimate(d) => &d.metadata,
        }
    }
}
