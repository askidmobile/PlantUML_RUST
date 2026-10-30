//! AST типы для Activity Diagrams (диаграмм активностей).

use serde::{Deserialize, Serialize};

use crate::common::{Color, DiagramMetadata, Note};

/// Диаграмма активностей
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActivityDiagram {
    /// Метаданные диаграммы
    pub metadata: DiagramMetadata,
    /// Элементы диаграммы
    pub elements: Vec<ActivityElement>,
    /// Swim lanes
    pub swimlanes: Vec<Swimlane>,
    /// Предупреждения об устаревшем синтаксисе: PlantUML печатает их
    /// рамками над схемой, и пользователь должен их видеть.
    pub warnings: Vec<String>,
}

impl ActivityDiagram {
    /// Создаёт новую пустую диаграмму
    pub fn new() -> Self {
        Self::default()
    }
}

/// Элемент диаграммы активностей
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityElement {
    /// Начальный узел
    Start,
    /// Конечный узел
    Stop,
    /// Конечный узел (альтернативный)
    End,
    /// Действие
    Action(Action),
    /// Условие (if/else)
    Condition(Condition),
    /// Цикл while
    While(WhileLoop),
    /// Цикл repeat
    Repeat(RepeatLoop),
    /// Развилка (fork)
    Fork(Fork),
    /// Раздел (partition)
    Partition(Partition),
    /// Множественный выбор (switch)
    Switch(Switch),
    /// Разделение потока (split)
    Split(Split),
    /// Заметка
    Note(Note),
    /// Переход в swimlane
    SwimlaneChange(Swimlane),
    /// Соединитель
    Connector(String),
    /// Detach
    Detach,
    /// Kill
    Kill,
    /// Подпись и стиль следующей стрелки: `-> текст;`, `-[#blue]->`
    Arrow(ArrowSpec),
}

/// Подпись и оформление стрелки потока.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ArrowSpec {
    /// Текст на стрелке
    pub label: Option<String>,
    /// Цвет линии
    pub color: Option<Color>,
    /// Штриховая линия (`dashed`, `dotted`)
    pub dashed: bool,
}

/// Действие
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Текст действия
    pub label: String,
    /// Цвет фона
    pub background_color: Option<Color>,
    /// Стиль (прямоугольник, скруглённый, etc.)
    pub style: ActionStyle,
    /// Стрелка входа (опционально)
    pub arrow_label: Option<String>,
}

impl Action {
    /// Создаёт новое действие
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            background_color: None,
            style: ActionStyle::default(),
            arrow_label: None,
        }
    }
}

/// Стиль действия
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ActionStyle {
    #[default]
    /// Обычное действие :action;
    Normal,
    /// Условие `<condition>`
    Condition,
    /// Отправка сигнала >signal>
    SendSignal,
    /// Получение сигнала `<signal>`
    ReceiveSignal,
    /// `<<input>>`: пятиугольник с вырезом справа
    Input,
    /// `<<output>>`: пятиугольник с острием справа
    Output,
    /// `<<procedure>>`: прямоугольник с прямыми углами
    Procedure,
    /// `<<task>>`: прямоугольник с прямыми углами
    Task,
}

/// Условие (if/elseif/else)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Условие
    pub condition: String,
    /// Ветка then
    pub then_branch: Vec<ActivityElement>,
    /// Метка ветки then
    pub then_label: Option<String>,
    /// Ветки elseif
    pub elseif_branches: Vec<ElseIfBranch>,
    /// Ветка else
    pub else_branch: Option<Vec<ActivityElement>>,
    /// Метка ветки else
    pub else_label: Option<String>,
}

impl Condition {
    /// Создаёт новое условие
    pub fn new(condition: impl Into<String>) -> Self {
        Self {
            condition: condition.into(),
            then_branch: Vec::new(),
            then_label: None,
            elseif_branches: Vec::new(),
            else_branch: None,
            else_label: None,
        }
    }
}

/// Ветка elseif
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElseIfBranch {
    /// Условие
    pub condition: String,
    /// Элементы ветки
    pub elements: Vec<ActivityElement>,
    /// Метка
    pub label: Option<String>,
}

/// Цикл while
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhileLoop {
    /// Условие
    pub condition: String,
    /// Тело цикла
    pub body: Vec<ActivityElement>,
    /// Метка выхода
    pub end_label: Option<String>,
    /// Backward label
    pub backward_label: Option<String>,
}

impl WhileLoop {
    /// Создаёт новый цикл while
    pub fn new(condition: impl Into<String>) -> Self {
        Self {
            condition: condition.into(),
            body: Vec::new(),
            end_label: None,
            backward_label: None,
        }
    }
}

/// Цикл repeat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatLoop {
    /// Тело цикла
    pub body: Vec<ActivityElement>,
    /// Условие выхода
    pub condition: String,
    /// Backward label
    pub backward_label: Option<String>,
    /// Подпись возврата в начало: `repeat while (c) is (да)`
    pub end_label: Option<String>,
    /// Подпись выхода из цикла: `repeat while (c) is (да) not (нет)`
    pub not_label: Option<String>,
}

impl RepeatLoop {
    /// Создаёт новый цикл repeat
    pub fn new(condition: impl Into<String>) -> Self {
        Self {
            body: Vec::new(),
            condition: condition.into(),
            backward_label: None,
            end_label: None,
            not_label: None,
        }
    }
}

/// Развилка (fork/join)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fork {
    /// Параллельные ветки
    pub branches: Vec<Vec<ActivityElement>>,
    /// Тип слияния
    pub join_type: JoinType,
}

/// Раздел (`partition Имя { ... }`).
///
/// PlantUML рисует раздел рамкой с заголовком, внутри — обычный поток.
/// Раньше правило грамматики было, но ни AST, ни парсер, ни layout его не
/// знали: **всё содержимое раздела терялось**, а не только заголовок.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partition {
    /// Название раздела
    pub name: String,
    /// Цвет рамки
    pub color: Option<Color>,
    /// Содержимое раздела
    pub elements: Vec<ActivityElement>,
}

/// Ветка `switch`/`split` вместе с её условием.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    /// Условие или название ветки
    pub label: Option<String>,
    /// Содержимое ветки
    pub elements: Vec<ActivityElement>,
}

/// Множественный выбор (`switch (выражение) ... endswitch`).
///
/// Как и раздел, раньше не поддерживался вовсе — содержимое терялось.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Switch {
    /// Выражение выбора
    pub expression: Option<String>,
    /// Ветки `case`
    pub branches: Vec<Branch>,
}

/// Разделение потока (`split ... split again ... end split`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Split {
    /// Ветки
    pub branches: Vec<Branch>,
}

/// Тип слияния
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum JoinType {
    #[default]
    /// Все ветки должны завершиться
    And,
    /// Любая ветка
    Or,
}

/// Swim lane
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Swimlane {
    /// Имя swimlane
    pub name: String,
    /// Цвет
    pub color: Option<Color>,
}

impl Swimlane {
    /// Создаёт новый swimlane
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            color: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_action() {
        let action = Action::new("Process request");
        assert_eq!(action.label, "Process request");
    }

    #[test]
    fn test_create_condition() {
        let mut cond = Condition::new("Valid?");
        cond.then_branch
            .push(ActivityElement::Action(Action::new("Process")));
        cond.else_branch = Some(vec![ActivityElement::Action(Action::new("Error"))]);

        assert_eq!(cond.condition, "Valid?");
        assert_eq!(cond.then_branch.len(), 1);
    }
}
