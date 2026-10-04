//! AST типы для State Diagrams (диаграмм состояний).

use serde::{Deserialize, Serialize};

use crate::common::{Color, DiagramMetadata, Note, Stereotype};

/// Диаграмма состояний
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StateDiagram {
    /// Метаданные диаграммы
    pub metadata: DiagramMetadata,
    /// Состояния верхнего уровня
    pub states: Vec<State>,
    /// Переходы
    pub transitions: Vec<Transition>,
    /// Заметки
    pub notes: Vec<Note>,
}

impl StateDiagram {
    /// Создаёт новую пустую диаграмму
    pub fn new() -> Self {
        Self::default()
    }

    /// Добавляет состояние
    pub fn add_state(&mut self, state: State) {
        self.states.push(state);
    }

    /// Добавляет переход
    pub fn add_transition(&mut self, transition: Transition) {
        self.transitions.push(transition);
    }
}

/// Состояние
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// Имя состояния
    pub name: String,
    /// Алиас
    pub alias: Option<String>,
    /// Описание
    pub description: Option<String>,
    /// Стереотип
    pub stereotype: Option<Stereotype>,
    /// Тип состояния
    pub state_type: StateType,
    /// Вложенные состояния
    pub substates: Vec<State>,
    /// Внутренние переходы
    pub internal_transitions: Vec<Transition>,
    /// Параллельные регионы
    pub regions: Vec<Vec<State>>,
    /// Цвет
    pub color: Option<Color>,
    /// Действие при входе
    pub entry_action: Option<String>,
    /// Действие при выходе
    pub exit_action: Option<String>,
    /// Действие во время нахождения
    pub do_action: Option<String>,
}

impl State {
    /// Создаёт новое состояние
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            alias: None,
            description: None,
            stereotype: None,
            state_type: StateType::Simple,
            substates: Vec::new(),
            internal_transitions: Vec::new(),
            regions: Vec::new(),
            color: None,
            entry_action: None,
            exit_action: None,
            do_action: None,
        }
    }

    /// Создаёт начальное состояние
    pub fn initial() -> Self {
        Self {
            state_type: StateType::Initial,
            ..Self::new("[*]")
        }
    }

    /// Создаёт конечное состояние
    pub fn final_state() -> Self {
        Self {
            state_type: StateType::Final,
            ..Self::new("[*]")
        }
    }

    /// Создаёт составное состояние
    pub fn composite(name: impl Into<String>) -> Self {
        Self {
            state_type: StateType::Composite,
            ..Self::new(name)
        }
    }

    /// Добавляет вложенное состояние
    pub fn add_substate(&mut self, state: State) {
        self.substates.push(state);
        if self.state_type == StateType::Simple {
            self.state_type = StateType::Composite;
        }
    }
}

/// Тип состояния
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StateType {
    #[default]
    /// Простое состояние
    Simple,
    /// Составное состояние (содержит вложенные)
    Composite,
    /// Начальное состояние `[*]`
    Initial,
    /// Конечное состояние `[*]`
    Final,
    /// История (H)
    History,
    /// Глубокая история (H*)
    DeepHistory,
    /// Точка выбора
    Choice,
    /// Развилка
    Fork,
    /// Слияние
    Join,
    /// Точка входа
    EntryPoint,
    /// Точка выхода
    ExitPoint,
}

/// Переход между состояниями
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    /// Исходное состояние
    pub from: String,
    /// Целевое состояние
    pub to: String,
    /// Событие (триггер)
    pub event: Option<String>,
    /// Условие (guard)
    pub guard: Option<String>,
    /// Действие
    pub action: Option<String>,
    /// Цвет
    pub color: Option<Color>,
}

impl Transition {
    /// Создаёт новый переход
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            event: None,
            guard: None,
            action: None,
            color: None,
        }
    }

    /// Устанавливает событие
    pub fn with_event(mut self, event: impl Into<String>) -> Self {
        self.event = Some(event.into());
        self
    }

    /// Устанавливает условие
    pub fn with_guard(mut self, guard: impl Into<String>) -> Self {
        self.guard = Some(guard.into());
        self
    }

    /// Устанавливает действие
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Возвращает полную метку перехода
    pub fn label(&self) -> String {
        let mut parts = Vec::new();
        if let Some(event) = &self.event {
            parts.push(event.clone());
        }
        if let Some(guard) = &self.guard {
            parts.push(format!("[{}]", guard));
        }
        if let Some(action) = &self.action {
            parts.push(format!("/ {}", action));
        }
        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_state() {
        let state = State::new("Active");
        assert_eq!(state.name, "Active");
        assert_eq!(state.state_type, StateType::Simple);
    }

    #[test]
    fn test_composite_state() {
        let mut composite = State::composite("Processing");
        composite.add_substate(State::new("Validating"));
        composite.add_substate(State::new("Executing"));

        assert_eq!(composite.state_type, StateType::Composite);
        assert_eq!(composite.substates.len(), 2);
    }

    #[test]
    fn test_transition_label() {
        let trans = Transition::new("A", "B")
            .with_event("click")
            .with_guard("valid")
            .with_action("process");

        assert_eq!(trans.label(), "click [valid] / process");
    }
}
