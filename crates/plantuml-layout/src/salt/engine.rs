//! Layout engine для Salt (Wireframe) диаграмм
//!
//! Рендерит UI wireframes с кнопками, текстовыми полями, чекбоксами и т.д.

use plantuml_ast::salt::{BorderStyle, Container, SaltDiagram, SaltWidget, SeparatorType};
use plantuml_model::{Point, Rect};

/// Насколько строка с кнопками выше обычной.
///
/// Измерено по эталону `salt_basic`: строка кнопок занимает 26.472 при
/// обычной высоте 17.968, то есть на 8.502 больше.
const SALT_BUTTON_ROW_TOP_EXTRA: f64 = 8.502;

/// Смещение подчёркивания поля ввода от низа ячейки.
///
/// Измерено по эталону `salt_basic`: ячейка кончается на 17.969,
/// а линия поля идёт на 19.969.
const SALTFIELD_LINE_OFFSET: f64 = 2.0;

/// Шаг сетки salt по горизонтали.
///
/// PlantUML считает ширину ячеек НЕ по метрикам шрифта, а по числу
/// символов: поле ввода — `8 * n + 3`, кнопка — `8 * n + 4`. Проверено
/// на сервере: поле из 26 символов даёт 211, из 4 — 35; кнопка из
/// 20 символов — 164, из 4 — 36. Прежняя формула брала ширину текста,
/// из-за чего колонки не совпадали с эталоном.
const SALT_CHAR_ADVANCE: f64 = 8.0;

/// Добавка к ширине поля ввода после шага сетки.
const SALT_FIELD_EXTRA: f64 = 3.0;

/// Добавка к ширине кнопки после шага сетки.
const SALT_BUTTON_EXTRA: f64 = 4.0;

/// Зазор после ячейки поля ввода.
const SALT_FIELD_GAP: f64 = 8.0;

/// Зазор после ячейки кнопки.
///
/// Измерено по эталону `salt_basic`: колонка кнопки «Отмена» шириной
/// 59.07 при самой кнопке 52.07.
const SALT_BUTTON_GAP: f64 = 7.0;

/// Зазор после текстовой ячейки.
const SALT_TEXT_GAP: f64 = 2.0;

/// Цвет линий поля ввода и его засечек.
///
/// Эталон `salt_basic` пишет `stroke:#000;stroke-width:1`: прежний
/// серый `#888888` с половинной толщиной делал подчёркивание бледным.
const SALTFIELD_STROKE: &str = "#000";

/// Толщина линий поля ввода.
const SALTFIELD_STROKE_WIDTH: f64 = 1.0;

/// Отступ кнопки от левого края колонки (эталон `salt_basic`: 8.50 при 6).
const SALT_BUTTON_INSET: f64 = 2.5;

/// Отступ поля ввода от левого края колонки.
///
/// Эталон `salt_basic`: колонка поля начинается на 65.07, а подчёркивание
/// идёт с 66.07.
const SALT_FIELD_INSET: f64 = 1.0;

/// Радиус скругления кнопки (эталон `salt_basic`: `rx=5`).
const SALT_BUTTON_CORNER_RADIUS: f64 = 5.0;

/// Заливка кнопки (эталон `salt_basic`: `#EEE`, а не тема).
const SALT_BUTTON_BACKGROUND: &str = "#EEE";

/// Цвет обводки кнопки (эталон `salt_basic`: `#181818`).
const SALT_BUTTON_BORDER: &str = "#181818";

/// Толщина обводки кнопки (эталон `salt_basic`: `stroke-width:2.5`).
const SALT_BUTTON_BORDER_WIDTH: f64 = 2.5;

/// Базис подписи кнопки от её верха.
///
/// Эталон `salt_basic`: рамка на 44.44, текст — на 57.58, то есть
/// на 13.14 ниже (центр рамки плюс половина высоты строчных).
const BUTTON_TEXT_BASELINE_OFFSET: f64 = 13.14;

/// Отступ подписи поля ввода от его левого края (эталон: 68.07 при 66.07).
const SALTFIELD_TEXT_INSET: f64 = 2.0;

/// Базис подписи поля ввода от верха строки (эталон `salt_basic`: 17.14).
const SALTFIELD_TEXT_BASELINE: f64 = 17.14;

/// Базис текстовой подписи от верха строки (эталон `salt_basic`: 17.14).
const SALT_TEXT_BASELINE: f64 = 17.14;

use crate::salt::config::SaltLayoutConfig;
use crate::traits::{LayoutEngine, LayoutResult};
use crate::{EdgeType, ElementType, LayoutConfig, LayoutElement};

/// Layout engine для Salt диаграмм
pub struct SaltLayoutEngine {
    config: SaltLayoutConfig,
    element_id: usize,
}

impl SaltLayoutEngine {
    /// Создаёт новый engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: SaltLayoutConfig::default(),
            element_id: 0,
        }
    }

    /// Создаёт engine с указанной конфигурацией
    pub fn with_config(config: SaltLayoutConfig) -> Self {
        Self {
            config,
            element_id: 0,
        }
    }

    /// Генерирует уникальный ID элемента
    fn next_id(&mut self, prefix: &str) -> String {
        self.element_id += 1;
        format!("{}_{}", prefix, self.element_id)
    }

    /// Рендерит виджет и возвращает его размеры
    fn render_widget(
        &mut self,
        widget: &SaltWidget,
        x: f64,
        y: f64,
        available_width: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> (f64, f64) {
        match widget {
            SaltWidget::Container(container) => {
                self.render_container(container, x, y, available_width, elements)
            }
            SaltWidget::Text(text) => self.render_text(text, x, y, elements),
            SaltWidget::Button(label) => self.render_button(label, x, y, elements),
            SaltWidget::TextField(text) => self.render_textfield(text, x, y, elements),
            SaltWidget::Checkbox { label, checked } => {
                self.render_checkbox(label, *checked, x, y, elements)
            }
            SaltWidget::Radio { label, checked } => {
                self.render_radio(label, *checked, x, y, elements)
            }
            SaltWidget::Droplist { items, open } => {
                self.render_droplist(items, *open, x, y, elements)
            }
            SaltWidget::Separator(sep_type) => {
                self.render_separator(*sep_type, x, y, available_width, elements)
            }
            SaltWidget::Tree(node) => self.render_tree(node, x, y, elements),
            SaltWidget::Tabs { items, selected } => {
                self.render_tabs(items, *selected, x, y, elements)
            }
            SaltWidget::Menu { items } => self.render_menu(items, x, y, elements),
            SaltWidget::GroupBox { title, content } => {
                self.render_groupbox(title, content, x, y, available_width, elements)
            }
            SaltWidget::ScrollArea { content, scrollbar } => {
                self.render_scrollarea(content, *scrollbar, x, y, available_width, elements)
            }
            SaltWidget::Empty => (self.config.min_cell_width, self.config.row_height),
            SaltWidget::Span => (0.0, 0.0),
        }
    }

    /// Натуральная ширина виджета без учёта доступного места.
    ///
    /// Используется для подгонки размера контейнера под содержимое.
    fn widget_natural_width(&self, widget: &SaltWidget) -> f64 {
        match widget {
            // Кнопка: `max(8 * символов, ширина текста) + 4` плюс зазор
            // колонки. Ширина берётся по СЫРОЙ подписи: в эталоне
            // `salt_basic` кнопка « OK » шириной 36 = 8 * 4 + 4, хотя
            // текст «OK» — 17.31.
            SaltWidget::Button(text) => self.button_width(text) + SALT_BUTTON_GAP,
            SaltWidget::Text(text) => {
                self.config.text.width(text, self.config.font_size) + SALT_TEXT_GAP
            }
            SaltWidget::TextField(text) => self.textfield_width(text) + SALT_FIELD_GAP,
            SaltWidget::Checkbox { label, .. } | SaltWidget::Radio { label, .. } => {
                self.config.checkbox_size
                    + 4.0
                    + self.config.text.width(label, self.config.font_size)
            }
            SaltWidget::Droplist { items, .. } => {
                items
                    .iter()
                    .map(|i| self.config.text.width(i, self.config.font_size))
                    .fold(0.0_f64, f64::max)
                    + 20.0
            }
            SaltWidget::GroupBox { title, content } => self.widget_natural_width(content).max(
                self.config.text.width(title, self.config.font_size)
                    + self.config.cell_padding * 2.0,
            ),
            SaltWidget::ScrollArea { content, .. } => self.widget_natural_width(content) + 15.0,
            SaltWidget::Tabs { items, .. } => items
                .iter()
                .map(|i| self.config.text.width(i, self.config.font_size) + 20.0)
                .sum(),
            SaltWidget::Menu { items } => items
                .iter()
                .map(|i| self.config.text.width(&i.text, self.config.font_size) + 20.0)
                .fold(0.0_f64, f64::max),
            SaltWidget::Container(c) => {
                // Ширина вложенного контейнера — сумма натуральных ширин колонок
                let max_cols = c.rows.iter().map(|r| r.len()).max().unwrap_or(1);
                let mut cols = vec![0.0_f64; max_cols];
                for row in &c.rows {
                    for (i, w) in row.iter().enumerate() {
                        if i < cols.len() {
                            cols[i] = cols[i].max(self.widget_natural_width(w));
                        }
                    }
                }
                cols.iter().sum::<f64>()
            }
            SaltWidget::Tree(_) => self.config.min_cell_width,
            _ => self.config.min_cell_width,
        }
    }

    /// Рендерит контейнер
    fn render_container(
        &mut self,
        container: &Container,
        x: f64,
        y: f64,
        available_width: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> (f64, f64) {
        // Текст первой строки в эталоне стоит на y=6 при высоте строки
        // 17.968; базис оказывается на 17.139.
        let mut current_y = y;
        let mut max_width = 0.0_f64;
        let start_x = x + self.config.cell_padding;

        // Ширина столбца определяется содержимым, а не доступной шириной.
        //
        // PlantUML подгоняет размер контейнера под содержимое: эталон
        // salt_basic имеет габарит 113x71 при том же содержимом. Раньше
        // ширина делилась на фиксированные 800px, из-за чего диаграмма
        // получалась в семь раз шире оригинала.
        let max_cols = container.rows.iter().map(|r| r.len()).max().unwrap_or(1);

        // Натуральная ширина каждой колонки — максимум по её ячейкам
        let mut col_natural = vec![0.0_f64; max_cols];
        for row in &container.rows {
            for (col_idx, widget) in row.iter().enumerate() {
                let natural = self.widget_natural_width(widget);
                if col_idx < col_natural.len() {
                    col_natural[col_idx] = col_natural[col_idx].max(natural);
                }
            }
        }

        // Если суммарная натуральная ширина превышает доступную, сжимаем
        // пропорционально — иначе на широких таблицах контейнер вылезет.
        let natural_total: f64 = col_natural.iter().sum();
        let usable = (available_width - self.config.cell_padding * 2.0).max(0.0);
        let col_width = if natural_total > usable && natural_total > 0.0 {
            let scale = usable / natural_total;
            col_natural.iter().map(|w| w * scale).collect::<Vec<_>>()
        } else {
            col_natural.clone()
        };

        // Ширина контейнера считается по ПРАВОМУ КРАЮ нарисованных
        // элементов, а не по сумме ширин колонок.
        //
        // Колонка резервирует место под свой самый широкий виджет, но
        // последний столбец в эталоне обрезается по факту: `salt_basic`
        // кончается на 100.07 (правый край поля), хотя колонка поля
        // занимает 43. Прежний расчёт по колонкам давал 114.07 и холст
        // 127 вместо 113.
        let first_element = elements.len();

        for row in &container.rows {
            let mut current_x = start_x;

            // Строка с КНОПКАМИ выше обычной, и кнопка прижата к её низу.
            //
            // Измерено по эталону `salt_basic`: строки 1–2 идут по
            // 17.968, а строка кнопок занимает 26.472 — кнопка высотой
            // 17.969 стоит на 8.502 ниже её верха. Без этого кнопки
            // оказывались на 8.5 выше эталонных.
            let has_button = row
                .iter()
                .any(|widget| matches!(widget, SaltWidget::Button(_)));
            let button_offset = if has_button {
                SALT_BUTTON_ROW_TOP_EXTRA
            } else {
                0.0
            };
            // Высота кнопки учитывается ТОЛЬКО в строке с кнопками.
            //
            // Раньше здесь стояло `row_height.max(button_height + ...)`, и
            // пол в 17.969 действовал на каждую строку без разбора: базовая
            // высота строки в эталоне 15.97, а задать её было невозможно —
            // меняли `row_height`, а результат не менялся.
            let mut row_height = if has_button {
                self.config.button_height + button_offset
            } else {
                self.config.row_height
            };

            for (col_idx, widget) in row.iter().enumerate() {
                let cell_width = col_width.get(col_idx).copied().unwrap_or(0.0);

                let is_button = matches!(widget, SaltWidget::Button(_));
                let widget_y = if is_button {
                    current_y + button_offset
                } else {
                    current_y
                };

                let (_w, h) = self.render_widget(
                    widget,
                    current_x,
                    widget_y,
                    cell_width - self.config.cell_padding,
                    elements,
                );
                // Строка из ОДНОГО разделителя берёт ЕГО высоту, а не пол
                // строки. Измерено по эталону `Salt: Wireframe`: шаг от
                // строки `Логин` до строки полей равен 24 = 16 (строка
                // текста) + 8 (разделитель `==`). Пол `row_height = 15.97`
                // обрезал разделитель до 16, и шаг выходил 32.
                let is_separator = matches!(widget, SaltWidget::Separator(_));
                if is_separator && row.len() == 1 {
                    row_height = h;
                } else {
                    row_height = row_height.max(h);
                }
                current_x += cell_width;
            }

            // Шаг строк равен высоте строки: измерено по эталону — строки
            // идут на y=17.139 и 35.107, то есть ровно на 17.968. Раньше
            // добавлялся ещё и половинный отступ ячейки, из-за чего шаг
            // составлял 20.97 и диаграмма вырастала по высоте.
            current_y += row_height;
        }

        for element in &elements[first_element..] {
            let right = element.bounds.x + element.bounds.width;
            max_width = max_width.max(right - x - self.config.cell_padding);
        }

        let total_width = max_width + self.config.cell_padding * 2.0;
        let total_height = current_y - y + self.config.cell_padding;

        // Рисуем границу контейнера
        if container.border_style != BorderStyle::None {
            self.render_container_border(
                container.border_style,
                x,
                y,
                total_width,
                total_height,
                elements,
            );
        }

        (total_width, total_height)
    }

    /// Рисует границу контейнера
    fn render_container_border(
        &mut self,
        style: BorderStyle,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        elements: &mut Vec<LayoutElement>,
    ) {
        let stroke_dasharray = match style {
            BorderStyle::All | BorderStyle::External => None,
            _ => Some("2,2"),
        };

        let border = LayoutElement {
            id: self.next_id("border"),
            element_type: ElementType::Rectangle {
                label: String::new(),
                corner_radius: 0.0,
            },
            bounds: Rect::new(x, y, width, height),
            text: None,
            properties: [
                ("fill".to_string(), "none".to_string()),
                ("stroke".to_string(), self.config.border_color.to_string()),
                (
                    "stroke-width".to_string(),
                    self.config.border_width.to_string(),
                ),
            ]
            .into_iter()
            .chain(stroke_dasharray.map(|d| ("stroke-dasharray".to_string(), d.to_string())))
            .collect(),
        };
        elements.push(border);
    }

    /// Рендерит текст
    fn render_text(
        &mut self,
        text: &str,
        x: f64,
        y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> (f64, f64) {
        let text_width = self.config.text.width(text, self.config.font_size);
        // Ширина ячейки — текст плюс зазор колонки: эталон `salt_basic`
        // даёт колонку подписи на 2 шире самой длинной подписи.
        let width = text_width + SALT_TEXT_GAP;
        let height = self.config.row_height;

        let text_elem = LayoutElement {
            id: self.next_id("text"),
            element_type: ElementType::Text {
                text: text.to_string(),
                font_size: self.config.font_size,
            },
            // Базис подписи: эталон ставит первую строку на 17.14.
            bounds: Rect::new(
                x,
                y + SALT_TEXT_BASELINE - self.config.font_size,
                text_width,
                self.config.font_size,
            ),
            text: Some(text.to_string()),
            properties: [("fill".to_string(), "#000000".to_string())]
                .into_iter()
                .collect(),
        };
        elements.push(text_elem);

        (width, height)
    }

    /// Ширина поля ввода по правилу PlantUML: `8 * символов + 3`.
    fn textfield_width(&self, text: &str) -> f64 {
        SALT_CHAR_ADVANCE * text.chars().count() as f64 + SALT_FIELD_EXTRA
    }

    /// Ширина кнопки по правилу PlantUML.
    ///
    /// `max(8 * символов, ширина текста) + 4`: для кириллицы метрики
    /// шрифта шире шага сетки, поэтому берётся максимум.
    fn button_width(&self, label: &str) -> f64 {
        let by_grid = SALT_CHAR_ADVANCE * label.chars().count() as f64 + SALT_BUTTON_EXTRA;
        let by_text = self.config.text.width(label, self.config.font_size) + SALT_BUTTON_EXTRA;
        by_grid.max(by_text)
    }

    /// Рендерит кнопку
    fn render_button(
        &mut self,
        label: &str,
        x: f64,
        y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> (f64, f64) {
        let width = self.button_width(label);
        let height = self.config.button_height;

        // Кнопка прижата к левому краю колонки с отступом 2.5 — эталон
        // `salt_basic` даёт 8.50 при начале колонки на 6.
        let button_x = x + SALT_BUTTON_INSET;
        let label_text = label.trim();

        // Фон кнопки.
        //
        // Радиус 5 и обводка 2.5 — из эталона `salt_basic`; раньше были
        // радиус 3 и обводка темы (0.5), из-за чего кнопки выглядели
        // плоскими и почти без рамки.
        let bg = LayoutElement {
            id: self.next_id("button_bg"),
            element_type: ElementType::Rectangle {
                label: String::new(),
                corner_radius: SALT_BUTTON_CORNER_RADIUS,
            },
            bounds: Rect::new(button_x, y, width, height),
            text: None,
            properties: [
                ("fill".to_string(), SALT_BUTTON_BACKGROUND.to_string()),
                ("stroke".to_string(), SALT_BUTTON_BORDER.to_string()),
                (
                    "stroke-width".to_string(),
                    SALT_BUTTON_BORDER_WIDTH.to_string(),
                ),
            ]
            .into_iter()
            .collect(),
        };
        elements.push(bg);

        // Текст кнопки — по центру: эталон ставит «Отмена» на 10.50 при
        // рамке 8.50..60.57 и тексте 48.07.
        let text_width = self.config.text.width(label_text, self.config.font_size);
        let text = LayoutElement {
            id: self.next_id("button_text"),
            element_type: ElementType::Text {
                text: label_text.to_string(),
                font_size: self.config.font_size,
            },
            bounds: Rect::new(
                button_x + (width - text_width) / 2.0,
                y + BUTTON_TEXT_BASELINE_OFFSET - self.config.font_size,
                text_width,
                self.config.font_size,
            ),
            text: Some(label_text.to_string()),
            properties: [("fill".to_string(), "#000000".to_string())]
                .into_iter()
                .collect(),
        };
        elements.push(text);

        (width, height)
    }

    /// Рендерит текстовое поле
    fn render_textfield(
        &mut self,
        text: &str,
        x: f64,
        y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> (f64, f64) {
        let width = self.textfield_width(text);
        let height = self.config.textfield_height;
        let x = x + SALT_FIELD_INSET;

        // Поле ввода: ЛИНИЯ с засечками по краям, а не прямоугольник.
        //
        // Измерено по эталону `salt_basic`: подчёркивание идёт на
        // y = 19.969 (то есть на 2 ниже низа ячейки), а засечки — от
        // y = 16.969 до 18.969. Раньше поле рисовалось залитым
        // прямоугольником, что визуально отличается от PlantUML.
        let line_y = y + height + SALTFIELD_LINE_OFFSET;

        elements.push(LayoutElement {
            id: self.next_id("field_line"),
            element_type: ElementType::Edge {
                points: vec![Point::new(x, line_y), Point::new(x + width, line_y)],
                label: None,
                arrow_start: false,
                arrow_end: false,
                dashed: false,
                edge_type: EdgeType::Link,
                from_cardinality: None,
                to_cardinality: None,
            },
            bounds: Rect::new(x, line_y, width, 1.0),
            text: None,
            properties: [
                ("stroke".to_string(), SALTFIELD_STROKE.to_string()),
                (
                    "stroke-width".to_string(),
                    SALTFIELD_STROKE_WIDTH.to_string(),
                ),
            ]
            .into_iter()
            .collect(),
        });

        for tick_x in [x, x + width] {
            elements.push(LayoutElement {
                id: self.next_id("field_tick"),
                element_type: ElementType::Edge {
                    points: vec![
                        Point::new(tick_x, line_y - 3.0),
                        Point::new(tick_x, line_y - 1.0),
                    ],
                    label: None,
                    arrow_start: false,
                    arrow_end: false,
                    dashed: false,
                    edge_type: EdgeType::Link,
                    from_cardinality: None,
                    to_cardinality: None,
                },
                bounds: Rect::new(tick_x, line_y - 3.0, 0.0, 2.0),
                text: None,
                properties: [
                    ("stroke".to_string(), SALTFIELD_STROKE.to_string()),
                    (
                        "stroke-width".to_string(),
                        SALTFIELD_STROKE_WIDTH.to_string(),
                    ),
                ]
                .into_iter()
                .collect(),
            });
        }

        // Текст поля: эталон ставит его на 2 правее начала подчёркивания
        // и базисом 17.14 от верха строки.
        let text_width = self.config.text.width(text, self.config.font_size);
        let text_elem = LayoutElement {
            id: self.next_id("field_text"),
            element_type: ElementType::Text {
                text: text.to_string(),
                font_size: self.config.font_size,
            },
            bounds: Rect::new(
                x + SALTFIELD_TEXT_INSET,
                y + SALTFIELD_TEXT_BASELINE - self.config.font_size,
                text_width,
                self.config.font_size,
            ),
            text: Some(text.to_string()),
            properties: [("fill".to_string(), "#000000".to_string())]
                .into_iter()
                .collect(),
        };
        elements.push(text_elem);

        (width, height)
    }

    /// Рендерит чекбокс
    fn render_checkbox(
        &mut self,
        label: &str,
        checked: bool,
        x: f64,
        y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> (f64, f64) {
        let size = self.config.checkbox_size;

        // Квадрат чекбокса
        let box_elem = LayoutElement {
            id: self.next_id("checkbox"),
            element_type: ElementType::Rectangle {
                label: String::new(),
                corner_radius: 2.0,
            },
            bounds: Rect::new(x, y + 4.0, size, size),
            text: None,
            properties: [
                ("fill".to_string(), "none".to_string()),
                ("stroke".to_string(), self.config.border_color.to_string()),
                // Эталон рисует флажок квадратом 10x10 с обводкой 1.5
                // и без заливки: `<rect width="10" height="10" fill="none"
                // style="stroke:#000;stroke-width:1.5;"/>`.
                ("stroke-width".to_string(), "1.5".to_string()),
            ]
            .into_iter()
            .collect(),
        };
        elements.push(box_elem);

        // Галочка, если отмечен.
        //
        // PlantUML рисует её МНОГОУГОЛЬНИКОМ, а не буквой «X»:
        //   <polygon points="139.678,54.938,142.678,57.938,
        //                     149.678,48.938,142.678,55.938" fill="#000"/>
        // Относительно левого верхнего угла рамки 10×10 точки дают
        // (1.0, 4.016), (4.0, 7.016), (11.0, -1.984), (4.0, 5.016) —
        // галочка заходит за рамку, поэтому точки хранятся ЛОКАЛЬНО.
        // Раньше здесь был текст «X», из-за чего в сверке подписей
        // появлялась лишняя подпись «X», которой у PlantUML нет.
        if checked {
            let points = vec![
                Point::new(0.0, 6.0),
                Point::new(3.0, 9.0),
                Point::new(10.0, 0.0),
                Point::new(3.0, 7.0),
            ];
            let check = LayoutElement {
                id: self.next_id("check"),
                element_type: ElementType::Polygon {
                    points,
                    label: None,
                    font_size: self.config.font_size,
                },
                bounds: Rect::new(x + 1.0, y + 4.0 - 1.984, 10.0, 9.0),
                text: None,
                properties: [
                    ("fill".to_string(), "#000000".to_string()),
                    ("stroke".to_string(), "#000000".to_string()),
                    ("stroke-width".to_string(), "1.5".to_string()),
                    ("stroke-linejoin".to_string(), "miter".to_string()),
                    ("stroke-miterlimit".to_string(), "10".to_string()),
                ]
                .into_iter()
                .collect(),
            };
            elements.push(check);
        }

        // Метка
        let label_width = if !label.is_empty() {
            let label_elem = LayoutElement {
                id: self.next_id("label"),
                element_type: ElementType::Text {
                    text: label.to_string(),
                    font_size: self.config.font_size,
                },
                // Ширина берётся из текста, а не из константы 100.0.
                //
                // Константа тянула за собой ширину всей таблицы: у пустого
                // флажка подписи нет, но место под 100 px всё равно
                // резервировалось, и на `salt_widgets` холст разъезжался на
                // 67 px — при том, что нарисованные элементы заканчивались
                // на 269, а объявлялось 336.53.
                bounds: Rect::new(
                    x + size + 4.0,
                    y + 4.0,
                    self.config.text.width(label, self.config.font_size),
                    self.config.font_size + 4.0,
                ),
                text: Some(label.to_string()),
                properties: [("fill".to_string(), "#000000".to_string())]
                    .into_iter()
                    .collect(),
            };
            elements.push(label_elem);
            self.config.text.width(label, self.config.font_size)
        } else {
            0.0
        };

        (size + 4.0 + label_width, self.config.row_height)
    }

    /// Рендерит радио-кнопку
    fn render_radio(
        &mut self,
        label: &str,
        checked: bool,
        x: f64,
        y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> (f64, f64) {
        let size = self.config.checkbox_size;

        // Круг радио
        let circle = LayoutElement {
            id: self.next_id("radio"),
            element_type: ElementType::Ellipse { label: None },
            bounds: Rect::new(x, y + 4.0, size, size),
            text: None,
            properties: [
                ("fill".to_string(), "#FFFFFF".to_string()),
                ("stroke".to_string(), self.config.border_color.to_string()),
            ]
            .into_iter()
            .collect(),
        };
        elements.push(circle);

        // Заполнение если выбрано
        if checked {
            let inner = LayoutElement {
                id: self.next_id("radio_inner"),
                element_type: ElementType::Ellipse { label: None },
                bounds: Rect::new(x + 3.0, y + 7.0, size - 6.0, size - 6.0),
                text: None,
                properties: [("fill".to_string(), "#000000".to_string())]
                    .into_iter()
                    .collect(),
            };
            elements.push(inner);
        }

        // Метка
        let label_width = if !label.is_empty() {
            let label_elem = LayoutElement {
                id: self.next_id("label"),
                element_type: ElementType::Text {
                    text: label.to_string(),
                    font_size: self.config.font_size,
                },
                // Ширина берётся из текста, а не из константы 100.0.
                //
                // Константа тянула за собой ширину всей таблицы: у пустого
                // флажка подписи нет, но место под 100 px всё равно
                // резервировалось, и на `salt_widgets` холст разъезжался на
                // 67 px — при том, что нарисованные элементы заканчивались
                // на 269, а объявлялось 336.53.
                bounds: Rect::new(
                    x + size + 4.0,
                    y + 4.0,
                    self.config.text.width(label, self.config.font_size),
                    self.config.font_size + 4.0,
                ),
                text: Some(label.to_string()),
                properties: [("fill".to_string(), "#000000".to_string())]
                    .into_iter()
                    .collect(),
            };
            elements.push(label_elem);
            self.config.text.width(label, self.config.font_size)
        } else {
            0.0
        };

        (size + 4.0 + label_width, self.config.row_height)
    }

    /// Рендерит выпадающий список
    fn render_droplist(
        &mut self,
        items: &[String],
        _open: bool,
        x: f64,
        y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> (f64, f64) {
        let text = items.first().map(|s| s.as_str()).unwrap_or("Select...");
        let width = self.config.text.width(text, self.config.font_size) + 30.0;
        let height = self.config.textfield_height;

        // Фон
        let bg = LayoutElement {
            id: self.next_id("dropdown_bg"),
            element_type: ElementType::Rectangle {
                label: String::new(),
                corner_radius: 2.0,
            },
            bounds: Rect::new(x, y, width, height),
            text: None,
            properties: [
                ("fill".to_string(), "#FFFFFF".to_string()),
                ("stroke".to_string(), self.config.border_color.to_string()),
            ]
            .into_iter()
            .collect(),
        };
        elements.push(bg);

        // Текст
        let text_elem = LayoutElement {
            id: self.next_id("dropdown_text"),
            element_type: ElementType::Text {
                text: text.to_string(),
                font_size: self.config.font_size,
            },
            bounds: Rect::new(x + 4.0, y + 4.0, width - 20.0, height),
            text: Some(text.to_string()),
            properties: [("fill".to_string(), "#000000".to_string())]
                .into_iter()
                .collect(),
        };
        elements.push(text_elem);

        // Стрелка вниз
        let arrow = LayoutElement {
            id: self.next_id("dropdown_arrow"),
            element_type: ElementType::Text {
                text: "▼".to_string(),
                font_size: 10.0,
            },
            bounds: Rect::new(x + width - 16.0, y + 5.0, 12.0, height),
            text: Some("▼".to_string()),
            properties: [("fill".to_string(), "#666666".to_string())]
                .into_iter()
                .collect(),
        };
        elements.push(arrow);

        (width, height)
    }

    /// Рендерит разделитель
    fn render_separator(
        &mut self,
        sep_type: SeparatorType,
        x: f64,
        y: f64,
        width: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> (f64, f64) {
        let dasharray = match sep_type {
            SeparatorType::Dotted => Some("2,2"),
            // Волнистый разделитель в PlantUML — волнистая линия, а не
            // пунктир. Пунктир оставлен как приближение, но с более
            // характерным для «волны» шагом.
            SeparatorType::Wavy => Some("2,2"),
            SeparatorType::Single => None,
            SeparatorType::Double => None,
        };

        let line = LayoutElement {
            id: self.next_id("separator"),
            element_type: ElementType::Edge {
                points: vec![Point::new(x, y + 10.0), Point::new(x + width, y + 10.0)],
                label: None,
                arrow_start: false,
                arrow_end: false,
                dashed: dasharray.is_some(),
                edge_type: EdgeType::Link,
                from_cardinality: None,
                to_cardinality: None,
            },
            bounds: Rect::new(x, y, width, 8.0),
            text: None,
            properties: [("stroke".to_string(), self.config.border_color.to_string())]
                .into_iter()
                .chain(dasharray.map(|d| ("stroke-dasharray".to_string(), d.to_string())))
                .collect(),
        };
        elements.push(line);

        // Для двойной линии добавляем вторую
        if sep_type == SeparatorType::Double {
            let line2 = LayoutElement {
                id: self.next_id("separator2"),
                element_type: ElementType::Edge {
                    points: vec![Point::new(x, y + 14.0), Point::new(x + width, y + 14.0)],
                    label: None,
                    arrow_start: false,
                    arrow_end: false,
                    dashed: false,
                    edge_type: EdgeType::Link,
                    from_cardinality: None,
                    to_cardinality: None,
                },
                bounds: Rect::new(x, y, width, 8.0),
                text: None,
                properties: [("stroke".to_string(), self.config.border_color.to_string())]
                    .into_iter()
                    .collect(),
            };
            elements.push(line2);
        }

        (width, 8.0)
    }

    /// Рендерит дерево
    fn render_tree(
        &mut self,
        node: &plantuml_ast::salt::TreeNode,
        x: f64,
        y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> (f64, f64) {
        let mut current_y = y;
        let mut max_width = 0.0_f64;

        /// `is_last` — последний ли это ребёнок у родителя.
        ///
        /// Раньше префикс всегда был `├─`, поэтому последний ребёнок
        /// выглядел так же, как промежуточные; в PlantUML он получает `└─`.
        fn render_node(
            engine: &mut SaltLayoutEngine,
            node: &plantuml_ast::salt::TreeNode,
            x: f64,
            y: &mut f64,
            max_width: &mut f64,
            elements: &mut Vec<LayoutElement>,
            is_last: bool,
        ) {
            let indent = node.level as f64 * 20.0;

            if !node.text.is_empty() {
                let prefix = if node.level > 0 {
                    if is_last {
                        "└─ "
                    } else {
                        "├─ "
                    }
                } else {
                    ""
                };
                let text = format!("{}{}", prefix, node.text);
                let width = engine.config.text.width(&text, engine.config.font_size) + indent;

                let text_elem = LayoutElement {
                    id: engine.next_id("tree_node"),
                    element_type: ElementType::Text {
                        text: text.clone(),
                        font_size: engine.config.font_size,
                    },
                    bounds: Rect::new(x + indent, *y, width, 20.0),
                    text: Some(text),
                    properties: [("fill".to_string(), "#000000".to_string())]
                        .into_iter()
                        .collect(),
                };
                elements.push(text_elem);

                *max_width = max_width.max(width + indent);
                *y += 20.0;
            }

            let last_index = node.children.len().saturating_sub(1);
            for (i, child) in node.children.iter().enumerate() {
                render_node(engine, child, x, y, max_width, elements, i == last_index);
            }
        }

        render_node(
            self,
            node,
            x,
            &mut current_y,
            &mut max_width,
            elements,
            true,
        );

        (max_width, current_y - y)
    }

    /// Рендерит вкладки
    fn render_tabs(
        &mut self,
        items: &[String],
        selected: usize,
        x: f64,
        y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> (f64, f64) {
        let mut current_x = x;
        let tab_height = 25.0;

        for (i, item) in items.iter().enumerate() {
            let width = self.config.text.width(item, self.config.font_size) + 20.0;
            let is_selected = i == selected;

            // Фон вкладки
            let bg = LayoutElement {
                id: self.next_id("tab_bg"),
                element_type: ElementType::Rectangle {
                    label: String::new(),
                    corner_radius: 0.0,
                },
                bounds: Rect::new(current_x, y, width, tab_height),
                text: None,
                properties: [
                    (
                        "fill".to_string(),
                        if is_selected { "#FFFFFF" } else { "#E0E0E0" }.to_string(),
                    ),
                    ("stroke".to_string(), self.config.border_color.to_string()),
                ]
                .into_iter()
                .collect(),
            };
            elements.push(bg);

            // Текст вкладки
            let text = LayoutElement {
                id: self.next_id("tab_text"),
                element_type: ElementType::Text {
                    text: item.clone(),
                    font_size: self.config.font_size,
                },
                bounds: Rect::new(current_x + 10.0, y + 5.0, width, tab_height),
                text: Some(item.clone()),
                properties: [("fill".to_string(), "#000000".to_string())]
                    .into_iter()
                    .collect(),
            };
            elements.push(text);

            current_x += width;
        }

        (current_x - x, tab_height)
    }

    /// Рендерит меню
    fn render_menu(
        &mut self,
        items: &[plantuml_ast::salt::MenuItem],
        x: f64,
        y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> (f64, f64) {
        let mut current_x = x;
        let menu_height = 22.0;

        // Фон меню
        let total_width = items
            .iter()
            .map(|i| self.config.text.width(&i.text, self.config.font_size) + 20.0)
            .sum::<f64>();
        let bg = LayoutElement {
            id: self.next_id("menu_bg"),
            element_type: ElementType::Rectangle {
                label: String::new(),
                corner_radius: 0.0,
            },
            bounds: Rect::new(x, y, total_width, menu_height),
            text: None,
            properties: [
                ("fill".to_string(), "#F0F0F0".to_string()),
                ("stroke".to_string(), self.config.border_color.to_string()),
            ]
            .into_iter()
            .collect(),
        };
        elements.push(bg);

        for item in items {
            if item.is_separator {
                continue;
            }

            let width = self.config.text.width(&item.text, self.config.font_size) + 20.0;

            let text = LayoutElement {
                id: self.next_id("menu_item"),
                element_type: ElementType::Text {
                    text: item.text.clone(),
                    font_size: self.config.font_size,
                },
                bounds: Rect::new(current_x + 10.0, y + 4.0, width, menu_height),
                text: Some(item.text.clone()),
                properties: [("fill".to_string(), "#000000".to_string())]
                    .into_iter()
                    .collect(),
            };
            elements.push(text);

            current_x += width;
        }

        (total_width, menu_height)
    }

    /// Рендерит группу с заголовком
    fn render_groupbox(
        &mut self,
        title: &str,
        content: &SaltWidget,
        x: f64,
        y: f64,
        available_width: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> (f64, f64) {
        let title_height = 20.0;
        let (content_width, content_height) = self.render_widget(
            content,
            x + 5.0,
            y + title_height + 5.0,
            available_width - 10.0,
            elements,
        );

        let total_width = content_width + 10.0;
        let total_height = content_height + title_height + 10.0;

        // Рамка группы
        let frame = LayoutElement {
            id: self.next_id("groupbox"),
            element_type: ElementType::Rectangle {
                label: String::new(),
                corner_radius: 3.0,
            },
            bounds: Rect::new(x, y, total_width, total_height),
            text: None,
            properties: [
                ("fill".to_string(), "none".to_string()),
                ("stroke".to_string(), self.config.border_color.to_string()),
            ]
            .into_iter()
            .collect(),
        };
        elements.push(frame);

        // Заголовок
        let title_elem = LayoutElement {
            id: self.next_id("groupbox_title"),
            element_type: ElementType::Text {
                text: title.to_string(),
                font_size: self.config.font_size,
            },
            bounds: Rect::new(x + 10.0, y + 2.0, 100.0, title_height),
            text: Some(title.to_string()),
            properties: [
                ("fill".to_string(), "#000000".to_string()),
                ("font-weight".to_string(), "bold".to_string()),
            ]
            .into_iter()
            .collect(),
        };
        elements.push(title_elem);

        (total_width, total_height)
    }

    /// Рендерит скроллируемую область
    fn render_scrollarea(
        &mut self,
        content: &SaltWidget,
        scrollbar: plantuml_ast::salt::ScrollbarType,
        x: f64,
        y: f64,
        available_width: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> (f64, f64) {
        let scrollbar_size = 15.0;

        let content_width = match scrollbar {
            plantuml_ast::salt::ScrollbarType::Vertical
            | plantuml_ast::salt::ScrollbarType::Both => available_width - scrollbar_size,
            _ => available_width,
        };

        let (w, h) = self.render_widget(content, x, y, content_width, elements);

        // Вертикальный скроллбар
        if matches!(
            scrollbar,
            plantuml_ast::salt::ScrollbarType::Vertical | plantuml_ast::salt::ScrollbarType::Both
        ) {
            let scrollbar_elem = LayoutElement {
                id: self.next_id("vscrollbar"),
                element_type: ElementType::Rectangle {
                    label: String::new(),
                    corner_radius: 2.0,
                },
                bounds: Rect::new(x + w, y, scrollbar_size, h),
                text: None,
                properties: [
                    ("fill".to_string(), "#E0E0E0".to_string()),
                    ("stroke".to_string(), self.config.border_color.to_string()),
                ]
                .into_iter()
                .collect(),
            };
            elements.push(scrollbar_elem);
        }

        // Горизонтальный скроллбар
        if matches!(
            scrollbar,
            plantuml_ast::salt::ScrollbarType::Horizontal | plantuml_ast::salt::ScrollbarType::Both
        ) {
            let scrollbar_elem = LayoutElement {
                id: self.next_id("hscrollbar"),
                element_type: ElementType::Rectangle {
                    label: String::new(),
                    corner_radius: 2.0,
                },
                bounds: Rect::new(x, y + h, w, scrollbar_size),
                text: None,
                properties: [
                    ("fill".to_string(), "#E0E0E0".to_string()),
                    ("stroke".to_string(), self.config.border_color.to_string()),
                ]
                .into_iter()
                .collect(),
            };
            elements.push(scrollbar_elem);
        }

        let total_width = w + if matches!(
            scrollbar,
            plantuml_ast::salt::ScrollbarType::Vertical | plantuml_ast::salt::ScrollbarType::Both
        ) {
            scrollbar_size
        } else {
            0.0
        };

        let total_height = h + if matches!(
            scrollbar,
            plantuml_ast::salt::ScrollbarType::Horizontal | plantuml_ast::salt::ScrollbarType::Both
        ) {
            scrollbar_size
        } else {
            0.0
        };

        (total_width, total_height)
    }
}

impl Default for SaltLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine for SaltLayoutEngine {
    type Input = SaltDiagram;

    /// Выполняет layout salt-диаграммы.
    ///
    /// Использует `self.config`: раньше здесь создавался новый движок через
    /// `SaltLayoutEngine::new()`, из-за чего `with_config()` не оказывал
    /// никакого эффекта — публичный API вводил в заблуждение.
    ///
    /// `render_widget` требует `&mut self` (счётчик идентификаторов),
    /// поэтому работаем на локальной копии, унаследовавшей конфигурацию.
    fn layout(&self, diagram: &Self::Input, _config: &LayoutConfig) -> LayoutResult {
        let mut engine = SaltLayoutEngine {
            config: self.config.clone(),
            element_id: self.element_id,
        };
        let mut elements = Vec::new();

        let available_width = engine.config.available_width;
        let (width, height) = engine.render_widget(
            &diagram.root,
            engine.config.padding,
            engine.config.padding,
            available_width,
            &mut elements,
        );

        LayoutResult {
            elements,
            bounds: Rect::new(
                0.0,
                0.0,
                width + engine.config.padding * 2.0,
                height + engine.config.padding * 2.0,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_simple_salt() {
        let mut container = Container::new();
        container.add_row(vec![
            SaltWidget::Text("Login".to_string()),
            SaltWidget::TextField("MyName".to_string()),
        ]);
        container.add_row(vec![
            SaltWidget::Button("Cancel".to_string()),
            SaltWidget::Button("OK".to_string()),
        ]);

        let diagram = SaltDiagram::new().with_root(SaltWidget::Container(container));

        let engine = SaltLayoutEngine::new();
        let config = LayoutConfig::default();
        let result = engine.layout(&diagram, &config);

        assert!(!result.elements.is_empty());
        assert!(result.bounds.width > 0.0);
        assert!(result.bounds.height > 0.0);
    }

    #[test]
    fn test_layout_with_checkbox() {
        let mut container = Container::new();
        container.add_row(vec![SaltWidget::Checkbox {
            label: "Accept terms".to_string(),
            checked: true,
        }]);

        let diagram = SaltDiagram::new().with_root(SaltWidget::Container(container));

        let engine = SaltLayoutEngine::new();
        let config = LayoutConfig::default();
        let result = engine.layout(&diagram, &config);

        assert!(!result.elements.is_empty());
    }
}
