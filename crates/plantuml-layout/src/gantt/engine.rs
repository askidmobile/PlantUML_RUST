//! Layout engine для Gantt Diagrams
//!
//! Создаёт горизонтальную временную шкалу с задачами.

use std::collections::HashMap;

use plantuml_ast::gantt::{GanttDate, GanttDiagram, TaskDuration, TaskStart, Weekday};
use plantuml_model::{Point, Rect};

use super::GanttLayoutConfig;
use crate::traits::LayoutResult;
use crate::{EdgeType, ElementType, LayoutElement};

/// Layout engine для Gantt Diagrams
pub struct GanttLayoutEngine {
    config: GanttLayoutConfig,
}

impl GanttLayoutEngine {
    /// Создаёт новый layout engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: GanttLayoutConfig::default(),
        }
    }

    /// Создаёт layout engine с заданной конфигурацией
    pub fn with_config(config: GanttLayoutConfig) -> Self {
        Self { config }
    }

    /// Выполняет layout диаграммы
    pub fn layout(&self, diagram: &GanttDiagram) -> LayoutResult {
        let mut elements = Vec::new();

        // Определяем даты начала и окончания проекта
        let project_start = diagram
            .project_start
            .clone()
            .unwrap_or_else(|| GanttDate::new(2024, 1, 1));

        // Вычисляем позиции задач
        let task_positions = self.calculate_task_positions(diagram, &project_start);

        // Находим общую длительность
        let total_days = task_positions
            .values()
            .map(|(_start, end)| *end)
            .max()
            .unwrap_or(30);

        let timeline_width = (total_days as f64) * self.config.day_width;
        let timeline_start_x = self.config.padding + self.config.task_label_width;

        // 1. Рисуем заголовок с датами
        self.draw_header(
            &mut elements,
            timeline_start_x,
            &project_start,
            total_days,
            &diagram.closed_days.iter().map(|c| c.day).collect::<Vec<_>>(),
        );

        // 2. Рисуем сетку
        self.draw_grid(
            &mut elements,
            timeline_start_x,
            timeline_width,
            diagram.tasks.len(),
            total_days,
            &diagram.closed_days.iter().map(|c| c.day).collect::<Vec<_>>(),
        );

        // 3. Рисуем задачи
        for (i, task) in diagram.tasks.iter().enumerate() {
            let row_y = self.config.padding
                + self.config.header_height
                + (i as f64) * (self.config.row_height + self.config.row_spacing);

            // Метка задачи
            elements.push(LayoutElement {
                id: format!("task_label_{}", i),
                bounds: Rect::new(
                    self.config.padding,
                    row_y,
                    self.config.task_label_width - 10.0,
                    self.config.row_height,
                ),
                text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Text {
                    text: task.name.clone(),
                    font_size: self.config.label_font_size,
                },
            });

            // Бар задачи
            if let Some((start_day, end_day)) = task_positions.get(&task.name) {
                let bar_x = timeline_start_x + (*start_day as f64) * self.config.day_width;
                let bar_width = ((*end_day - *start_day) as f64) * self.config.day_width;
                let bar_y = row_y + (self.config.row_height - self.config.bar_height) / 2.0;

                // Основной бар
                elements.push(LayoutElement {
                    id: format!("task_bar_{}", i),
                    bounds: Rect::new(bar_x, bar_y, bar_width.max(5.0), self.config.bar_height),
                    text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Rectangle {
                        label: String::new(),
                        corner_radius: 3.0,
                    },
                });

                // Прогресс бар (если есть)
                if let Some(complete) = task.complete {
                    let progress_width = bar_width * (complete as f64 / 100.0);
                    if progress_width > 0.0 {
                        elements.push(LayoutElement {
                            id: format!("task_progress_{}", i),
                            bounds: Rect::new(
                                bar_x,
                                bar_y,
                                progress_width,
                                self.config.bar_height,
                            ),
                            text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Rectangle {
                                label: String::new(),
                                corner_radius: 3.0,
                            },
                        });
                    }
                }
            }
        }

        // 4. Рисуем разделители
        let separator_offset = 0;
        for (i, separator) in diagram.separators.iter().enumerate() {
            // Находим позицию разделителя между задачами
            // Простая логика: разделитель после каждой группы задач
            let sep_y = self.config.padding
                + self.config.header_height
                + ((separator_offset + i) as f64) * (self.config.row_height + self.config.row_spacing)
                - self.config.row_spacing / 2.0;

            elements.push(LayoutElement {
                id: format!("separator_{}", i),
                bounds: Rect::new(
                    self.config.padding,
                    sep_y,
                    timeline_start_x + timeline_width - self.config.padding,
                    2.0,
                ),
                text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Edge {
                    points: vec![
                        Point::new(self.config.padding, sep_y),
                        Point::new(timeline_start_x + timeline_width, sep_y),
                    ],
                    label: separator.label.clone(),
                    arrow_start: false,
                    arrow_end: false,
                    dashed: true,
                    edge_type: EdgeType::Link, from_cardinality: None, to_cardinality: None,
                },
            });
        }

        // 5. Title
        if let Some(ref title) = diagram.metadata.title {
            elements.push(LayoutElement {
                id: "title".to_string(),
                bounds: Rect::new(self.config.padding, 5.0, 500.0, 20.0),
                text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Text {
                    text: title.clone(),
                    font_size: 14.0,
                },
            });
        }

        // Вычисляем общие размеры
        let total_width = timeline_start_x + timeline_width + self.config.padding;
        let total_height = self.config.padding
            + self.config.header_height
            + (diagram.tasks.len() as f64) * (self.config.row_height + self.config.row_spacing)
            + self.config.padding;

        let mut result = LayoutResult {
            elements,
            bounds: Rect::new(0.0, 0.0, total_width, total_height),
        };
        result.calculate_bounds();
        result
    }

    /// Вычисляет позиции задач (начало и конец в днях)
    fn calculate_task_positions(
        &self,
        diagram: &GanttDiagram,
        _project_start: &GanttDate,
    ) -> HashMap<String, (u32, u32)> {
        let mut positions: HashMap<String, (u32, u32)> = HashMap::new();
        let mut current_day = 0u32;

        for task in &diagram.tasks {
            let start_day = match &task.start {
                TaskStart::AfterPrevious => current_day,
                TaskStart::After(ref id) => {
                    positions.get(id).map(|(_, end)| *end).unwrap_or(current_day)
                }
                TaskStart::AtDate(_) => current_day, // Упрощение
                TaskStart::With(ref id) => {
                    positions.get(id).map(|(start, _)| *start).unwrap_or(current_day)
                }
                TaskStart::AtEnd(ref id) => {
                    positions.get(id).map(|(_, end)| *end).unwrap_or(current_day)
                }
            };

            let duration = match &task.duration {
                TaskDuration::Days(d) => *d,
                TaskDuration::Weeks(w) => w * 7,
                TaskDuration::Until(_) => 5, // Упрощение
                TaskDuration::EndsAt(ref id) => {
                    positions.get(id).map(|(_, end)| end.saturating_sub(start_day)).unwrap_or(5)
                }
            };

            let end_day = start_day + duration;
            
            let task_id = task.id.clone().unwrap_or_else(|| task.name.clone());
            positions.insert(task_id, (start_day, end_day));
            positions.insert(task.name.clone(), (start_day, end_day));
            
            current_day = end_day;
        }

        positions
    }

    /// Рисует заголовок с датами
    fn draw_header(
        &self,
        elements: &mut Vec<LayoutElement>,
        start_x: f64,
        project_start: &GanttDate,
        total_days: u32,
        closed_days: &[Weekday],
    ) {
        let header_y = self.config.padding;

        // Метки дней
        for day in 0..total_days {
            let x = start_x + (day as f64) * self.config.day_width;
            let date_day = project_start.day + day;

            // Проверяем выходной ли это день
            let day_of_week = (day % 7) as usize;
            let weekdays = [
                Weekday::Monday,
                Weekday::Tuesday,
                Weekday::Wednesday,
                Weekday::Thursday,
                Weekday::Friday,
                Weekday::Saturday,
                Weekday::Sunday,
            ];
            let is_closed = closed_days.contains(&weekdays[day_of_week]);

            // Показываем только каждый 5-й день чтобы не было слишком плотно
            if day % 5 == 0 || day == 0 {
                elements.push(LayoutElement {
                    id: format!("date_{}", day),
                    bounds: Rect::new(x, header_y + 20.0, self.config.day_width * 5.0, 15.0),
                    text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Text {
                        text: format!("{}", date_day),
                        font_size: self.config.date_font_size,
                    },
                });
            }

            // Фон для выходных
            if is_closed {
                elements.push(LayoutElement {
                    id: format!("weekend_{}", day),
                    bounds: Rect::new(
                        x,
                        header_y + self.config.header_height,
                        self.config.day_width,
                        1000.0, // Высокое значение, будет обрезано
                    ),
                    text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Rectangle {
                        label: String::new(),
                        corner_radius: 0.0,
                    },
                });
            }
        }
    }

    /// Рисует сетку
    fn draw_grid(
        &self,
        elements: &mut Vec<LayoutElement>,
        start_x: f64,
        width: f64,
        num_tasks: usize,
        total_days: u32,
        _closed_days: &[Weekday],
    ) {
        let grid_start_y = self.config.padding + self.config.header_height;

        // Горизонтальные линии
        for i in 0..=num_tasks {
            let y = grid_start_y + (i as f64) * (self.config.row_height + self.config.row_spacing);
            elements.push(LayoutElement {
                id: format!("grid_h_{}", i),
                bounds: Rect::new(start_x, y, width, 1.0),
                text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Edge {
                    points: vec![Point::new(start_x, y), Point::new(start_x + width, y)],
                    label: None,
                    arrow_start: false,
                    arrow_end: false,
                    dashed: false,
                    edge_type: EdgeType::Link, from_cardinality: None, to_cardinality: None,
                },
            });
        }

        // Вертикальные линии (каждую неделю)
        for day in (0..=total_days).step_by(7) {
            let x = start_x + (day as f64) * self.config.day_width;
            let grid_height = (num_tasks as f64) * (self.config.row_height + self.config.row_spacing);

            elements.push(LayoutElement {
                id: format!("grid_v_{}", day),
                bounds: Rect::new(x, grid_start_y, 1.0, grid_height),
                text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Edge {
                    points: vec![
                        Point::new(x, grid_start_y),
                        Point::new(x, grid_start_y + grid_height),
                    ],
                    label: None,
                    arrow_start: false,
                    arrow_end: false,
                    dashed: true,
                    edge_type: EdgeType::Link, from_cardinality: None, to_cardinality: None,
                },
            });
        }
    }
}

impl Default for GanttLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_ast::gantt::GanttTask;

    #[test]
    fn test_layout_simple_gantt() {
        let mut diagram = GanttDiagram::new();
        diagram.tasks.push(GanttTask::new("Task 1").lasts_days(5));
        diagram.tasks.push(GanttTask::new("Task 2").lasts_days(3));

        let engine = GanttLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Должны быть элементы для задач
        assert!(!result.elements.is_empty());
    }

    #[test]
    fn test_task_position_calculation() {
        let mut diagram = GanttDiagram::new();
        diagram.tasks.push(GanttTask::new("Task 1").lasts_days(5));
        diagram
            .tasks
            .push(GanttTask::new("Task 2").lasts_days(3).starts_after("Task 1"));

        let engine = GanttLayoutEngine::new();
        let positions = engine.calculate_task_positions(&diagram, &GanttDate::new(2024, 1, 1));

        assert_eq!(positions.get("Task 1"), Some(&(0, 5)));
        assert_eq!(positions.get("Task 2"), Some(&(5, 8)));
    }
}
