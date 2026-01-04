//! Layout engine для Salt (Wireframe) диаграмм
//!
//! Рендерит UI wireframes с кнопками, текстовыми полями, чекбоксами и т.д.

use plantuml_ast::salt::{BorderStyle, Container, SaltDiagram, SaltWidget, SeparatorType};
use plantuml_model::{Point, Rect};

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

    /// Рендерит контейнер
    fn render_container(
        &mut self,
        container: &Container,
        x: f64,
        y: f64,
        available_width: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> (f64, f64) {
        let mut current_y = y + self.config.cell_padding;
        let mut max_width = 0.0_f64;
        let start_x = x + self.config.cell_padding;

        // Вычисляем ширину каждого столбца
        let max_cols = container.rows.iter().map(|r| r.len()).max().unwrap_or(1);
        let col_width = (available_width - self.config.cell_padding * 2.0) / max_cols as f64;

        for row in &container.rows {
            let mut current_x = start_x;
            let mut row_height = self.config.row_height;

            for (col_idx, widget) in row.iter().enumerate() {
                let cell_width = if col_idx == row.len() - 1 {
                    // Последняя ячейка занимает оставшееся место
                    (max_cols - col_idx) as f64 * col_width
                } else {
                    col_width
                };

                let (_w, h) = self.render_widget(
                    widget,
                    current_x,
                    current_y,
                    cell_width - self.config.cell_padding,
                    elements,
                );
                row_height = row_height.max(h);
                current_x += cell_width;
            }

            max_width = max_width.max(current_x - start_x);
            current_y += row_height + self.config.cell_padding / 2.0;
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
            .chain(
                stroke_dasharray
                    .map(|d| ("stroke-dasharray".to_string(), d.to_string()))
                    .into_iter(),
            )
            .collect(),
        };
        elements.push(border);
    }

    /// Рендерит текст
    fn render_text(&mut self, text: &str, x: f64, y: f64, elements: &mut Vec<LayoutElement>) -> (f64, f64) {
        let width = text.len() as f64 * 8.0 + self.config.cell_padding;
        let height = self.config.row_height;

        let text_elem = LayoutElement {
            id: self.next_id("text"),
            element_type: ElementType::Text {
                text: text.to_string(),
                font_size: self.config.font_size,
            },
            bounds: Rect::new(x, y + 5.0, width, height),
            text: Some(text.to_string()),
            properties: [("fill".to_string(), "#000000".to_string())]
                .into_iter()
                .collect(),
        };
        elements.push(text_elem);

        (width, height)
    }

    /// Рендерит кнопку
    fn render_button(
        &mut self,
        label: &str,
        x: f64,
        y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> (f64, f64) {
        let width = label.len() as f64 * 8.0 + self.config.cell_padding * 2.0;
        let width = width.max(self.config.min_cell_width);
        let height = self.config.button_height;

        // Фон кнопки
        let bg = LayoutElement {
            id: self.next_id("button_bg"),
            element_type: ElementType::Rectangle {
                label: String::new(),
                corner_radius: 3.0,
            },
            bounds: Rect::new(x, y, width, height),
            text: None,
            properties: [
                ("fill".to_string(), self.config.button_color.to_string()),
                ("stroke".to_string(), self.config.border_color.to_string()),
            ]
            .into_iter()
            .collect(),
        };
        elements.push(bg);

        // Текст кнопки
        let text = LayoutElement {
            id: self.next_id("button_text"),
            element_type: ElementType::Text {
                text: label.to_string(),
                font_size: self.config.font_size,
            },
            bounds: Rect::new(x + self.config.cell_padding, y + 4.0, width, height),
            text: Some(label.to_string()),
            properties: [
                ("fill".to_string(), "#000000".to_string()),
                ("text-anchor".to_string(), "middle".to_string()),
            ]
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
        let width = text.len() as f64 * 8.0 + self.config.cell_padding * 2.0;
        let width = width.max(self.config.min_cell_width);
        let height = self.config.textfield_height;

        // Фон поля
        let bg = LayoutElement {
            id: self.next_id("field_bg"),
            element_type: ElementType::Rectangle {
                label: String::new(),
                corner_radius: 2.0,
            },
            bounds: Rect::new(x, y, width, height),
            text: None,
            properties: [
                ("fill".to_string(), self.config.textfield_color.to_string()),
                ("stroke".to_string(), self.config.border_color.to_string()),
            ]
            .into_iter()
            .collect(),
        };
        elements.push(bg);

        // Текст
        let text_elem = LayoutElement {
            id: self.next_id("field_text"),
            element_type: ElementType::Text {
                text: text.to_string(),
                font_size: self.config.font_size,
            },
            bounds: Rect::new(x + 4.0, y + 4.0, width, height),
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
                ("fill".to_string(), "#FFFFFF".to_string()),
                ("stroke".to_string(), self.config.border_color.to_string()),
            ]
            .into_iter()
            .collect(),
        };
        elements.push(box_elem);

        // Галочка если отмечен
        if checked {
            let check = LayoutElement {
                id: self.next_id("check"),
                element_type: ElementType::Text {
                    text: "X".to_string(),
                    font_size: size - 2.0,
                },
                bounds: Rect::new(x + 2.0, y + 4.0, size, size),
                text: Some("X".to_string()),
                properties: [("fill".to_string(), "#000000".to_string())]
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
                bounds: Rect::new(x + size + 4.0, y + 4.0, 100.0, 20.0),
                text: Some(label.to_string()),
                properties: [("fill".to_string(), "#000000".to_string())]
                    .into_iter()
                    .collect(),
            };
            elements.push(label_elem);
            label.len() as f64 * 8.0
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
                bounds: Rect::new(x + size + 4.0, y + 4.0, 100.0, 20.0),
                text: Some(label.to_string()),
                properties: [("fill".to_string(), "#000000".to_string())]
                    .into_iter()
                    .collect(),
            };
            elements.push(label_elem);
            label.len() as f64 * 8.0
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
        let width = text.len() as f64 * 8.0 + 30.0;
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
            SeparatorType::Wavy => Some("4,2"),
            SeparatorType::Single => None,
            SeparatorType::Double => None,
        };

        let line = LayoutElement {
            id: self.next_id("separator"),
            element_type: ElementType::Edge {
                points: vec![
                    Point::new(x, y + 10.0),
                    Point::new(x + width, y + 10.0),
                ],
                label: None,
                arrow_start: false,
                arrow_end: false,
                dashed: dasharray.is_some(),
                edge_type: EdgeType::Link, from_cardinality: None, to_cardinality: None,
            },
            bounds: Rect::new(x, y, width, 20.0),
            text: None,
            properties: [
                ("stroke".to_string(), self.config.border_color.to_string()),
            ]
            .into_iter()
            .chain(
                dasharray
                    .map(|d| ("stroke-dasharray".to_string(), d.to_string()))
                    .into_iter(),
            )
            .collect(),
        };
        elements.push(line);

        // Для двойной линии добавляем вторую
        if sep_type == SeparatorType::Double {
            let line2 = LayoutElement {
                id: self.next_id("separator2"),
                element_type: ElementType::Edge {
                    points: vec![
                        Point::new(x, y + 14.0),
                        Point::new(x + width, y + 14.0),
                    ],
                    label: None,
                    arrow_start: false,
                    arrow_end: false,
                    dashed: false,
                    edge_type: EdgeType::Link, from_cardinality: None, to_cardinality: None,
                },
                bounds: Rect::new(x, y, width, 20.0),
                text: None,
                properties: [
                    ("stroke".to_string(), self.config.border_color.to_string()),
                ]
                .into_iter()
                .collect(),
            };
            elements.push(line2);
        }

        (width, 20.0)
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

        fn render_node(
            engine: &mut SaltLayoutEngine,
            node: &plantuml_ast::salt::TreeNode,
            x: f64,
            y: &mut f64,
            max_width: &mut f64,
            elements: &mut Vec<LayoutElement>,
        ) {
            let indent = node.level as f64 * 20.0;

            if !node.text.is_empty() {
                let prefix = if node.level > 0 { "├─ " } else { "" };
                let text = format!("{}{}", prefix, node.text);
                let width = text.len() as f64 * 8.0 + indent;

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

            for child in &node.children {
                render_node(engine, child, x, y, max_width, elements);
            }
        }

        render_node(self, node, x, &mut current_y, &mut max_width, elements);

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
            let width = item.len() as f64 * 8.0 + 20.0;
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
                        if is_selected {
                            "#FFFFFF"
                        } else {
                            "#E0E0E0"
                        }
                        .to_string(),
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
        let total_width = items.iter().map(|i| i.text.len() as f64 * 8.0 + 20.0).sum::<f64>();
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

            let width = item.text.len() as f64 * 8.0 + 20.0;

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
        let (content_width, content_height) =
            self.render_widget(content, x + 5.0, y + title_height + 5.0, available_width - 10.0, elements);

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
            plantuml_ast::salt::ScrollbarType::Vertical | plantuml_ast::salt::ScrollbarType::Both => {
                available_width - scrollbar_size
            }
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

        let total_width = w + if matches!(scrollbar, plantuml_ast::salt::ScrollbarType::Vertical | plantuml_ast::salt::ScrollbarType::Both) {
            scrollbar_size
        } else {
            0.0
        };

        let total_height = h + if matches!(scrollbar, plantuml_ast::salt::ScrollbarType::Horizontal | plantuml_ast::salt::ScrollbarType::Both) {
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

    fn layout(&self, diagram: &Self::Input, _config: &LayoutConfig) -> LayoutResult {
        let mut engine = SaltLayoutEngine::new();
        let mut elements = Vec::new();

        let (width, height) = engine.render_widget(
            &diagram.root,
            engine.config.padding,
            engine.config.padding,
            800.0, // default available width
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
