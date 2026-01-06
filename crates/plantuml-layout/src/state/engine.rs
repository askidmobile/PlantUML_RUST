//! State Diagram Layout Engine
//!
//! Алгоритм layout для диаграмм состояний.
//! Поддерживает вложенные (composite) состояния.

use indexmap::{IndexMap, IndexSet};
use plantuml_ast::state::{State, StateDiagram, StateType};
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

/// Результат layout подсостояний
struct SubLayoutResult {
    elements: Vec<LayoutElement>,
    bounds: Rect,
}

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

        // Определяем composite состояния и собираем их внутренние состояния
        let composite_states: IndexMap<String, &State> = diagram
            .states
            .iter()
            .filter(|s| s.state_type == StateType::Composite)
            .map(|s| (s.name.clone(), s))
            .collect();

        // Собираем ВСЕ внутренние состояния из всех composite
        let mut internal_states: IndexSet<String> = IndexSet::new();
        for cs in composite_states.values() {
            for trans in &cs.internal_transitions {
                if trans.from != "[*]" {
                    internal_states.insert(trans.from.clone());
                }
                if trans.to != "[*]" {
                    internal_states.insert(trans.to.clone());
                }
            }
            for sub in &cs.substates {
                internal_states.insert(sub.name.clone());
            }
        }

        // Анализируем использование [*] на верхнем уровне
        let has_initial = diagram.transitions.iter().any(|t| t.from == "[*]");
        let has_final = diagram.transitions.iter().any(|t| t.to == "[*]");

        // Собираем состояния ТОЛЬКО верхнего уровня
        let mut top_level_states: IndexSet<String> = IndexSet::new();
        
        if has_initial {
            top_level_states.insert(INITIAL_STATE_ID.to_string());
        }
        
        // Добавляем явно определённые состояния верхнего уровня (НЕ внутренние)
        for state in &diagram.states {
            if state.name != "[*]" && !internal_states.contains(&state.name) {
                top_level_states.insert(state.name.clone());
            }
        }
        
        // Добавляем состояния из переходов ВЕРХНЕГО УРОВНЯ (НЕ внутренние)
        for trans in &diagram.transitions {
            if trans.from != "[*]" && !internal_states.contains(&trans.from) {
                top_level_states.insert(trans.from.clone());
            }
            if trans.to != "[*]" && !internal_states.contains(&trans.to) {
                top_level_states.insert(trans.to.clone());
            }
        }
        
        if has_final {
            top_level_states.insert(FINAL_STATE_ID.to_string());
        }

        // Преобразуем переходы верхнего уровня
        let top_level_transitions: Vec<(String, String, Option<String>)> = diagram
            .transitions
            .iter()
            .filter(|t| {
                // Оставляем только переходы между состояниями верхнего уровня
                let from_ok = t.from == "[*]" || top_level_states.contains(&t.from);
                let to_ok = t.to == "[*]" || top_level_states.contains(&t.to);
                from_ok && to_ok
            })
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

        // Определяем уровни состояний верхнего уровня
        let levels = self.assign_levels(&top_level_states, &top_level_transitions, has_initial, has_final);
        
        // Группируем по уровням
        let mut level_states: IndexMap<usize, Vec<String>> = IndexMap::new();
        for (state, level) in &levels {
            level_states
                .entry(*level)
                .or_default()
                .push(state.clone());
        }

        // Сначала делаем layout для composite состояний, чтобы узнать их размеры
        let mut composite_layouts: IndexMap<String, SubLayoutResult> = IndexMap::new();
        
        for (name, composite) in &composite_states {
            let sub_result = self.layout_composite_content(composite);
            composite_layouts.insert(name.clone(), sub_result);
        }

        // Располагаем состояния верхнего уровня
        // Используем динамический расчёт Y с учётом реальной высоты composite контейнеров
        let max_level = levels.values().max().copied().unwrap_or(0);
        
        // Сначала вычисляем размеры для каждого уровня
        let mut level_heights: IndexMap<usize, f64> = IndexMap::new();
        let mut level_widths: IndexMap<usize, f64> = IndexMap::new();
        
        for level in 0..=max_level {
            if let Some(states) = level_states.get(&level) {
                let max_height = states.iter().map(|name| {
                    if let Some(layout) = composite_layouts.get(name) {
                        layout.bounds.height + self.config.margin * 2.0 + 30.0 // header
                    } else if name == INITIAL_STATE_ID || name == FINAL_STATE_ID {
                        self.config.node_radius * 2.0
                    } else {
                        self.config.state_min_height
                    }
                }).fold(0.0f64, f64::max);
                level_heights.insert(level, max_height);
                
                // Вычисляем ширину для центрирования
                let total_width: f64 = states.iter().map(|name| {
                    if let Some(layout) = composite_layouts.get(name) {
                        layout.bounds.width + self.config.margin * 2.0
                    } else if name == INITIAL_STATE_ID || name == FINAL_STATE_ID {
                        self.config.node_radius * 2.0
                    } else {
                        self.config.state_width
                    }
                }).sum::<f64>() + (states.len().saturating_sub(1)) as f64 * self.config.horizontal_spacing;
                level_widths.insert(level, total_width);
            }
        }
        
        // Находим максимальную ширину среди всех уровней для центрирования
        let max_width = level_widths.values().copied().fold(0.0f64, f64::max);
        let diagram_center_x = self.config.margin + max_width / 2.0;
        
        // Вычисляем начальную Y позицию для каждого уровня на основе предыдущих
        let mut level_y_positions: IndexMap<usize, f64> = IndexMap::new();
        let mut current_y = self.config.margin;
        for level in 0..=max_level {
            level_y_positions.insert(level, current_y);
            let height = level_heights.get(&level).copied().unwrap_or(self.config.state_min_height);
            current_y += height + self.config.vertical_spacing;
        }
        
        for level in 0..=max_level {
            if let Some(states) = level_states.get(&level) {
                let level_width = level_widths.get(&level).copied().unwrap_or(0.0);
                
                // Центрируем относительно общего центра диаграммы
                let start_x = diagram_center_x - level_width / 2.0;
                let mut x = start_x;
                
                // Получаем Y позицию для данного уровня
                let y = level_y_positions.get(&level).copied().unwrap_or(self.config.margin);

                for state_name in states {
                    // Проверяем, это composite состояние?
                    if let Some(composite) = composite_states.get(state_name) {
                        let sub_layout = composite_layouts.get(state_name).unwrap();
                        
                        // Создаём контейнер composite состояния
                        let container_elements = self.create_composite_container(
                            composite,
                            x,
                            y,
                            sub_layout,
                        );
                        
                        // Сохраняем позицию контейнера
                        let container_rect = Rect::new(
                            x,
                            y,
                            sub_layout.bounds.width + self.config.margin * 2.0,
                            sub_layout.bounds.height + self.config.margin * 2.0 + 30.0,
                        );
                        state_positions.insert(state_name.clone(), container_rect.clone());
                        
                        // Добавляем все элементы
                        elements.extend(container_elements);
                        
                        x += container_rect.width + self.config.horizontal_spacing;
                    } else {
                        // Обычное состояние
                        let state_type = self.get_state_type_internal(diagram, state_name);
                        let (elem, bounds) = self.create_state_element(state_name, state_type, x, y);
                        state_positions.insert(state_name.clone(), bounds.clone());
                        elements.push(elem);
                        
                        x += bounds.width + self.config.horizontal_spacing;
                    }
                }
            }
        }

        // Создаём переходы верхнего уровня
        for (from, to, label) in &top_level_transitions {
            if let (Some(from_rect), Some(to_rect)) = 
                (state_positions.get(from), state_positions.get(to)) 
            {
                let edge = self.create_transition_element(from, to, label.as_deref(), from_rect, to_rect);
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

    /// Выполняет layout содержимого composite состояния
    fn layout_composite_content(&self, composite: &State) -> SubLayoutResult {
        let mut elements = Vec::new();
        let mut state_positions: IndexMap<String, Rect> = IndexMap::new();

        // Анализируем internal_transitions
        let has_initial = composite.internal_transitions.iter().any(|t| t.from == "[*]");
        let has_final = composite.internal_transitions.iter().any(|t| t.to == "[*]");

        // Собираем все внутренние состояния
        let mut inner_states: IndexSet<String> = IndexSet::new();
        
        if has_initial {
            inner_states.insert(INITIAL_STATE_ID.to_string());
        }
        
        for state in &composite.substates {
            if state.name != "[*]" {
                inner_states.insert(state.name.clone());
            }
        }
        
        for trans in &composite.internal_transitions {
            if trans.from != "[*]" {
                inner_states.insert(trans.from.clone());
            }
            if trans.to != "[*]" {
                inner_states.insert(trans.to.clone());
            }
        }
        
        if has_final {
            inner_states.insert(FINAL_STATE_ID.to_string());
        }

        // Преобразуем переходы
        let internal_transitions: Vec<(String, String, Option<String>)> = composite
            .internal_transitions
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

        // Назначаем уровни
        let levels = self.assign_levels(&inner_states, &internal_transitions, has_initial, has_final);
        
        // Группируем по уровням
        let mut level_states: IndexMap<usize, Vec<String>> = IndexMap::new();
        for (state, level) in &levels {
            level_states
                .entry(*level)
                .or_default()
                .push(state.clone());
        }

        // Располагаем внутренние состояния
        let max_level = levels.values().max().copied().unwrap_or(0);
        let inner_margin = 15.0;
        let inner_state_width = 90.0;
        let inner_state_height = 35.0;
        let inner_spacing_v = 40.0;
        let inner_spacing_h = 30.0;
        
        // Считаем количество обратных переходов для вычисления необходимого пространства справа
        let backward_count = internal_transitions.iter()
            .filter(|(from, to, _)| {
                let from_level = levels.get(from).copied().unwrap_or(0);
                let to_level = levels.get(to).copied().unwrap_or(0);
                to_level < from_level // переход на уровень выше = обратный
            })
            .count();
        
        // Пространство справа для обратных стрелок
        let backward_space = if backward_count > 0 {
            20.0 + backward_count as f64 * 25.0
        } else {
            0.0
        };
        
        // Вычисляем максимальную ширину уровня (для центрирования)
        let mut max_level_width = 0.0f64;
        for level in 0..=max_level {
            if let Some(states) = level_states.get(&level) {
                let level_width = states.len() as f64 * inner_state_width 
                    + (states.len().saturating_sub(1)) as f64 * inner_spacing_h;
                max_level_width = max_level_width.max(level_width);
            }
        }
        
        // Общая ширина контента: элементы + пространство для обратных стрелок
        let content_width = max_level_width + backward_space;
        
        let mut max_x = 0.0f64;
        let mut max_y = 0.0f64;
        
        for level in 0..=max_level {
            if let Some(states) = level_states.get(&level) {
                let level_width = states.len() as f64 * inner_state_width 
                    + (states.len().saturating_sub(1)) as f64 * inner_spacing_h;
                
                // Центрируем элементы относительно общей ширины контента (без backward_space)
                // Это сместит элементы немного влево, оставляя место справа для стрелок
                let start_x = inner_margin + (max_level_width - level_width) / 2.0;

                for (i, state_name) in states.iter().enumerate() {
                    let x = start_x + i as f64 * (inner_state_width + inner_spacing_h);
                    let y = inner_margin + level as f64 * (inner_state_height + inner_spacing_v);
                    
                    let state_type = if state_name == INITIAL_STATE_ID {
                        StateType::Initial
                    } else if state_name == FINAL_STATE_ID {
                        StateType::Final
                    } else {
                        composite.substates.iter()
                            .find(|s| s.name == *state_name)
                            .map(|s| s.state_type)
                            .unwrap_or(StateType::Simple)
                    };
                    
                    let (elem, bounds) = self.create_inner_state_element(
                        state_name, state_type, x, y, inner_state_width, inner_state_height
                    );
                    state_positions.insert(state_name.clone(), bounds.clone());
                    elements.push(elem);
                    
                    max_x = max_x.max(bounds.x + bounds.width);
                    max_y = max_y.max(bounds.y + bounds.height);
                }
            }
        }
        
        // Обновляем max_x с учётом пространства для обратных стрелок
        max_x += backward_space;

        // Создаём внутренние переходы
        // Считаем обратные переходы для уникального offset
        let mut backward_transition_index = 0;
        for (from, to, label) in &internal_transitions {
            if let (Some(from_rect), Some(to_rect)) = 
                (state_positions.get(from), state_positions.get(to)) 
            {
                let dy = (to_rect.y + to_rect.height / 2.0) - (from_rect.y + from_rect.height / 2.0);
                let is_backward = dy < -20.0;
                
                let edge = self.create_inner_transition_indexed(
                    from, to, label.as_deref(), from_rect, to_rect,
                    if is_backward { backward_transition_index } else { 0 }
                );
                elements.push(edge);
                
                if is_backward {
                    backward_transition_index += 1;
                }
            }
        }

        // Общая ширина контента для возврата
        let total_content_width = max_x + inner_margin;
        
        // Центрируем внутренние элементы относительно общей ширины
        // Находим текущий центр элементов
        let elements_center_x = inner_margin + max_level_width / 2.0;
        // Целевой центр (середина общей ширины)
        let target_center_x = total_content_width / 2.0;
        // Смещение для центрирования
        let center_offset = target_center_x - elements_center_x;
        
        // Смещаем все элементы
        for elem in &mut elements {
            elem.bounds.x += center_offset;
            
            // Смещаем точки в Edge
            if let ElementType::Edge { ref mut points, .. } = elem.element_type {
                for point in points.iter_mut() {
                    point.x += center_offset;
                }
            }
        }
        
        // Обновляем state_positions для корректных переходов (уже созданы, не нужно)
        
        SubLayoutResult {
            elements,
            bounds: Rect::new(0.0, 0.0, total_content_width, max_y + inner_margin),
        }
    }

    /// Создаёт контейнер composite состояния со всем содержимым
    fn create_composite_container(
        &self,
        composite: &State,
        x: f64,
        y: f64,
        sub_layout: &SubLayoutResult,
    ) -> Vec<LayoutElement> {
        let mut elements = Vec::new();
        
        let header_height = 30.0;
        let padding = self.config.margin;
        
        let container_width = sub_layout.bounds.width + padding * 2.0;
        let container_height = sub_layout.bounds.height + padding * 2.0 + header_height;
        
        // Создаём внешний контейнер
        let container_bounds = Rect::new(x, y, container_width, container_height);
        
        elements.push(LayoutElement {
            id: format!("composite_{}", composite.name),
            bounds: container_bounds.clone(),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::CompositeState {
                name: composite.name.clone(),
                header_height,
            },
        });
        
        // Смещаем все внутренние элементы
        let offset_x = x + padding;
        let offset_y = y + header_height + padding;
        
        for elem in &sub_layout.elements {
            let mut shifted_elem = elem.clone();
            shifted_elem.bounds.x += offset_x;
            shifted_elem.bounds.y += offset_y;
            
            // Обновляем id чтобы был уникальным
            shifted_elem.id = format!("{}_{}", composite.name, shifted_elem.id);
            
            // Смещаем точки в Edge
            if let ElementType::Edge { ref mut points, .. } = shifted_elem.element_type {
                for point in points.iter_mut() {
                    point.x += offset_x;
                    point.y += offset_y;
                }
            }
            
            elements.push(shifted_elem);
        }
        
        elements
    }

    /// Создаёт элемент внутреннего состояния (меньший размер)
    fn create_inner_state_element(
        &self,
        name: &str,
        state_type: StateType,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> (LayoutElement, Rect) {
        match state_type {
            StateType::Initial => {
                let r = 8.0;
                let cx = x + width / 2.0;
                let cy = y + r;
                let bounds = Rect::new(cx - r, cy - r, r * 2.0, r * 2.0);
                
                (LayoutElement {
                    id: format!("inner_initial_{}", name.replace(['[', ']', '*', '_'], "")),
                    bounds: bounds.clone(),
                    text: None,
                    properties: std::collections::HashMap::new(),
                    element_type: ElementType::InitialState,
                }, bounds)
            }
            StateType::Final => {
                let r = 8.0;
                let cx = x + width / 2.0;
                let cy = y + r;
                let bounds = Rect::new(cx - r, cy - r, r * 2.0, r * 2.0);
                
                (LayoutElement {
                    id: format!("inner_final_{}", name.replace(['[', ']', '*', '_'], "")),
                    bounds: bounds.clone(),
                    text: None,
                    properties: std::collections::HashMap::new(),
                    element_type: ElementType::FinalState,
                }, bounds)
            }
            _ => {
                let bounds = Rect::new(x, y, width, height);
                
                (LayoutElement {
                    id: format!("inner_state_{}", name),
                    bounds: bounds.clone(),
                    text: None,
                    properties: std::collections::HashMap::new(),
                    element_type: ElementType::State {
                        name: name.to_string(),
                        description: None,
                    },
                }, bounds)
            }
        }
    }

    /// Создаёт внутренний переход с индексом для уникального offset
    fn create_inner_transition_indexed(
        &self,
        from: &str,
        to: &str,
        label: Option<&str>,
        from_rect: &Rect,
        to_rect: &Rect,
        backward_index: usize,
    ) -> LayoutElement {
        let from_center_x = from_rect.x + from_rect.width / 2.0;
        let to_center_x = to_rect.x + to_rect.width / 2.0;
        let from_center_y = from_rect.y + from_rect.height / 2.0;
        let to_center_y = to_rect.y + to_rect.height / 2.0;

        let dy = to_center_y - from_center_y;
        let dx = to_center_x - from_center_x;
        
        // Обратный переход (вверх)?
        let is_backward = dy < -20.0;
        
        let points = if is_backward {
            // Обход справа с уникальным offset для каждого обратного перехода
            // Стрелка выходит СПРАВА от исходного элемента, входит СПРАВА в целевой
            // Но с вертикальным смещением чтобы не накладываться на другие стрелки
            let base_offset = 15.0;
            let offset = base_offset + backward_index as f64 * 20.0;
            let right_x = from_rect.x.max(to_rect.x) + from_rect.width.max(to_rect.width) + offset;
            
            // Смещаем точки выхода и входа по вертикали, чтобы они не накладывались
            // Выход: верхняя часть элемента (для обратного перехода)
            // Вход: нижняя часть элемента
            let from_y = from_rect.y + from_rect.height * 0.3; // верхняя треть
            let to_y = to_rect.y + to_rect.height * 0.7; // нижняя треть
            
            vec![
                Point::new(from_rect.x + from_rect.width, from_y),
                Point::new(right_x, from_y),
                Point::new(right_x, to_y),
                Point::new(to_rect.x + to_rect.width, to_y),
            ]
        } else if dy > 10.0 {
            // Переход вниз - прямой переход
            // Выход снизу по центру, вход сверху по центру
            vec![
                Point::new(from_center_x, from_rect.y + from_rect.height),
                Point::new(to_center_x, to_rect.y),
            ]
        } else {
            // Горизонтальный или небольшой переход
            if dx > 0.0 {
                vec![
                    Point::new(from_rect.x + from_rect.width, from_center_y),
                    Point::new(to_rect.x, to_center_y),
                ]
            } else {
                vec![
                    Point::new(from_rect.x, from_center_y),
                    Point::new(to_rect.x + to_rect.width, to_center_y),
                ]
            }
        };

        let min_x = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
        let min_y = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
        let max_x = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
        let max_y = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
        
        let from_clean = from.replace(['[', ']', '*', '_'], "");
        let to_clean = to.replace(['[', ']', '*', '_'], "");

        LayoutElement {
            id: format!("inner_trans_{}_{}", from_clean, to_clean),
            bounds: Rect::new(min_x, min_y, (max_x - min_x).max(1.0), (max_y - min_y).max(1.0)),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Edge {
                points,
                label: label.map(|s| s.to_string()),
                arrow_start: false,
                arrow_end: true,
                dashed: false,
                edge_type: EdgeType::Association,
                from_cardinality: None,
                to_cardinality: None,
            },
        }
    }

    /// Назначает уровни состояниям
    fn assign_levels(
        &self,
        all_states: &IndexSet<String>,
        transitions: &[(String, String, Option<String>)],
        has_initial: bool,
        has_final: bool,
    ) -> IndexMap<String, usize> {
        let mut levels: IndexMap<String, usize> = IndexMap::new();
        
        if has_initial {
            levels.insert(INITIAL_STATE_ID.to_string(), 0);
        } else {
            let targets: IndexSet<&String> = transitions.iter().map(|(_, to, _)| to).collect();
            for state in all_states {
                if state != FINAL_STATE_ID && !targets.contains(state) {
                    levels.insert(state.clone(), 0);
                }
            }
        }

        if levels.is_empty() {
            if let Some(first) = all_states.iter().find(|s| *s != FINAL_STATE_ID) {
                levels.insert(first.clone(), 0);
            }
        }

        let max_iterations = all_states.len() + 1;
        for _ in 0..max_iterations {
            let mut new_levels = levels.clone();
            
            for (from, to, _) in transitions {
                if to == FINAL_STATE_ID {
                    continue;
                }
                
                if let Some(&from_level) = levels.get(from) {
                    let new_level = from_level + 1;
                    
                    if new_levels.get(to).is_none() {
                        new_levels.insert(to.clone(), new_level);
                    }
                }
            }
            
            if new_levels.len() == levels.len() {
                break;
            }
            levels = new_levels;
        }

        if has_final {
            let max_level = levels.values().max().copied().unwrap_or(0);
            levels.insert(FINAL_STATE_ID.to_string(), max_level + 1);
        }

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
        
        self.get_state_type(diagram, name)
    }

    /// Получает тип состояния
    fn get_state_type(&self, diagram: &StateDiagram, name: &str) -> StateType {
        if name == "[*]" {
            let is_source = diagram.transitions.iter().any(|t| t.from == name);
            let is_target = diagram.transitions.iter().any(|t| t.to == name);
            
            if is_source && !is_target {
                return StateType::Initial;
            } else if is_target && !is_source {
                return StateType::Final;
            }
            return StateType::Initial;
        }

        if name == "[H]" {
            return StateType::History;
        }
        if name == "[H*]" {
            return StateType::DeepHistory;
        }

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
            StateType::Composite => self.create_simple_state(name, x, y), // Handled separately
            _ => self.create_simple_state(name, x, y),
        }
    }

    /// Создаёт начальное состояние
    fn create_initial_state(&self, name: &str, x: f64, y: f64) -> (LayoutElement, Rect) {
        let r = self.config.node_radius;
        // x уже указывает на левый край области для элемента
        // Центр должен быть в середине выделенной ширины (node_radius * 2)
        let cx = x + r;
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

    /// Создаёт конечное состояние
    fn create_final_state(&self, name: &str, x: f64, y: f64) -> (LayoutElement, Rect) {
        let r = self.config.node_radius;
        // x уже указывает на левый край области для элемента
        let cx = x + r;
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

    /// Создаёт простое состояние
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

    /// Создаёт choice state (ромб)
    fn create_choice_state(&self, name: &str, x: f64, y: f64) -> (LayoutElement, Rect) {
        let size = self.config.choice_size;
        let cx = x + self.config.state_width / 2.0;
        let cy = y + size / 2.0;
        
        let bounds = Rect::new(cx - size / 2.0, cy - size / 2.0, size, size);
        
        (LayoutElement {
            id: format!("choice_{}", name),
            bounds: bounds.clone(),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Text {
                text: "◇".to_string(),
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
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Rectangle {
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
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Ellipse { 
                label: Some(label.to_string())
            },
        }, bounds)
    }

    /// Создаёт элемент перехода
    fn create_transition_element(
        &self,
        from: &str,
        to: &str,
        label: Option<&str>,
        from_rect: &Rect,
        to_rect: &Rect,
    ) -> LayoutElement {
        let from_center_x = from_rect.x + from_rect.width / 2.0;
        let to_center_x = to_rect.x + to_rect.width / 2.0;
        let from_center_y = from_rect.y + from_rect.height / 2.0;
        let to_center_y = to_rect.y + to_rect.height / 2.0;

        let dy = to_center_y - from_center_y;
        
        let is_backward_transition = dy < -self.config.vertical_spacing * 0.5;
        let is_to_small = to_rect.width < 30.0 && to_rect.height < 30.0;
        let is_from_small = from_rect.width < 30.0 && from_rect.height < 30.0;
        
        let points = if is_backward_transition {
            let offset = 50.0;
            let right_x = from_rect.x.max(to_rect.x) + from_rect.width.max(to_rect.width) + offset;
            
            let start = Point::new(from_rect.x + from_rect.width, from_center_y);
            let corner1 = Point::new(right_x, from_center_y);
            let corner2 = Point::new(right_x, to_center_y);
            let end = if is_to_small {
                Point::new(to_center_x + to_rect.width / 2.0, to_center_y)
            } else {
                Point::new(to_rect.x + to_rect.width, to_center_y)
            };
            
            vec![start, corner1, corner2, end]
        } else if is_from_small && dy > 0.0 {
            let start = Point::new(from_center_x, from_rect.y + from_rect.height);
            let end = Point::new(to_center_x, to_rect.y);
            vec![start, end]
        } else if is_to_small && dy > 0.0 {
            let start = Point::new(from_center_x, from_rect.y + from_rect.height);
            let end = Point::new(to_center_x, to_rect.y);
            vec![start, end]
        } else {
            let (start, end) = self.calculate_connection_points(from_rect, to_rect);
            vec![start, end]
        };

        let min_x = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
        let min_y = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
        let max_x = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
        let max_y = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
        
        let from_clean = from.replace(['[', ']', '*', '_'], "");
        let to_clean = to.replace(['[', ']', '*', '_'], "");

        LayoutElement {
            id: format!("trans_{}_{}", from_clean, to_clean),
            bounds: Rect::new(min_x, min_y, (max_x - min_x).max(1.0), (max_y - min_y).max(1.0)),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Edge {
                points,
                label: label.map(|s| s.to_string()),
                arrow_start: false,
                arrow_end: true,
                dashed: false,
                edge_type: EdgeType::Association,
                from_cardinality: None,
                to_cardinality: None,
            },
        }
    }

    /// Вычисляет точки соединения
    fn calculate_connection_points(&self, from: &Rect, to: &Rect) -> (Point, Point) {
        let from_center_x = from.x + from.width / 2.0;
        let to_center_x = to.x + to.width / 2.0;

        let dx = to_center_x - from_center_x;
        let dy = (to.y + to.height / 2.0) - (from.y + from.height / 2.0);

        if dy > 0.0 {
            let is_to_small = to.width < 30.0 && to.height < 30.0;
            
            if is_to_small {
                let from_x = if dx.abs() < 10.0 {
                    from_center_x
                } else if dx > 0.0 {
                    from_center_x + from.width * 0.2
                } else {
                    from_center_x - from.width * 0.2
                };
                
                let start = Point::new(from_x, from.y + from.height);
                let end = Point::new(to_center_x, to.y);
                (start, end)
            } else {
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
    use plantuml_ast::state::Transition;

    #[test]
    fn test_layout_simple_state_machine() {
        let mut diagram = StateDiagram::new();
        diagram.add_transition(Transition::new("[*]", "Active"));
        diagram.add_transition(Transition::new("Active", "Inactive").with_event("timeout"));
        diagram.add_transition(Transition::new("Inactive", "[*]"));

        let engine = StateLayoutEngine::new();
        let result = engine.layout(&diagram);

        assert!(result.elements.len() >= 6);
    }

    #[test]
    fn test_layout_composite_state() {
        let mut diagram = StateDiagram::new();
        
        // Создаём composite состояние
        let mut composite = State::composite("Active");
        composite.internal_transitions.push(Transition::new("[*]", "Idle"));
        composite.internal_transitions.push(Transition::new("Idle", "Running").with_event("start"));
        composite.internal_transitions.push(Transition::new("Running", "Paused").with_event("pause"));
        composite.internal_transitions.push(Transition::new("Paused", "Running").with_event("resume"));
        composite.internal_transitions.push(Transition::new("Running", "Idle").with_event("stop"));
        
        diagram.add_state(composite);
        diagram.add_transition(Transition::new("[*]", "Active"));
        diagram.add_transition(Transition::new("Active", "Inactive").with_event("disable"));
        diagram.add_transition(Transition::new("Inactive", "Active").with_event("enable"));
        diagram.add_transition(Transition::new("Inactive", "[*]").with_event("delete"));

        let engine = StateLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Должен быть composite контейнер с внутренними элементами
        let composite_elements: Vec<_> = result.elements.iter()
            .filter(|e| e.id.contains("Active"))
            .collect();
        
        assert!(!composite_elements.is_empty(), "Должны быть элементы для Active");
        
        // Inactive НЕ должен быть внутри Active (проверяем, что нет элементов с prefix Active_inner)
        // Правильный паттерн: "Active_inner_state_Inactive" или "Active_inner_trans_..._Inactive"
        let inactive_in_active = result.elements.iter()
            .any(|e| e.id.starts_with("Active_inner_") && e.id.contains("Inactive"));
        
        assert!(!inactive_in_active, "Inactive не должен быть внутри Active");
    }
}
