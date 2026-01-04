//! State Diagram Layout Engine
//!
//! Алгоритм layout для диаграмм состояний.
//! Использует послойное расположение с учётом переходов.

use indexmap::{IndexMap, IndexSet};
use plantuml_ast::state::{StateDiagram, StateType, Transition};
use plantuml_model::{Point, Rect};

use super::config::StateLayoutConfig;
use crate::{EdgeType, ElementType, LayoutElement, LayoutResult};

/// Layout engine для state diagrams
pub struct StateLayoutEngine {
    config: StateLayoutConfig,
}

/// Внутренние идентификаторы для [*]
const INITIAL_STATE_ID: &str = "[*]_initial";
const FINAL_STATE_ID: &str = "[*]_final";

impl StateLayoutEngine {
    /// Создаёт новый engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: StateLayoutConfig::default(),
        }
    }

    /// Создаёт engine с заданной конфигурацией
    pub fn with_config(config: StateLayoutConfig) -> Self {
        Self { config }
    }

    /// Выполняет layout диаграммы
    pub fn layout(&self, diagram: &StateDiagram) -> LayoutResult {
        let mut elements = Vec::new();
        let mut state_positions: IndexMap<String, Rect> = IndexMap::new();

        // Анализируем использование [*] - может быть initial, final или оба
        let has_initial = diagram.transitions.iter().any(|t| t.from == "[*]");
        let has_final = diagram.transitions.iter().any(|t| t.to == "[*]");

        // Собираем все состояния с разделением [*] на initial и final
        let mut all_states: IndexSet<String> = IndexSet::new();
        
        // Добавляем начальное состояние если есть
        if has_initial {
            all_states.insert(INITIAL_STATE_ID.to_string());
        }
        
        // Добавляем обычные состояния
        for state in &diagram.states {
            if state.name != "[*]" {
                all_states.insert(state.name.clone());
            }
            if let Some(alias) = &state.alias {
                if alias != "[*]" {
                    all_states.insert(alias.clone());
                }
            }
        }
        
        // Добавляем состояния из переходов (кроме [*])
        for trans in &diagram.transitions {
            if trans.from != "[*]" {
                all_states.insert(trans.from.clone());
            }
            if trans.to != "[*]" {
                all_states.insert(trans.to.clone());
            }
        }
        
        // Добавляем конечное состояние если есть
        if has_final {
            all_states.insert(FINAL_STATE_ID.to_string());
        }

        // Преобразуем переходы для внутренней обработки
        let internal_transitions: Vec<(String, String, Option<String>)> = diagram
            .transitions
            .iter()
            .map(|t| {
                let from = if t.from == "[*]" {
                    INITIAL_STATE_ID.to_string()
                } else {
                    t.from.clone()
                };
                let to = if t.to == "[*]" {
                    FINAL_STATE_ID.to_string()
                } else {
                    t.to.clone()
                };
                let label = t.label();
                (from, to, if label.is_empty() { None } else { Some(label) })
            })
            .collect();

        // Определяем уровни состояний
        let levels = self.assign_levels(&all_states, &internal_transitions, has_initial, has_final);
        
        // Группируем состояния по уровням (используем IndexMap для стабильного порядка)
        let mut level_states: IndexMap<usize, Vec<String>> = IndexMap::new();
        for (state, level) in &levels {
            level_states
                .entry(*level)
                .or_insert_with(Vec::new)
                .push(state.clone());
        }

        // Располагаем состояния по уровням
        let max_level = levels.values().max().copied().unwrap_or(0);
        
        for level in 0..=max_level {
            if let Some(states) = level_states.get(&level) {
                let y = self.config.margin 
                    + level as f64 * (self.config.state_min_height + self.config.vertical_spacing);
                
                let total_width = states.len() as f64 * self.config.state_width 
                    + (states.len() as f64 - 1.0) * self.config.horizontal_spacing;
                let start_x = self.config.margin + (500.0 - total_width) / 2.0; // Центрируем

                for (i, state_name) in states.iter().enumerate() {
                    let x = start_x + i as f64 * (self.config.state_width + self.config.horizontal_spacing);
                    
                    // Определяем тип состояния
                    let state_type = self.get_state_type_internal(diagram, state_name);
                    
                    let (elem, bounds) = self.create_state_element(state_name, state_type, x, y);
                    state_positions.insert(state_name.clone(), bounds);
                    elements.push(elem);
                }
            }
        }

        // Создаём переходы
        for (from, to, label) in &internal_transitions {
            if let (Some(from_rect), Some(to_rect)) = 
                (state_positions.get(from), state_positions.get(to)) 
            {
                let edge = self.create_transition_element_internal(from, to, label.as_deref(), from_rect, to_rect);
                elements.push(edge);
            }
        }

        // Вычисляем bounds
        let mut result = LayoutResult {
            elements,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        };
        result.calculate_bounds();
        
        // Добавляем отступы
        result.bounds.width += self.config.margin * 2.0;
        result.bounds.height += self.config.margin * 2.0;

        result
    }

    /// Назначает уровни состояниям (с сохранением порядка)
    fn assign_levels(
        &self,
        all_states: &IndexSet<String>,
        transitions: &[(String, String, Option<String>)],
        has_initial: bool,
        has_final: bool,
    ) -> IndexMap<String, usize> {
        let mut levels: IndexMap<String, usize> = IndexMap::new();
        
        // Начальное состояние всегда на уровне 0
        if has_initial {
            levels.insert(INITIAL_STATE_ID.to_string(), 0);
        } else {
            // Если нет начального [*], находим состояния без входящих переходов
            let targets: IndexSet<&String> = transitions.iter().map(|(_, to, _)| to).collect();
            for state in all_states {
                if state != FINAL_STATE_ID && !targets.contains(state) {
                    levels.insert(state.clone(), 0);
                }
            }
        }

        // Если всё ещё пусто, берём первое состояние (кроме final)
        if levels.is_empty() {
            if let Some(first) = all_states.iter().find(|s| *s != FINAL_STATE_ID) {
                levels.insert(first.clone(), 0);
            }
        }

        // BFS для назначения уровней (кроме final state)
        let max_iterations = all_states.len() + 1;
        for _ in 0..max_iterations {
            let mut new_levels = levels.clone();
            
            for (from, to, _) in transitions {
                // Пропускаем переходы к final state - он будет обработан позже
                if to == FINAL_STATE_ID {
                    continue;
                }
                
                if let Some(&from_level) = levels.get(from) {
                    let new_level = from_level + 1;
                    
                    // Назначаем уровень только если ещё не назначен или новый уровень больше
                    let current = new_levels.get(to).copied();
                    if current.is_none() {
                        new_levels.insert(to.clone(), new_level);
                    }
                }
            }
            
            if new_levels.len() == levels.len() {
                break;
            }
            levels = new_levels;
        }

        // Конечное состояние всегда на последнем уровне
        if has_final {
            let max_level = levels.values().max().copied().unwrap_or(0);
            levels.insert(FINAL_STATE_ID.to_string(), max_level + 1);
        }

        // Устанавливаем уровень 0 для оставшихся состояний (не должно происходить)
        for state in all_states {
            if !levels.contains_key(state) {
                levels.insert(state.clone(), 0);
            }
        }

        levels
    }
    
    /// Получает тип состояния для внутреннего идентификатора
    fn get_state_type_internal(&self, diagram: &StateDiagram, name: &str) -> StateType {
        if name == INITIAL_STATE_ID {
            return StateType::Initial;
        }
        if name == FINAL_STATE_ID {
            return StateType::Final;
        }
        
        // Делегируем к исходному методу
        self.get_state_type(diagram, name)
    }

    /// Получает тип состояния
    fn get_state_type(&self, diagram: &StateDiagram, name: &str) -> StateType {
        if name == "[*]" {
            // Определяем: это начальное или конечное состояние
            let is_source = diagram.transitions.iter().any(|t| t.from == name);
            let is_target = diagram.transitions.iter().any(|t| t.to == name);
            
            if is_source && !is_target {
                return StateType::Initial;
            } else if is_target && !is_source {
                return StateType::Final;
            }
            return StateType::Initial; // По умолчанию
        }

        if name == "[H]" {
            return StateType::History;
        }
        if name == "[H*]" {
            return StateType::DeepHistory;
        }

        // Ищем в определениях состояний
        for state in &diagram.states {
            if state.name == name || state.alias.as_deref() == Some(name) {
                return state.state_type;
            }
        }

        StateType::Simple
    }

    /// Создаёт элемент состояния
    fn create_state_element(
        &self,
        name: &str,
        state_type: StateType,
        x: f64,
        y: f64,
    ) -> (LayoutElement, Rect) {
        match state_type {
            StateType::Initial => self.create_initial_state(name, x, y),
            StateType::Final => self.create_final_state(name, x, y),
            StateType::Choice => self.create_choice_state(name, x, y),
            StateType::Fork | StateType::Join => self.create_fork_join_state(name, x, y),
            StateType::History => self.create_history_state(name, x, y, false),
            StateType::DeepHistory => self.create_history_state(name, x, y, true),
            StateType::Composite => self.create_composite_state(name, x, y),
            _ => self.create_simple_state(name, x, y),
        }
    }

    /// Создаёт начальное состояние (UML: заполненный чёрный круг)
    fn create_initial_state(&self, name: &str, x: f64, y: f64) -> (LayoutElement, Rect) {
        let r = self.config.node_radius;
        let cx = x + self.config.state_width / 2.0;
        let cy = y + r;
        
        let bounds = Rect::new(cx - r, cy - r, r * 2.0, r * 2.0);
        
        (LayoutElement {
            id: format!("initial_{}", name.replace(['[', ']', '*', '_'], "")),
            bounds: bounds.clone(),
            text: None, 
            properties: std::collections::HashMap::new(), 
            element_type: ElementType::InitialState,
        }, bounds)
    }

    /// Создаёт конечное состояние (UML: bullseye - круг в круге)
    fn create_final_state(&self, name: &str, x: f64, y: f64) -> (LayoutElement, Rect) {
        let r = self.config.node_radius;
        let cx = x + self.config.state_width / 2.0;
        let cy = y + r;
        
        let bounds = Rect::new(cx - r, cy - r, r * 2.0, r * 2.0);
        
        (LayoutElement {
            id: format!("final_{}", name.replace(['[', ']', '*', '_'], "")),
            bounds: bounds.clone(),
            text: None, 
            properties: std::collections::HashMap::new(), 
            element_type: ElementType::FinalState,
        }, bounds)
    }

    /// Создаёт простое состояние (UML: скруглённый прямоугольник с разделителем)
    fn create_simple_state(&self, name: &str, x: f64, y: f64) -> (LayoutElement, Rect) {
        let bounds = Rect::new(x, y, self.config.state_width, self.config.state_min_height);
        
        (LayoutElement {
            id: format!("state_{}", name),
            bounds: bounds.clone(),
            text: None, 
            properties: std::collections::HashMap::new(), 
            element_type: ElementType::State {
                name: name.to_string(),
                description: None,
            },
        }, bounds)
    }

    /// Создаёт составное состояние
    fn create_composite_state(&self, name: &str, x: f64, y: f64) -> (LayoutElement, Rect) {
        // Составное состояние выше обычного
        let height = self.config.state_min_height * 2.0;
        let bounds = Rect::new(x, y, self.config.state_width * 1.5, height);
        
        (LayoutElement {
            id: format!("composite_{}", name),
            bounds: bounds.clone(),
            text: None, 
            properties: std::collections::HashMap::new(), 
            element_type: ElementType::State {
                name: name.to_string(),
                description: None,
            },
        }, bounds)
    }

    /// Создаёт choice state (ромб)
    fn create_choice_state(&self, name: &str, x: f64, y: f64) -> (LayoutElement, Rect) {
        let size = self.config.choice_size;
        let cx = x + self.config.state_width / 2.0;
        let cy = y + size / 2.0;
        
        let bounds = Rect::new(cx - size / 2.0, cy - size / 2.0, size, size);
        
        (LayoutElement {
            id: format!("choice_{}", name),
            bounds: bounds.clone(),
            text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Text {
                text: "◇".to_string(), // Ромб
                font_size: 16.0,
            },
        }, bounds)
    }

    /// Создаёт fork/join bar
    fn create_fork_join_state(&self, name: &str, x: f64, y: f64) -> (LayoutElement, Rect) {
        let cx = x + self.config.state_width / 2.0;
        let bounds = Rect::new(
            cx - self.config.bar_width / 2.0,
            y,
            self.config.bar_width,
            self.config.bar_height,
        );
        
        (LayoutElement {
            id: format!("bar_{}", name),
            bounds: bounds.clone(),
            text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Rectangle {
                label: String::new(),
                corner_radius: 0.0,
            },
        }, bounds)
    }

    /// Создаёт history state
    fn create_history_state(&self, name: &str, x: f64, y: f64, deep: bool) -> (LayoutElement, Rect) {
        let r = self.config.node_radius * 0.8;
        let cx = x + self.config.state_width / 2.0;
        let cy = y + r;
        
        let bounds = Rect::new(cx - r, cy - r, r * 2.0, r * 2.0);
        
        let label = if deep { "H*" } else { "H" };
        
        (LayoutElement {
            id: format!("history_{}", name.replace(['[', ']', '*'], "")),
            bounds: bounds.clone(),
            text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Ellipse { 
                label: Some(label.to_string())
            },
        }, bounds)
    }

    /// Создаёт элемент перехода (устаревший метод, оставлен для совместимости)
    #[allow(dead_code)]
    fn create_transition_element(
        &self,
        trans: &Transition,
        from_rect: &Rect,
        to_rect: &Rect,
    ) -> LayoutElement {
        let label = trans.label();
        let label_opt = if label.is_empty() { None } else { Some(label) };
        self.create_transition_element_internal(&trans.from, &trans.to, label_opt.as_deref(), from_rect, to_rect)
    }
    
    /// Создаёт элемент перехода для внутренних идентификаторов
    fn create_transition_element_internal(
        &self,
        from: &str,
        to: &str,
        label: Option<&str>,
        from_rect: &Rect,
        to_rect: &Rect,
    ) -> LayoutElement {
        let from_center_y = from_rect.y + from_rect.height / 2.0;
        let to_center_y = to_rect.y + to_rect.height / 2.0;

        let dy = to_center_y - from_center_y;
        
        // Определяем, нужен ли ортогональный путь с обходом
        // Обратный переход (вверх) на несколько уровней требует обхода справа
        let is_backward_transition = dy < -self.config.vertical_spacing;
        
        let points = if is_backward_transition {
            // Ортогональный путь с обходом СПРАВА (как в PlantUML)
            // Выход справа от source, вверх, влево, в правый край target
            let offset = 40.0; // отступ для обхода
            
            // Находим правую границу обоих элементов
            let right_x = from_rect.x.max(to_rect.x) + from_rect.width.max(to_rect.width) + offset;
            
            let start = Point::new(from_rect.x + from_rect.width, from_center_y);
            let corner1 = Point::new(right_x, from_center_y);
            let corner2 = Point::new(right_x, to_center_y);
            let end = Point::new(to_rect.x + to_rect.width, to_center_y);
            
            vec![start, corner1, corner2, end]
        } else {
            // Обычное прямое соединение
            let (start, end) = self.calculate_connection_points(from_rect, to_rect);
            vec![start, end]
        };

        // Вычисляем bounds для всех точек
        let min_x = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
        let min_y = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
        let max_x = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
        let max_y = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
        
        // Очищаем имена от специальных символов для id
        let from_clean = from.replace(['[', ']', '*', '_'], "");
        let to_clean = to.replace(['[', ']', '*', '_'], "");

        LayoutElement {
            id: format!("trans_{}_{}", from_clean, to_clean),
            bounds: Rect::new(min_x, min_y, (max_x - min_x).max(1.0), (max_y - min_y).max(1.0)),
            text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Edge {
                points,
                label: label.map(|s| s.to_string()),
                arrow_start: false,
                arrow_end: true,
                dashed: false,
                edge_type: EdgeType::Association, from_cardinality: None, to_cardinality: None,
            },
        }
    }

    /// Вычисляет точки соединения для прямого перехода
    /// PlantUML style: вертикальные переходы выходят снизу/сверху
    fn calculate_connection_points(&self, from: &Rect, to: &Rect) -> (Point, Point) {
        let from_center_x = from.x + from.width / 2.0;
        let to_center_x = to.x + to.width / 2.0;

        // Определяем направление
        let dx = to_center_x - from_center_x;
        let dy = (to.y + to.height / 2.0) - (from.y + from.height / 2.0);

        // Переход ВНИЗ
        if dy > 0.0 {
            // Проверяем, это маленький элемент (final state)?
            let is_to_small = to.width < 30.0 && to.height < 30.0;
            
            if is_to_small {
                // Переход к Final state: выходим снизу, входим СВЕРХУ по центру
                // Диагональная линия от низа source к верху target
                let from_x = if dx.abs() < 10.0 {
                    from_center_x
                } else if dx > 0.0 {
                    from_center_x + from.width * 0.2
                } else {
                    from_center_x - from.width * 0.2
                };
                
                let start = Point::new(from_x, from.y + from.height);
                let end = Point::new(to_center_x, to.y); // Вход сверху в центр
                (start, end)
            } else {
                // Обычный переход между состояниями
                // Смещаем точки по X для визуального разделения
                let from_x = if dx.abs() < 10.0 {
                    from_center_x
                } else if dx > 0.0 {
                    from_center_x + from.width * 0.2
                } else {
                    from_center_x - from.width * 0.2
                };
                
                let to_x = if dx.abs() < 10.0 {
                    to_center_x
                } else if dx > 0.0 {
                    to_center_x - to.width * 0.2
                } else {
                    to_center_x + to.width * 0.2
                };
                
                let start = Point::new(from_x, from.y + from.height);
                let end = Point::new(to_x, to.y);
                (start, end)
            }
        } else {
            // Переход ВВЕРХ (небольшой)
            let start = Point::new(from_center_x, from.y);
            let end = Point::new(to_center_x, to.y + to.height);
            (start, end)
        }
    }
}

impl Default for StateLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_ast::state::State;

    #[test]
    fn test_layout_simple_state_machine() {
        let mut diagram = StateDiagram::new();
        diagram.add_transition(Transition::new("[*]", "Active"));
        diagram.add_transition(Transition::new("Active", "Inactive").with_event("timeout"));
        diagram.add_transition(Transition::new("Inactive", "[*]"));

        let engine = StateLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Должны быть: initial, Active, Inactive, final + 3 перехода
        assert!(result.elements.len() >= 6);
    }

    #[test]
    fn test_layout_with_choice() {
        let mut diagram = StateDiagram::new();
        diagram.add_state(State {
            name: "choice1".to_string(),
            alias: None,
            description: None,
            stereotype: None,
            state_type: StateType::Choice,
            substates: Vec::new(),
            internal_transitions: Vec::new(),
            regions: Vec::new(),
            color: None,
            entry_action: None,
            exit_action: None,
            do_action: None,
        });
        
        diagram.add_transition(Transition::new("[*]", "choice1"));
        diagram.add_transition(Transition::new("choice1", "State1"));
        diagram.add_transition(Transition::new("choice1", "State2"));

        let engine = StateLayoutEngine::new();
        let result = engine.layout(&diagram);

        assert!(!result.elements.is_empty());
    }

    #[test]
    fn test_layout_with_fork_join() {
        let mut diagram = StateDiagram::new();
        diagram.add_state(State {
            name: "fork1".to_string(),
            alias: None,
            description: None,
            stereotype: None,
            state_type: StateType::Fork,
            substates: Vec::new(),
            internal_transitions: Vec::new(),
            regions: Vec::new(),
            color: None,
            entry_action: None,
            exit_action: None,
            do_action: None,
        });
        
        diagram.add_transition(Transition::new("[*]", "fork1"));
        diagram.add_transition(Transition::new("fork1", "Task1"));
        diagram.add_transition(Transition::new("fork1", "Task2"));

        let engine = StateLayoutEngine::new();
        let result = engine.layout(&diagram);

        assert!(!result.elements.is_empty());
    }
}
