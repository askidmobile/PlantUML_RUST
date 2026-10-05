//! Layout engine для Gantt Diagrams
//!
//! Создаёт горизонтальную временную шкалу с задачами.

use std::collections::HashMap;

/// Отступ подписи задачи от левого края полосы (измерено по эталону).
const GANTT_LABEL_INSET: f64 = 4.0;

/// Верх подписи месяца над таблицей.
///
/// Рендерер рисует базовую линию в `bounds.y + font_size`, поэтому верх
/// считается как «базовая линия минус размер шрифта». Эталонные базовые
/// линии: 11.14 сверху и 127.55 снизу; header_y у нас равен padding.
const GANTT_MONTH_TOP_ROW_Y: f64 = 11.14 - GANTT_MONTH_FONT_SIZE - 5.0;

/// Верх подписи месяца под таблицей.
const GANTT_MONTH_BOTTOM_ROW_Y: f64 = 127.55 - GANTT_MONTH_FONT_SIZE - 5.0;

/// Размер шрифта подписей месяца (в эталоне 12).
const GANTT_MONTH_FONT_SIZE: f64 = 12.0;

/// Смещение ряда сокращённых дней недели от верха шапки (по эталону).
const GANTT_WEEKDAY_ROW_Y: f64 = 83.7;

/// Смещение ряда номеров дней месяца от верха шапки (по эталону).
const GANTT_DAY_NUMBER_ROW_Y: f64 = 97.7;

/// Смещение ряда дней недели в ШАПКЕ.
///
/// PlantUML повторяет календарь дважды: строки «Mo Tu We …» и номера дней
/// стоят и над таблицей задач, и под ней. Эталон `gantt_basic` даёт базисы
/// 23.282 и 35.3 сверху, 98.7 и 112.7 снизу. Раньше верхнего календаря
/// не было вовсе — на диаграмме отсутствовала половина календаря.
const GANTT_HEADER_WEEKDAY_ROW_Y: f64 = 8.282;

/// Смещение ряда номеров дней месяца в шапке.
const GANTT_HEADER_DAY_NUMBER_ROW_Y: f64 = 20.3;

/// Смещение строки заголовков колонок (Start, End, Duration).
///
/// Эталон `gantt_basic`: базис 22.962 при кегле 10.
const GANTT_TABLE_HEADER_ROW_Y: f64 = 7.962;

/// Смещение базиса даты задачи от верха её строки.
///
/// Эталон `gantt_basic`: первая строка начинается на 39, базис — 50.864.
const GANTT_CELL_BASELINE_OFFSET: f64 = 11.864;

/// Смещение базиса подписи задачи от верха её полосы.
///
/// Эталон `gantt_basic`: полоса на 41, подпись — на 51.2 (кегль 11).
const GANTT_TASK_LABEL_BASELINE_OFFSET: f64 = 10.2;

/// Внутренний отступ полосы задачи от границ дней.
///
/// Эталон `gantt_basic`: сетка дней начинается на 136.07, полоса — на
/// 138.069.
const GANTT_BAR_INSET: f64 = 2.0;

/// Насколько полоса КОРОЧЕ своего диапазона дней.
///
/// Эталон `gantt_basic`: 10 дней дают 156 при шаге 16, то есть `n * 16 - 4`.
const GANTT_BAR_WIDTH_TRIM: f64 = 4.0;

/// Насколько ломаная связи отступает назад от правого края предшественника.
///
/// Эталон `gantt_basic`: полоса кончается на 294.069, ломаная идёт по
/// 288.069, то есть на 6 левее.
const GANTT_LINK_BACK: f64 = 6.0;

/// Насколько кончик стрелки связи не доходит до полосы последователя.
const GANTT_LINK_HEAD: f64 = 2.0;

/// Длина треугольника стрелки связи (эталон: 292.1 → 296.1).
const GANTT_LINK_HEAD_LENGTH: f64 = 4.0;

/// Половина высоты треугольника стрелки связи (эталон: 60.2 → 68.2).
const GANTT_LINK_HALF_HEIGHT: f64 = 4.0;

/// Цвет линий сетки (эталон `gantt_basic`: `#C0C0C0`, толщина 1).
const GANTT_GRID_COLOR: &str = "#C0C0C0";

/// Ширина средней колонки таблицы задач.
///
/// Эталон `gantt_basic`: вертикали рамки стоят на 0, 41.32, 82.64 и 136.07.
const GANTT_TABLE_MID_COL: f64 = 41.32;

/// Ширина колонки таблицы задач.
///
/// Измерено по эталону PlantUML: колонки Start, End и Duration начинаются
/// на x=5, 46.3 и 87.6, то есть шаг 41.3.
const GANTT_TABLE_COL_WIDTH: f64 = 41.3;

/// Форматирует дату задачи как в PlantUML: `Jan 1`, `Feb 4`.
///
/// Даты считаются от даты старта проекта прибавлением номера дня, поэтому
/// возможен переход через месяц и год.
/// Перечисляет месяцы, покрытые диапазоном дней.
///
/// Возвращает `(номер первого дня, число дней, подпись)` для каждого
/// месяца, начиная с месяца старта проекта.
fn gantt_months(project_start: &GanttDate, total_days: u32) -> Vec<(u32, u32, String)> {
    const FULL_MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];

    let mut result: Vec<(u32, u32, String)> = Vec::new();
    if total_days == 0 {
        return result;
    }

    let mut year = project_start.year;
    let mut month = project_start.month.clamp(1, 12);
    let mut day_index = 0_u32;
    // Дней до конца месяца старта
    let mut left_in_month = gantt_days_in_month(year, month) - project_start.day.min(30);

    while day_index < total_days {
        let days = left_in_month.min(total_days - day_index);
        // Год PlantUML пишет только там, где он меняется: в эталоне
        // `gantt_basic` подписи такие — «January 2024» и «February».
        let name = FULL_MONTHS[(month - 1) as usize];
        let label = if result.is_empty() || month == 1 {
            format!("{name} {year}")
        } else {
            name.to_string()
        };
        result.push((day_index, days, label));

        day_index += days;
        month += 1;
        if month > 12 {
            month = 1;
            year += 1;
        }
        left_in_month = gantt_days_in_month(year, month);
    }

    result
}

/// Номер дня ВНУТРИ месяца для смещения `offset` дней от старта проекта.
///
/// PlantUML начинает нумерацию каждого месяца заново: в эталоне
/// `gantt_basic` после 31 января идут 1, 2, 3, 4 февраля.
fn month_day_at(project_start: &GanttDate, offset: u32) -> u32 {
    let mut year = project_start.year;
    let mut month = project_start.month.clamp(1, 12);
    let mut day = project_start.day;

    for _ in 0..offset {
        day += 1;
        if day > gantt_days_in_month(year, month) {
            day = 1;
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
        }
    }

    day
}

/// Дней в месяце (тот же упрощённый календарь, что и в подписях дат).
fn gantt_days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        _ => 30,
    }
}

fn format_gantt_date(project_start: &GanttDate, day: u32) -> String {
    /// Дней в месяце (без учёта високосности: PlantUML использует тот же
    /// упрощённый календарь при отображении диапазонов).
    fn days_in_month(year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
            2 => 28,
            _ => 30,
        }
    }

    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let mut year = project_start.year;
    let mut month = project_start.month.clamp(1, 12);
    let mut day_of_month = project_start.day + day;

    // Переносим через месяцы и годы
    loop {
        let in_month = days_in_month(year, month);
        if day_of_month <= in_month {
            break;
        }
        day_of_month -= in_month;
        month += 1;
        if month > 12 {
            month = 1;
            year += 1;
        }
    }

    let name = MONTHS.get((month - 1) as usize).copied().unwrap_or("Jan");
    format!("{name} {day_of_month}")
}

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

        // 1. Рисуем заголовок с датами.
        //
        // Высота области задач передаётся явно: раньше фон выходных
        // создавался высотой 1000.0 с комментарием «будет обрезано», но
        // обрезания нет, и calculate_bounds() раздувал высоту диаграммы
        // до ~1000px (weekends.svg: viewBox 1030 при 136–241 у остальных).
        let tasks_height =
            (diagram.tasks.len() as f64) * (self.config.row_height + self.config.row_spacing);
        self.draw_header(
            &mut elements,
            timeline_start_x,
            &project_start,
            total_days,
            &diagram
                .closed_days
                .iter()
                .map(|c| c.day)
                .collect::<Vec<_>>(),
            tasks_height,
        );

        // 2. Рисуем сетку
        self.draw_grid(
            &mut elements,
            timeline_start_x,
            timeline_width,
            diagram.tasks.len(),
            total_days,
            &diagram
                .closed_days
                .iter()
                .map(|c| c.day)
                .collect::<Vec<_>>(),
        );

        // 3. Рисуем задачи
        for (i, task) in diagram.tasks.iter().enumerate() {
            let row_y = self.config.padding
                + self.config.header_height
                + (i as f64) * (self.config.row_height + self.config.row_spacing);

            // Таблица задачи: PlantUML выводит её слева от диаграммы
            // тремя колонками — Start, End, Duration. Раньше рисовалось
            // только имя задачи, из-за чего диаграмма была заметно уже
            // эталона (394 против 800px).
            if let Some((start_day, end_day)) = task_positions.get(&task.name) {
                let col_width = GANTT_TABLE_COL_WIDTH;
                let date_text = |day: u32| format_gantt_date(&project_start, day);

                let cells = [
                    date_text(*start_day),
                    // Конец показывается как последний день включительно
                    date_text(end_day.saturating_sub(1)),
                    format!("{} days", end_day - start_day),
                ];

                for (col, text) in cells.iter().enumerate() {
                    // Ширина колонки — по фактическому тексту, а не фиксированная:
                    // в эталоне «Duration» шире «End».
                    let cell_w = self.config.text.width(text, self.config.label_font_size) + 6.0;
                    elements.push(LayoutElement {
                        id: format!("task_cell_{}_{}", i, col),
                        bounds: Rect::new(
                            self.config.padding + col as f64 * col_width,
                            row_y + GANTT_CELL_BASELINE_OFFSET - self.config.label_font_size,
                            cell_w,
                            self.config.label_font_size,
                        ),
                        text: None,
                        properties: std::collections::HashMap::new(),
                        element_type: ElementType::Text {
                            text: text.clone(),
                            font_size: self.config.label_font_size,
                        },
                    });
                }
            }

            // Бар задачи
            if let Some((start_day, end_day)) = task_positions.get(&task.name) {
                let bar_x = timeline_start_x
                    + GANTT_BAR_INSET
                    + (*start_day as f64) * self.config.day_width;
                let bar_width = (((*end_day - *start_day) as f64) * self.config.day_width
                    - GANTT_BAR_WIDTH_TRIM)
                    .max(5.0);
                let bar_y = row_y + GANTT_BAR_INSET;

                // Основной бар: заливка и обводка толщиной 1 — эталон
                // `gantt_basic` рисует их двумя прямоугольниками.
                elements.push(LayoutElement {
                    id: format!("task_bar_{}", i),
                    bounds: Rect::new(bar_x, bar_y, bar_width, self.config.bar_height),
                    text: None,
                    properties: [("stroke-width".to_string(), "1".to_string())]
                        .into_iter()
                        .collect(),
                    element_type: ElementType::Rectangle {
                        label: String::new(),
                        corner_radius: 3.0,
                    },
                });

                // Подпись задачи: PlantUML рисует её ВНУТРИ полосы, сразу
                // за её левым краем (эталон: полоса начинается на 138.069,
                // подпись — на 142.069). Раньше подпись выводилась отдельной
                // колонкой слева, а при добавлении таблицы Start/End/Duration
                // пропала вовсе.
                // Подпись помещается внутрь полосы, только если там есть
                // место. Иначе PlantUML выносит её ВПРАВО за полосу:
                // в эталоне полоса «Тестирование» шириной 76 при подписи
                // 81.136, поэтому подпись стоит на x=698.069 при правом крае
                // полосы 694.069. Именно эта подпись и определяет ширину
                // диаграммы (800 против 690 без неё).
                let label_width = self
                    .config
                    .text
                    .width(&task.name, self.config.date_font_size);
                let label_x = if label_width + GANTT_LABEL_INSET <= bar_width {
                    bar_x + GANTT_LABEL_INSET
                } else {
                    bar_x + bar_width + GANTT_LABEL_INSET
                };

                elements.push(LayoutElement {
                    id: format!("task_label_{}", i),
                    bounds: Rect::new(
                        label_x,
                        bar_y + GANTT_TASK_LABEL_BASELINE_OFFSET - self.config.date_font_size,
                        label_width,
                        self.config.date_font_size,
                    ),
                    text: None,
                    properties: std::collections::HashMap::new(),
                    element_type: ElementType::Text {
                        text: task.name.clone(),
                        font_size: self.config.date_font_size,
                    },
                });

                // Прогресс бар (если есть)
                if let Some(complete) = task.complete {
                    let progress_width = bar_width * (complete as f64 / 100.0);
                    if progress_width > 0.0 {
                        elements.push(LayoutElement {
                            id: format!("task_progress_{}", i),
                            bounds: Rect::new(bar_x, bar_y, progress_width, self.config.bar_height),
                            text: None,
                            properties: std::collections::HashMap::new(),
                            element_type: ElementType::Rectangle {
                                label: String::new(),
                                corner_radius: 3.0,
                            },
                        });
                    }
                }
            }
        }

        // 3.5. Связи «задача начинается с конца другой».
        //
        // PlantUML рисует их ломаной от низа предшественника к середине
        // последователя со стрелкой: эталон `gantt_basic` даёт
        // `M288.069,53.805 L288.069,64.207 L293.069,64.207` и треугольник
        // 292.1..296.1. Раньше связи не рисовались вовсе.
        for (i, task) in diagram.tasks.iter().enumerate() {
            let TaskStart::AtEnd(ref predecessor) = task.start else {
                continue;
            };
            let (Some((_, pred_end)), Some((succ_start, _))) = (
                task_positions.get(predecessor),
                task_positions.get(&task.name),
            ) else {
                continue;
            };
            let Some(pred_index) = diagram
                .tasks
                .iter()
                .position(|candidate| &candidate.name == predecessor)
            else {
                continue;
            };

            let pred_start_day = task_positions
                .get(predecessor)
                .map(|(start, _)| *start)
                .unwrap_or(0);
            let pred_right = timeline_start_x
                + GANTT_BAR_INSET
                + (pred_start_day as f64) * self.config.day_width
                + ((*pred_end - pred_start_day) as f64) * self.config.day_width
                - GANTT_BAR_WIDTH_TRIM;
            let succ_left =
                timeline_start_x + GANTT_BAR_INSET + (*succ_start as f64) * self.config.day_width;

            let line_x = pred_right - GANTT_LINK_BACK;
            // Ломаная входит в СЕРЕДИНУ полосы последователя.
            let center_y = self.row_bar_top(i) + self.config.bar_height / 2.0;
            let pred_bottom = self.row_bar_top(pred_index) + self.config.bar_height;

            elements.push(LayoutElement {
                id: format!("task_link_{}", i),
                bounds: Rect::new(line_x, pred_bottom, 1.0, (center_y - pred_bottom).max(1.0)),
                text: None,
                properties: HashMap::new(),
                element_type: ElementType::Edge {
                    points: vec![
                        Point::new(line_x, pred_bottom),
                        Point::new(line_x, center_y),
                        Point::new(succ_left - GANTT_LINK_BACK + 1.0, center_y),
                    ],
                    label: None,
                    arrow_start: false,
                    arrow_end: false,
                    dashed: false,
                    edge_type: EdgeType::Link,
                    from_cardinality: None,
                    to_cardinality: None,
                },
            });

            let tip_x = succ_left - GANTT_LINK_HEAD;
            let back_x = tip_x - GANTT_LINK_HEAD_LENGTH;
            elements.push(LayoutElement {
                id: format!("task_link_head_{}", i),
                bounds: Rect::new(
                    back_x,
                    center_y - GANTT_LINK_HALF_HEIGHT,
                    GANTT_LINK_HEAD_LENGTH,
                    GANTT_LINK_HALF_HEIGHT * 2.0,
                ),
                text: None,
                properties: HashMap::new(),
                element_type: ElementType::Polygon {
                    points: vec![
                        Point::new(0.0, 0.0),
                        Point::new(GANTT_LINK_HEAD_LENGTH, GANTT_LINK_HALF_HEIGHT),
                        Point::new(0.0, GANTT_LINK_HALF_HEIGHT * 2.0),
                    ],
                    label: None,
                    font_size: 0.0,
                },
            });
        }

        // 4. Рисуем разделители
        let separator_offset = 0;
        for (i, separator) in diagram.separators.iter().enumerate() {
            // Находим позицию разделителя между задачами
            // Простая логика: разделитель после каждой группы задач
            let sep_y = self.config.padding
                + self.config.header_height
                + ((separator_offset + i) as f64)
                    * (self.config.row_height + self.config.row_spacing)
                - self.config.row_spacing / 2.0;

            elements.push(LayoutElement {
                id: format!("separator_{}", i),
                bounds: Rect::new(
                    self.config.padding,
                    sep_y,
                    timeline_start_x + timeline_width - self.config.padding,
                    2.0,
                ),
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Edge {
                    points: vec![
                        Point::new(self.config.padding, sep_y),
                        Point::new(timeline_start_x + timeline_width, sep_y),
                    ],
                    label: separator.label.clone(),
                    arrow_start: false,
                    arrow_end: false,
                    dashed: true,
                    edge_type: EdgeType::Link,
                    from_cardinality: None,
                    to_cardinality: None,
                },
            });
        }

        // 5. Title
        if let Some(ref title) = diagram.metadata.title {
            elements.push(LayoutElement {
                id: "title".to_string(),
                bounds: Rect::new(self.config.padding, 5.0, 500.0, 20.0),
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Text {
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

    /// Верх полосы задачи в строке с номером `index`.
    fn row_bar_top(&self, index: usize) -> f64 {
        self.config.padding
            + self.config.header_height
            + (index as f64) * (self.config.row_height + self.config.row_spacing)
            + GANTT_BAR_INSET
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
                TaskStart::After(ref id) => positions
                    .get(id)
                    .map(|(_, end)| *end)
                    .unwrap_or(current_day),
                TaskStart::AtDate(_) => current_day, // Упрощение
                TaskStart::With(ref id) => positions
                    .get(id)
                    .map(|(start, _)| *start)
                    .unwrap_or(current_day),
                TaskStart::AtEnd(ref id) => positions
                    .get(id)
                    .map(|(_, end)| *end)
                    .unwrap_or(current_day),
            };

            let duration = match &task.duration {
                TaskDuration::Days(d) => *d,
                TaskDuration::Weeks(w) => w * 7,
                TaskDuration::Until(_) => 5, // Упрощение
                TaskDuration::EndsAt(ref id) => positions
                    .get(id)
                    .map(|(_, end)| end.saturating_sub(start_day))
                    .unwrap_or(5),
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
    ///
    /// `tasks_height` — высота области задач под заголовком. Фон выходных
    /// должен покрывать ровно её: раньше использовалась константа 1000.0,
    /// попадавшая в расчёт общих границ.
    fn draw_header(
        &self,
        elements: &mut Vec<LayoutElement>,
        start_x: f64,
        project_start: &GanttDate,
        total_days: u32,
        closed_days: &[Weekday],
        tasks_height: f64,
    ) {
        let header_y = self.config.padding;

        // Заголовки таблицы задач: Start, End, Duration.
        // PlantUML выводит их над колонками слева от диаграммы.
        for (col, title) in ["Start", "End", "Duration"].iter().enumerate() {
            elements.push(LayoutElement {
                id: format!("table_header_{col}"),
                bounds: Rect::new(
                    self.config.padding + col as f64 * GANTT_TABLE_COL_WIDTH,
                    header_y + GANTT_TABLE_HEADER_ROW_Y,
                    GANTT_TABLE_COL_WIDTH,
                    self.config.label_font_size,
                ),
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Text {
                    text: (*title).to_string(),
                    font_size: self.config.label_font_size,
                },
            });
        }

        // Метки дней
        for day in 0..total_days {
            let x = start_x + (day as f64) * self.config.day_width;

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

            // Под каждой колонкой PlantUML выводит ДВА ряда подписей:
            // сокращённый день недели (Mo, Tu, ...) на y=98.696 и номер дня
            // месяца (1, 2, 3, ...) на y=112.696. Раньше показывался только
            // каждый пятый день одним рядом, из-за чего шапка диаграммы
            // была на два ряда ниже эталонной.
            let weekday_short = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"]
                .get(day_of_week)
                .copied()
                .unwrap_or("Mo");

            // Номер дня ВНУТРИ месяца: PlantUML начинает отсчёт заново
            // с каждым месяцем. Раньше нумерация шла подряд до 35, из-за
            // чего февральские дни были подписаны как 32…35.
            let month_day = month_day_at(project_start, day);

            // Календарь повторяется дважды: в шапке и под таблицей.
            for (prefix, row_offset) in [
                ("hdr_weekday", GANTT_HEADER_WEEKDAY_ROW_Y),
                ("weekday", GANTT_WEEKDAY_ROW_Y),
            ] {
                elements.push(LayoutElement {
                    id: format!("{prefix}_{day}"),
                    bounds: Rect::new(
                        x,
                        header_y + row_offset,
                        self.config.day_width,
                        self.config.calendar_font_size,
                    ),
                    text: None,
                    properties: std::collections::HashMap::new(),
                    element_type: ElementType::Text {
                        text: weekday_short.to_string(),
                        font_size: self.config.calendar_font_size,
                    },
                });
            }

            for (prefix, row_offset) in [
                ("hdr_date", GANTT_HEADER_DAY_NUMBER_ROW_Y),
                ("date", GANTT_DAY_NUMBER_ROW_Y),
            ] {
                elements.push(LayoutElement {
                    id: format!("{prefix}_{day}"),
                    bounds: Rect::new(
                        x,
                        header_y + row_offset,
                        self.config.day_width,
                        self.config.calendar_font_size,
                    ),
                    text: None,
                    properties: std::collections::HashMap::new(),
                    element_type: ElementType::Text {
                        text: format!("{month_day}"),
                        font_size: self.config.calendar_font_size,
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
                        tasks_height,
                    ),
                    text: None,
                    properties: std::collections::HashMap::new(),
                    element_type: ElementType::Rectangle {
                        label: String::new(),
                        corner_radius: 0.0,
                    },
                });
            }
        }

        // Подписи месяцев.
        //
        // PlantUML выводит шкалу месяцев ДВАЖДЫ: над таблицей и под ней.
        // В эталоне gantt_basic «January 2024» стоит на y=11.14 и на
        // y=127.55, «February» — там же. У нас этих подписей не было
        // вовсе, то есть месяц на диаграмме не был подписан нигде.
        //
        // Подпись центрируется по своему диапазону дней: в эталоне
        // «January 2024» (31 день) занимает центр 384.07 при диапазоне
        // 136.069..632.07 — совпадает.
        for (start_day, days, label) in gantt_months(project_start, total_days) {
            let x = start_x + (start_day as f64) * self.config.day_width;
            let width = (days as f64) * self.config.day_width;

            for (id, y) in [
                ("month_top", header_y + GANTT_MONTH_TOP_ROW_Y),
                ("month_bottom", header_y + GANTT_MONTH_BOTTOM_ROW_Y),
            ] {
                elements.push(LayoutElement {
                    id: format!("{id}_{}", start_day),
                    bounds: Rect::new(x, y, width, GANTT_MONTH_FONT_SIZE),
                    text: None,
                    // Подписи месяцев PlantUML рисует ПОЛУЖИРНЫМИ:
                    // в эталоне gantt_basic все четыре подписи имеют
                    // font-weight="700".
                    properties: [("font-weight".to_string(), "700".to_string())]
                        .into_iter()
                        .collect(),
                    element_type: ElementType::Text {
                        text: label.clone(),
                        font_size: GANTT_MONTH_FONT_SIZE,
                    },
                });
            }
        }
    }

    /// Рисует сетку таблицы.
    ///
    /// Структура снята с эталона `gantt_basic` (45 линий, все `#C0C0C0`
    /// толщиной 1):
    ///   * рамка таблицы задач — горизонтали на 0 и 39 от x=0 до 136.07,
    ///     вертикали на 0, 41.32, 82.64 и 136.07 от y=0 до 89.41;
    ///   * вертикаль КАЖДОГО дня от 136.07 до 696.07 между y=39 и 89.41;
    ///   * горизонтали 39 и 89.41 от 136.07 до 696.07.
    ///
    /// Раньше рисовались только горизонтали по границам строк и редкие
    /// пунктирные вертикали «раз в неделю», а рамки не было вовсе.
    fn draw_grid(
        &self,
        elements: &mut Vec<LayoutElement>,
        start_x: f64,
        width: f64,
        num_tasks: usize,
        total_days: u32,
        _closed_days: &[Weekday],
    ) {
        let table_left = 0.0;
        let grid_top = self.config.padding + self.config.header_height;
        let grid_bottom =
            grid_top + (num_tasks as f64) * (self.config.row_height + self.config.row_spacing);
        let table_right = start_x;
        let grid_right = start_x + width;

        let line =
            |id: String, x1: f64, y1: f64, x2: f64, y2: f64, out: &mut Vec<LayoutElement>| {
                out.push(LayoutElement {
                    id,
                    bounds: Rect::new(x1.min(x2), y1.min(y2), (x2 - x1).abs(), (y2 - y1).abs()),
                    text: None,
                    properties: [
                        ("stroke".to_string(), GANTT_GRID_COLOR.to_string()),
                        ("stroke-width".to_string(), "1".to_string()),
                    ]
                    .into_iter()
                    .collect(),
                    element_type: ElementType::Edge {
                        points: vec![Point::new(x1, y1), Point::new(x2, y2)],
                        label: None,
                        arrow_start: false,
                        arrow_end: false,
                        dashed: false,
                        edge_type: EdgeType::Link,
                        from_cardinality: None,
                        to_cardinality: None,
                    },
                });
            };

        // Рамка таблицы задач
        line(
            "grid_h_top".to_string(),
            table_left,
            0.0,
            table_right,
            0.0,
            elements,
        );
        line(
            "grid_h_header".to_string(),
            table_left,
            grid_top,
            table_right,
            grid_top,
            elements,
        );
        line(
            "grid_h_bottom".to_string(),
            table_left,
            grid_bottom,
            table_right,
            grid_bottom,
            elements,
        );
        for (i, x) in [
            0.0,
            GANTT_TABLE_MID_COL,
            GANTT_TABLE_MID_COL * 2.0,
            table_right,
        ]
        .into_iter()
        .enumerate()
        {
            line(
                format!("grid_v_table_{i}"),
                x,
                0.0,
                x,
                grid_bottom,
                elements,
            );
        }

        // Вертикали дней: день 0 уже нарисован как правая граница таблицы
        for day in 1..=total_days {
            let x = start_x + (day as f64) * self.config.day_width;
            line(
                format!("grid_v_{day}"),
                x,
                grid_top,
                x,
                grid_bottom,
                elements,
            );
        }

        // Горизонтали области задач
        line(
            "grid_h_tasks_top".to_string(),
            start_x,
            grid_top,
            grid_right,
            grid_top,
            elements,
        );
        line(
            "grid_h_tasks_bottom".to_string(),
            start_x,
            grid_bottom,
            grid_right,
            grid_bottom,
            elements,
        );
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
        diagram.tasks.push(
            GanttTask::new("Task 2")
                .lasts_days(3)
                .starts_after("Task 1"),
        );

        let engine = GanttLayoutEngine::new();
        let positions = engine.calculate_task_positions(&diagram, &GanttDate::new(2024, 1, 1));

        assert_eq!(positions.get("Task 1"), Some(&(0, 5)));
        assert_eq!(positions.get("Task 2"), Some(&(5, 8)));
    }
}
