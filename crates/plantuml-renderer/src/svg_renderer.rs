//! SVG рендерер

use svg::node::element::{Definitions, Group, Marker, Path, Rectangle};
use svg::Document;

use crate::{
    ClassMember, ClassifierKind, EdgeType, ElementType, FragmentSection, LayoutElement,
    LayoutResult, MemberVisibility, Point, Rect, RenderOptions, Renderer, ZLayer,
};
use plantuml_themes::Theme;

/// Цвет рамки фрагмента (alt/opt/loop) в эталоне PlantUML.
const FRAGMENT_BORDER: &str = "#000";

/// Цвет заливки заголовка фрагмента в эталоне PlantUML.
const FRAGMENT_HEADER_FILL: &str = "#EEE";

/// SVG рендерер
pub struct SvgRenderer {
    options: RenderOptions,
}

impl SvgRenderer {
    /// Создаёт новый рендерер
    pub fn new() -> Self {
        Self {
            options: RenderOptions::default(),
        }
    }

    /// Создаёт рендерер с опциями
    pub fn with_options(options: RenderOptions) -> Self {
        Self { options }
    }

    /// Рендерит в строку
    pub fn render_to_string(&self, layout: &LayoutResult, theme: &Theme) -> String {
        self.render(layout, theme)
    }

    /// Создаёт SVG документ
    /// PlantUML стиль: прозрачный/белый фон БЕЗ рамки вокруг диаграммы
    fn create_document(&self, layout: &LayoutResult, theme: &Theme) -> Document {
        let bounds = &layout.bounds;
        let margin = 5.0; // Минимальный отступ от края (как в PlantUML)

        let width = (bounds.width + margin * 2.0) * self.options.scale;
        let height = (bounds.height + margin * 2.0) * self.options.scale;

        // Набор атрибутов повторяет PlantUML: он важен для потребителей,
        // разбирающих вывод (contentStyleType), и для корректного
        // масштабирования (preserveAspectRatio, zoomAndPan).
        let doc = Document::new()
            .set("xmlns", "http://www.w3.org/2000/svg")
            .set("xmlns:xlink", "http://www.w3.org/1999/xlink")
            .set("version", "1.1")
            .set("width", format!("{:.0}px", width))
            .set("height", format!("{:.0}px", height))
            .set(
                "viewBox",
                (
                    bounds.x - margin,
                    bounds.y - margin,
                    bounds.width + margin * 2.0,
                    bounds.height + margin * 2.0,
                ),
            )
            .set("zoomAndPan", "magnify")
            .set("preserveAspectRatio", "none")
            .set("contentStyleType", "text/css")
            // PlantUML дублирует размеры ещё и в style, а также пишет
            // единицы измерения (`306px`) в width/height. Это важно для
            // потребителей, читающих размеры из style.
            .set(
                "style",
                format!("width:{width:.0}px;height:{height:.0}px;background:#FFFFFF;"),
            );

        // PlantUML помечает корневой `<svg>` типом диаграммы: потребители
        // вывода (редакторы, конвертеры) определяют по нему разметку.
        let mut doc = if let Some(kind) = &self.options.diagram_type {
            doc.set("data-diagram-type", kind.as_str())
        } else {
            doc
        };

        // PlantUML по умолчанию НЕ добавляет фон и рамку вокруг диаграммы
        // Фон добавляется только если явно указан через skinparam backgroundColor
        if let Some(bg) = &self.options.background_color {
            let bg_rect = Rectangle::new()
                .set("x", bounds.x - margin)
                .set("y", bounds.y - margin)
                .set("width", bounds.width + margin * 2.0)
                .set("height", bounds.height + margin * 2.0)
                .set("fill", bg.as_str())
                .set("stroke", "none");
            doc = doc.add(bg_rect);
        }

        // Определения (маркеры стрелок)
        let defs = self.create_definitions(theme);
        doc = doc.add(defs);

        doc
    }

    /// Создаёт определения (маркеры, градиенты)
    /// PlantUML стиль: разные стрелки для разных типов связей
    fn create_definitions(&self, theme: &Theme) -> Definitions {
        let arrow_color = theme.arrow_color.to_css();

        // Маркер стрелки в стиле PlantUML (ромб с вырезом) - для ассоциаций и сообщений
        let arrow_marker = Marker::new()
            .set("id", "arrow")
            .set("markerWidth", 10)
            .set("markerHeight", 8)
            .set("refX", 10)
            .set("refY", 4)
            .set("orient", "auto")
            .set("markerUnits", "userSpaceOnUse")
            .add(
                Path::new()
                    // PlantUML style: ромб с вырезом
                    .set("d", "M0,0 L10,4 L0,8 L4,4 Z")
                    .set("fill", arrow_color.as_str()),
            );

        // Открытый маркер стрелки (для async сообщений)
        let open_arrow_marker = Marker::new()
            .set("id", "arrow-open")
            .set("markerWidth", 10)
            .set("markerHeight", 8)
            .set("refX", 10)
            .set("refY", 4)
            .set("orient", "auto")
            .set("markerUnits", "userSpaceOnUse")
            .add(
                Path::new()
                    .set("d", "M0,0 L10,4 L0,8")
                    .set("fill", "none")
                    .set("stroke", arrow_color.as_str())
                    .set("stroke-width", 1),
            );

        // Маркер наследования (пустой треугольник) - для --|> и ..|>
        // PlantUML использует polygon fill="none" для inheritance
        let inheritance_marker = Marker::new()
            .set("id", "inheritance")
            .set("markerWidth", 20)
            .set("markerHeight", 20)
            .set("refX", 20)
            .set("refY", 10)
            .set("orient", "auto")
            .set("markerUnits", "userSpaceOnUse")
            .add(
                Path::new()
                    // Треугольник: верх, кончик, низ
                    .set("d", "M0,0 L20,10 L0,20 Z")
                    .set("fill", theme.background_color.to_css()) // белый внутри
                    .set("stroke", arrow_color.as_str())
                    .set("stroke-width", 1),
            );

        // Маркер композиции (закрашенный ромб) - для *--
        let composition_marker = Marker::new()
            .set("id", "composition")
            .set("markerWidth", 12)
            .set("markerHeight", 12)
            .set("refX", 0)
            .set("refY", 6)
            .set("orient", "auto")
            .set("markerUnits", "userSpaceOnUse")
            .add(
                Path::new()
                    // Ромб: лево, верх, право, низ
                    .set("d", "M0,6 L6,0 L12,6 L6,12 Z")
                    .set("fill", arrow_color.as_str()),
            );

        // Маркер агрегации (пустой ромб) - для o--
        let aggregation_marker = Marker::new()
            .set("id", "aggregation")
            .set("markerWidth", 12)
            .set("markerHeight", 12)
            .set("refX", 0)
            .set("refY", 6)
            .set("orient", "auto")
            .set("markerUnits", "userSpaceOnUse")
            .add(
                Path::new()
                    .set("d", "M0,6 L6,0 L12,6 L6,12 Z")
                    .set("fill", theme.background_color.to_css()) // белый внутри
                    .set("stroke", arrow_color.as_str())
                    .set("stroke-width", 1),
            );

        Definitions::new()
            .add(arrow_marker)
            .add(open_arrow_marker)
            .add(inheritance_marker)
            .add(composition_marker)
            .add(aggregation_marker)
    }

    /// Рендерит элемент
    fn render_element(&self, element: &LayoutElement, theme: &Theme) -> Group {
        self.render_element_with_id(element, theme, &element.id)
    }

    /// Рендерит элемент с заданным идентификатором.
    ///
    /// Отдельный метод нужен, чтобы `render` мог подставить уникальный id,
    /// не изменяя сам элемент layout.
    fn render_element_with_id(&self, element: &LayoutElement, theme: &Theme, id: &str) -> Group {
        let mut group = Group::new().set("id", id);

        match &element.element_type {
            ElementType::Rectangle {
                label,
                corner_radius,
            } => {
                group = self.render_rectangle(
                    &element.bounds,
                    label,
                    *corner_radius,
                    theme,
                    group,
                    &element.properties,
                );
            }
            ElementType::Ellipse { label } => {
                group = self.render_ellipse(&element.bounds, label.as_deref(), theme, group);
            }
            ElementType::InitialState => {
                group = self.render_initial_state(&element.bounds, theme, group);
            }
            ElementType::FinalState => {
                group = self.render_final_state(&element.bounds, theme, group);
            }
            ElementType::State { name, description } => {
                group = self.render_uml_state(
                    &element.bounds,
                    name,
                    description.as_deref(),
                    theme,
                    group,
                );
            }
            ElementType::CompositeState {
                name,
                header_height,
            } => {
                group = self.render_composite_state(
                    &element.bounds,
                    name,
                    *header_height,
                    theme,
                    group,
                );
            }
            // Фигуры участников sequence. Нижние блоки (`footer_`) PlantUML
            // рисует зеркально: подпись сверху, фигура снизу.
            ElementType::Actor { label } => {
                let footer = id.starts_with("footer_");
                group = self.render_actor(&element.bounds, label, theme, group, footer);
            }
            ElementType::Database { label } => {
                let footer = id.starts_with("footer_");
                group = self.render_database(&element.bounds, label, theme, group, footer);
            }
            ElementType::Boundary { label } => {
                let footer = id.starts_with("footer_");
                group = self.render_boundary(&element.bounds, label, theme, group, footer);
            }
            ElementType::Control { label } => {
                let footer = id.starts_with("footer_");
                group = self.render_control(&element.bounds, label, theme, group, footer);
            }
            ElementType::Entity { label } => {
                let footer = id.starts_with("footer_");
                group = self.render_entity(&element.bounds, label, theme, group, footer);
            }
            ElementType::System { title } => {
                group = self.render_system(&element.bounds, title, theme, group);
            }
            ElementType::Edge {
                points,
                label,
                arrow_start,
                arrow_end,
                dashed,
                edge_type,
                from_cardinality,
                to_cardinality,
            } => {
                let autonumber = element.properties.get("autonumber").map(|s| s.as_str());
                group = self.render_edge(
                    points,
                    label.as_deref(),
                    autonumber,
                    *arrow_start,
                    *arrow_end,
                    *dashed,
                    *edge_type,
                    from_cardinality.as_deref(),
                    to_cardinality.as_deref(),
                    theme,
                    group,
                );
            }
            ElementType::Text { text, font_size } => {
                group = self.render_text(&element.bounds, text, *font_size, theme, group);
            }
            ElementType::Group { label, children } => {
                group =
                    self.render_group(&element.bounds, label.as_deref(), children, theme, group);
            }
            ElementType::Fragment {
                fragment_type,
                sections,
            } => {
                group =
                    self.render_fragment(&element.bounds, fragment_type, sections, theme, group);
            }
            ElementType::Activation => {
                group = self.render_activation(&element.bounds, theme, group);
            }
            ElementType::RoundedRectangle => {
                // Рендерим как прямоугольник со скруглёнными углами.
                // Радиус берётся из темы (`skinparam roundcorner`), а не из
                // литерала: иначе настройка темы игнорировалась.
                let label = element.text.as_deref().unwrap_or("");
                group = self.render_rectangle(
                    &element.bounds,
                    label,
                    theme.corner_radius,
                    theme,
                    group,
                    &element.properties,
                );
            }
            ElementType::Path => {
                // Рендерим SVG path (для кривых Безье)
                if let Some(path_data) = element.properties.get("path") {
                    let path = svg::node::element::Path::new()
                        .set("d", path_data.as_str())
                        .set("fill", "none")
                        .set("stroke", theme.node_border.to_css())
                        .set("stroke-width", 1);
                    group = group.add(path);
                }
            }
            ElementType::ClassBox {
                classifier_type,
                name,
                stereotype,
                fields,
                methods,
            } => {
                group = self.render_class_box(
                    &element.bounds,
                    *classifier_type,
                    name,
                    stereotype.as_deref(),
                    fields,
                    methods,
                    theme,
                    group,
                );
            }
            ElementType::ParticipantBox => {
                // Рендерим box для группировки участников
                let title = element.text.as_deref();
                let color = element.properties.get("color").map(|s| s.as_str());
                group = self.render_participant_box(&element.bounds, title, color, theme, group);
            }
        }

        group
    }

    /// Рендерит прямоугольник
    fn render_rectangle(
        &self,
        bounds: &Rect,
        label: &str,
        corner_radius: f64,
        theme: &Theme,
        mut group: Group,
        properties: &std::collections::HashMap<String, String>,
    ) -> Group {
        // Получаем цвет заливки из properties или используем тему
        let default_fill = theme.node_background.to_css();
        let fill_color = properties
            .get("fill")
            .map(|s| s.as_str())
            .unwrap_or(default_fill.as_str());

        // Получаем прозрачность из properties
        let opacity = properties.get("opacity").map(|s| s.as_str());

        // Радиус скругления может быть задан явно в properties: так делает
        // таблица JSON/YAML (rx = 5). Раньше свойство не читалось, и таблица
        // рисовалась с радиусом темы.
        let corner_radius = properties
            .get("rx")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(corner_radius);

        // Толщина границы берётся из темы (`skinparam linetype`), значение
        // по умолчанию 0.5 совпадает с PlantUML для участников.
        let mut rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("rx", corner_radius)
            .set("ry", corner_radius)
            .set("fill", fill_color)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", theme.line_width * 0.5);

        if let Some(op) = opacity {
            rect = rect.set("fill-opacity", op);
        }

        group = group.add(rect);

        // Тень: `skinparam shadowing true`. PlantUML рисует смещённый
        // прямоугольник под фигурой.
        if theme.shadow {
            group = group.add(
                Rectangle::new()
                    .set("x", bounds.x + SHADOW_OFFSET)
                    .set("y", bounds.y + SHADOW_OFFSET)
                    .set("width", bounds.width)
                    .set("height", bounds.height)
                    .set("rx", corner_radius)
                    .set("ry", corner_radius)
                    .set("fill", "#000000")
                    .set("fill-opacity", 0.2)
                    .set("stroke", "none"),
            );
        }

        // Текст по центру
        let mut text = svg::node::element::Text::new(label)
            .set("x", bounds.x + bounds.width / 2.0)
            .set("y", bounds.y + bounds.height / 2.0)
            .set("text-anchor", "middle")
            .set("dominant-baseline", "middle")
            .set("font-family", theme.font_family.as_str())
            .set("font-size", theme.font_size)
            .set("fill", theme.text_color.to_css());

        // Рукописный стиль: `skinparam handwritten true`. PlantUML рисует
        // текст слегка наклонным.
        if theme.handwritten {
            text = text.set("font-style", "italic");
        }

        group.add(text)
    }

    /// Рендерит эллипс
    fn render_ellipse(
        &self,
        bounds: &Rect,
        label: Option<&str>,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        let cx = bounds.x + bounds.width / 2.0;
        let cy = bounds.y + bounds.height / 2.0;
        let rx = bounds.width / 2.0;
        let ry = bounds.height / 2.0;

        let ellipse = svg::node::element::Ellipse::new()
            .set("cx", cx)
            .set("cy", cy)
            .set("rx", rx)
            .set("ry", ry)
            .set("fill", theme.node_background.to_css())
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1);

        group = group.add(ellipse);

        if let Some(label) = label {
            let text = svg::node::element::Text::new(label)
                .set("x", cx)
                .set("y", cy)
                .set("text-anchor", "middle")
                .set("dominant-baseline", "middle")
                .set("font-family", theme.font_family.as_str())
                .set("font-size", theme.font_size)
                .set("fill", theme.text_color.to_css());

            group = group.add(text);
        }

        group
    }

    /// Рендерит UML Initial State (чёрный заполненный круг)
    fn render_initial_state(&self, bounds: &Rect, theme: &Theme, group: Group) -> Group {
        let cx = bounds.x + bounds.width / 2.0;
        let cy = bounds.y + bounds.height / 2.0;
        let r = bounds.width.min(bounds.height) / 2.0;

        // Заполненный чёрный круг (UML standard)
        let circle = svg::node::element::Ellipse::new()
            .set("cx", cx)
            .set("cy", cy)
            .set("rx", r)
            .set("ry", r)
            .set("fill", theme.node_border.to_css()) // чёрная заливка
            .set("stroke", "none");

        group.add(circle)
    }

    /// Рендерит UML Final State (bullseye: внешний круг + внутренний заполненный круг)
    fn render_final_state(&self, bounds: &Rect, theme: &Theme, mut group: Group) -> Group {
        let cx = bounds.x + bounds.width / 2.0;
        let cy = bounds.y + bounds.height / 2.0;
        let outer_r = bounds.width.min(bounds.height) / 2.0;
        let inner_r = outer_r * 0.6; // внутренний круг 60% от внешнего

        // Внешний круг (пустой, с обводкой)
        let outer_circle = svg::node::element::Ellipse::new()
            .set("cx", cx)
            .set("cy", cy)
            .set("rx", outer_r)
            .set("ry", outer_r)
            .set("fill", theme.background_color.to_css())
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1.5);

        group = group.add(outer_circle);

        // Внутренний круг (заполненный чёрный)
        let inner_circle = svg::node::element::Ellipse::new()
            .set("cx", cx)
            .set("cy", cy)
            .set("rx", inner_r)
            .set("ry", inner_r)
            .set("fill", theme.node_border.to_css()) // чёрная заливка
            .set("stroke", "none");

        group.add(inner_circle)
    }

    /// Рендерит UML State (скруглённый прямоугольник с разделителем и названием)
    fn render_uml_state(
        &self,
        bounds: &Rect,
        name: &str,
        description: Option<&str>,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        // Параметры по эталону PlantUML: rx/ry = 12.5, толщина границы 0.5,
        // разделитель на 26.3px от верха, имя без жирного начертания.
        // Раньше было rx=10, толщина 1, заголовок 25 и жирное имя.
        let corner_radius = 12.5;
        let header_height = STATE_HEADER_HEIGHT;

        // 1. Основной прямоугольник со скруглёнными углами.
        // Заливка в эталоне — #F1F1F1, а не цвет фона узлов темы.
        let rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("rx", corner_radius)
            .set("ry", corner_radius)
            .set("fill", CLASS_BODY_FILL)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 0.5);

        group = group.add(rect);

        // 2. Название состояния (центрировано сверху)
        let name_y = bounds.y + header_height / 2.0 + 5.0;
        let name_text = svg::node::element::Text::new(name)
            .set("x", bounds.x + bounds.width / 2.0)
            .set("y", name_y)
            .set("text-anchor", "middle")
            .set("dominant-baseline", "middle")
            .set("font-family", theme.font_family.as_str())
            .set("font-size", theme.font_size)
            .set("fill", theme.text_color.to_css());

        group = group.add(name_text);

        // 3. Горизонтальный разделитель (UML style)
        let separator_y = bounds.y + header_height;
        let separator = svg::node::element::Line::new()
            .set("x1", bounds.x)
            .set("y1", separator_y)
            .set("x2", bounds.x + bounds.width)
            .set("y2", separator_y)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 0.5);

        group = group.add(separator);

        // 4. Описание (entry/exit/do actions) если есть
        if let Some(desc) = description {
            let desc_y = separator_y + 15.0;
            let desc_text = svg::node::element::Text::new(desc)
                .set("x", bounds.x + 5.0)
                .set("y", desc_y)
                .set("font-family", theme.font_family.as_str())
                .set("font-size", theme.font_size - 2.0)
                .set("fill", theme.text_color.to_css());

            group = group.add(desc_text);
        }

        group
    }

    /// Рендерит UML Composite State (контейнер с заголовком и вложенными состояниями)
    fn render_composite_state(
        &self,
        bounds: &Rect,
        name: &str,
        header_height: f64,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        let corner_radius = 10.0;

        // 1. Основной прямоугольник контейнера со скруглёнными углами
        let rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("rx", corner_radius)
            .set("ry", corner_radius)
            .set("fill", theme.node_background.to_css())
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1.5);

        group = group.add(rect);

        // 2. Заголовок состояния (название сверху, жирным, центрировано)
        let name_y = bounds.y + header_height / 2.0 + 2.0;
        let name_text = svg::node::element::Text::new(name)
            .set("x", bounds.x + bounds.width / 2.0)
            .set("y", name_y)
            .set("text-anchor", "middle")
            .set("dominant-baseline", "middle")
            .set("font-family", theme.font_family.as_str())
            .set("font-size", theme.font_size + 1.0)
            .set("font-weight", "bold")
            .set("fill", theme.text_color.to_css());

        group = group.add(name_text);

        // 3. Горизонтальный разделитель под заголовком
        let separator_y = bounds.y + header_height;
        let separator = svg::node::element::Line::new()
            .set("x1", bounds.x)
            .set("y1", separator_y)
            .set("x2", bounds.x + bounds.width)
            .set("y2", separator_y)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1);

        group = group.add(separator);

        group
    }

    /// Рендерит актёра (stick figure) для UseCase диаграмм
    /// Рисует фигуру участника sequence и его подпись.
    ///
    /// Геометрия снята с эталона PlantUML. Верхние блоки рисуют фигуру НАД
    /// подписью, нижние — зеркально, ПОД ней; подпись всегда прижата к
    /// линии жизни.
    fn render_participant_figure(
        &self,
        bounds: &Rect,
        label: &str,
        theme: &Theme,
        mut group: Group,
        footer: bool,
    ) -> Group {
        let cx = bounds.x + bounds.width / 2.0;
        let top = bounds.y;
        let bottom = bounds.y + bounds.height;

        // Подпись
        let label_y = if footer {
            top + PARTICIPANT_FOOTER_LABEL_BASELINE
        } else {
            bottom - PARTICIPANT_LABEL_BASELINE_GAP
        };
        let text = svg::node::element::Text::new(label)
            .set("x", cx)
            .set("y", label_y)
            .set("text-anchor", "middle")
            .set("font-family", theme.font_family.as_str())
            .set("font-size", theme.font_size)
            .set("fill", theme.text_color.to_css());
        group = group.add(text);

        group
    }

    /// Стик-фигура актёра.
    fn render_actor(
        &self,
        bounds: &Rect,
        label: &str,
        theme: &Theme,
        group: Group,
        footer: bool,
    ) -> Group {
        let cx = bounds.x + bounds.width / 2.0;
        let stroke = theme.node_border.to_css();
        let fill = theme.node_background.to_css();

        // Все размеры сняты с эталона `sequence_participants`.
        let head_radius = ACTOR_HEAD_RADIUS;
        let (head_cy, neck_y, waist_y, arms_y, feet_y) = if footer {
            let base = bounds.y;
            (
                base + ACTOR_FOOTER_HEAD_CY,
                base + ACTOR_FOOTER_NECK,
                base + ACTOR_FOOTER_WAIST,
                base + ACTOR_FOOTER_ARMS,
                base + ACTOR_FOOTER_FEET,
            )
        } else {
            let base = bounds.y;
            (
                base + head_radius,
                base + ACTOR_HEAD_RADIUS * 2.0,
                base + ACTOR_WAIST_OFFSET,
                base + ACTOR_ARMS_OFFSET,
                base + ACTOR_FEET_OFFSET,
            )
        };

        let body = format!(
            "M{cx},{neck_y} L{cx},{waist_y} \
             M{arm_l},{arms_y} L{arm_r},{arms_y} \
             M{cx},{waist_y} L{leg_l},{feet_y} \
             M{cx},{waist_y} L{leg_r},{feet_y}",
            neck_y = fmt(neck_y),
            waist_y = fmt(waist_y),
            arms_y = fmt(arms_y),
            feet_y = fmt(feet_y),
            arm_l = fmt(cx - ACTOR_LIMB_SPREAD),
            arm_r = fmt(cx + ACTOR_LIMB_SPREAD),
            leg_l = fmt(cx - ACTOR_LIMB_SPREAD),
            leg_r = fmt(cx + ACTOR_LIMB_SPREAD),
        );

        let mut group = group;
        group = group.add(
            svg::node::element::Ellipse::new()
                .set("cx", cx)
                .set("cy", head_cy)
                .set("rx", head_radius)
                .set("ry", head_radius)
                .set("fill", fill)
                .set("stroke", stroke.clone())
                .set("stroke-width", 0.5),
        );
        group = group.add(
            svg::node::element::Path::new()
                .set("d", body)
                .set("fill", "none")
                .set("stroke", stroke)
                .set("stroke-width", 0.5),
        );

        self.render_participant_figure(bounds, label, theme, group, footer)
    }

    /// Цилиндр базы данных.
    fn render_database(
        &self,
        bounds: &Rect,
        label: &str,
        theme: &Theme,
        group: Group,
        footer: bool,
    ) -> Group {
        let cx = bounds.x + bounds.width / 2.0;
        let rx = DATABASE_RX;
        let ry = DATABASE_CAP_RY;
        let stroke = theme.node_border.to_css();
        let fill = theme.node_background.to_css();

        let (top_cap, bottom_cap) = if footer {
            // Зеркально: цилиндр висит под полосой подписи
            (
                bounds.y + bounds.height - DATABASE_FOOTER_TOP_CAP,
                bounds.y + bounds.height - DATABASE_FOOTER_BOTTOM_CAP,
            )
        } else {
            (bounds.y + ry, bounds.y + DATABASE_BODY_HEIGHT + ry)
        };

        // Боковины и нижняя крышка
        let body = format!(
            "M{left},{top_cap} L{left},{bottom_cap} \
             C{left},{bottom} {cx},{bottom} {cx},{bottom} \
             C{cx},{bottom} {right},{bottom} {right},{bottom_cap} \
             L{right},{top_cap}",
            left = fmt(cx - rx),
            right = fmt(cx + rx),
            top_cap = fmt(top_cap),
            bottom_cap = fmt(bottom_cap),
            bottom = fmt(bottom_cap + ry),
        );

        let mut group = group;
        group = group.add(
            svg::node::element::Path::new()
                .set("d", body)
                .set("fill", fill)
                .set("stroke", stroke.clone())
                .set("stroke-width", 0.5),
        );
        // Верхняя крышка
        group = group.add(
            svg::node::element::Path::new()
                .set(
                    "d",
                    format!(
                        "M{left},{top_cap} C{left},{top_bulge} {cx},{top_bulge} {cx},{top_bulge} \
                         C{cx},{top_bulge} {right},{top_bulge} {right},{top_cap}",
                        left = fmt(cx - rx),
                        right = fmt(cx + rx),
                        top_cap = fmt(top_cap),
                        top_bulge = fmt(top_cap - ry),
                    ),
                )
                .set("fill", "none")
                .set("stroke", stroke)
                .set("stroke-width", 0.5),
        );

        self.render_participant_figure(bounds, label, theme, group, footer)
    }

    /// Граничный элемент: кружок со скобкой слева.
    fn render_boundary(
        &self,
        bounds: &Rect,
        label: &str,
        theme: &Theme,
        group: Group,
        footer: bool,
    ) -> Group {
        let cx = bounds.x + bounds.width / 2.0;
        let stroke = theme.node_border.to_css();
        let fill = theme.node_background.to_css();

        // Кружок прижат к линии жизни: сверху центр на 12 ниже верха,
        // снизу — на 12 выше низа.
        let circle_cy = if footer {
            bounds.y + bounds.height - PARTICIPANT_ICON_RADIUS
        } else {
            bounds.y + PARTICIPANT_ICON_RADIUS
        };
        let bracket_top = circle_cy - PARTICIPANT_ICON_RADIUS;
        let bracket_bottom = circle_cy + PARTICIPANT_ICON_RADIUS;

        let mut group = group;
        group = group.add(
            svg::node::element::Path::new()
                .set(
                    "d",
                    format!(
                        "M{bar},{top} L{bar},{bottom} M{bar},{mid} L{left_of_circle},{mid}",
                        bar = fmt(cx - BOUNDARY_BRACKET_OFFSET),
                        top = fmt(bracket_top),
                        bottom = fmt(bracket_bottom),
                        mid = fmt(circle_cy),
                        left_of_circle = fmt(cx - PARTICIPANT_ICON_RADIUS),
                    ),
                )
                .set("fill", "none")
                .set("stroke", stroke.clone())
                .set("stroke-width", 0.5),
        );
        group = group.add(
            svg::node::element::Ellipse::new()
                .set("cx", cx)
                .set("cy", circle_cy)
                .set("rx", PARTICIPANT_ICON_RADIUS)
                .set("ry", PARTICIPANT_ICON_RADIUS)
                .set("fill", fill)
                .set("stroke", stroke)
                .set("stroke-width", 0.5),
        );

        self.render_participant_figure(bounds, label, theme, group, footer)
    }

    /// Управляющий элемент: кружок со стрелкой сверху.
    fn render_control(
        &self,
        bounds: &Rect,
        label: &str,
        theme: &Theme,
        group: Group,
        footer: bool,
    ) -> Group {
        let cx = bounds.x + bounds.width / 2.0;
        let stroke = theme.node_border.to_css();
        let fill = theme.node_background.to_css();

        let circle_cy = if footer {
            bounds.y + bounds.height - PARTICIPANT_ICON_RADIUS
        } else {
            bounds.y + CONTROL_CIRCLE_OFFSET
        };
        // Стрелка прижата к внешнему краю фигуры
        let arrow_mid = if footer {
            bounds.y + bounds.height - CONTROL_FOOTER_ARROW_MID
        } else {
            bounds.y + CONTROL_ARROW_MID
        };

        let chevron = format!(
            "{x1},{y_mid} {x2},{y_top} {cx},{y_mid} {x2},{y_bot} {x1},{y_mid}",
            x1 = fmt(cx - CONTROL_ARROW_BACK),
            x2 = fmt(cx + CONTROL_ARROW_TIP),
            cx = fmt(cx),
            y_mid = fmt(arrow_mid),
            y_top = fmt(arrow_mid - CONTROL_ARROW_HALF),
            y_bot = fmt(arrow_mid + CONTROL_ARROW_HALF),
        );

        let mut group = group;
        group = group.add(
            svg::node::element::Ellipse::new()
                .set("cx", cx)
                .set("cy", circle_cy)
                .set("rx", PARTICIPANT_ICON_RADIUS)
                .set("ry", PARTICIPANT_ICON_RADIUS)
                .set("fill", fill)
                .set("stroke", stroke.clone())
                .set("stroke-width", 0.5),
        );
        group = group.add(
            svg::node::element::Polygon::new()
                .set("points", chevron)
                .set("fill", stroke.clone())
                .set("stroke", stroke)
                .set("stroke-width", 1.0),
        );

        self.render_participant_figure(bounds, label, theme, group, footer)
    }

    /// Сущность: кружок с подчёркиванием.
    fn render_entity(
        &self,
        bounds: &Rect,
        label: &str,
        theme: &Theme,
        group: Group,
        footer: bool,
    ) -> Group {
        let cx = bounds.x + bounds.width / 2.0;
        let stroke = theme.node_border.to_css();
        let fill = theme.node_background.to_css();

        let circle_cy = if footer {
            bounds.y + bounds.height - PARTICIPANT_ICON_RADIUS - ENTITY_FOOTER_GAP
        } else {
            bounds.y + PARTICIPANT_ICON_RADIUS
        };
        let underline_y = if footer {
            bounds.y + bounds.height
        } else {
            bounds.y + ENTITY_UNDERLINE_OFFSET
        };

        let mut group = group;
        group = group.add(
            svg::node::element::Ellipse::new()
                .set("cx", cx)
                .set("cy", circle_cy)
                .set("rx", PARTICIPANT_ICON_RADIUS)
                .set("ry", PARTICIPANT_ICON_RADIUS)
                .set("fill", fill)
                .set("stroke", stroke.clone())
                .set("stroke-width", 0.5),
        );
        group = group.add(
            svg::node::element::Line::new()
                .set("x1", cx - PARTICIPANT_ICON_RADIUS)
                .set("y1", underline_y)
                .set("x2", cx + PARTICIPANT_ICON_RADIUS)
                .set("y2", underline_y)
                .set("stroke", stroke)
                .set("stroke-width", 0.5),
        );

        self.render_participant_figure(bounds, label, theme, group, footer)
    }

    fn render_system(&self, bounds: &Rect, title: &str, theme: &Theme, mut group: Group) -> Group {
        let header_height = 25.0;

        // 1. Основной прямоугольник системы
        let rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("fill", theme.node_background.to_css())
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1);
        group = group.add(rect);

        // 2. Заголовок сверху по центру
        let title_text = svg::node::element::Text::new(title)
            .set("x", bounds.x + bounds.width / 2.0)
            .set("y", bounds.y + header_height / 2.0 + 5.0)
            .set("text-anchor", "middle")
            .set("font-family", theme.font_family.as_str())
            .set("font-size", theme.font_size + 1.0)
            .set("font-weight", "bold")
            .set("fill", theme.text_color.to_css());
        group = group.add(title_text);

        group
    }

    /// Рендерит линию/стрелку
    #[allow(clippy::too_many_arguments)]
    fn render_edge(
        &self,
        points: &[Point],
        label: Option<&str>,
        autonumber: Option<&str>,
        arrow_start: bool,
        arrow_end: bool,
        dashed: bool,
        edge_type: EdgeType,
        from_cardinality: Option<&str>,
        to_cardinality: Option<&str>,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        if points.len() < 2 {
            return group;
        }

        // Определяем, является ли это self-message (петля: 4 точки, первая и последняя
        // имеют одинаковый X но разный Y - прямоугольная петля вправо)
        let is_self_message = points.len() == 4
            && (points[0].x - points[3].x).abs() < 1.0
            && (points[0].y - points[3].y).abs() > 1.0;

        // Строим путь
        let d = if is_self_message {
            // PlantUML style self-message: прямые углы (3 линии)
            // points[0] = start (lifeline, top)
            // points[1] = right top
            // points[2] = right bottom
            // points[3] = end (lifeline, bottom)
            //
            // PlantUML SVG:
            // line 1: x1=28.8 → x2=70.8, y=67.4 (горизонтальная вправо)
            // line 2: x=70.8, y1=67.4 → y2=80.4 (вертикальная вниз)
            // line 3: x1=70.8 → x2=29.8, y=80.4 (горизонтальная влево)
            // polygon (стрелка): в конце line 3
            //
            // Путь: от lifeline вправо, вниз, обратно к lifeline
            format!(
                "M{},{} L{},{} L{},{} L{},{}",
                points[0].x,
                points[0].y, // начало (lifeline, верх)
                points[1].x,
                points[1].y, // вправо (верхний правый угол)
                points[2].x,
                points[2].y, // вниз (нижний правый угол)
                points[3].x,
                points[3].y, // влево к lifeline (конец)
            )
        } else {
            // Обычные линии
            let mut d = format!("M{},{}", points[0].x, points[0].y);
            for p in &points[1..] {
                d.push_str(&format!(" L{},{}", p.x, p.y));
            }
            d
        };

        // PlantUML использует stroke-width: 0.5 для lifelines, 1 для сообщений
        // Определяем по наличию стрелки - если есть стрелка, это сообщение
        let stroke_width = if arrow_end || arrow_start { 1.0 } else { 0.5 };

        let mut path = Path::new()
            .set("d", d)
            .set("fill", "none")
            .set("stroke", theme.arrow_color.to_css())
            .set("stroke-width", stroke_width);

        // Пунктирная линия для lifelines и dashed arrows
        // PlantUML использует stroke-dasharray: 5,5 для lifelines, 2,2 для dashed сообщений
        if dashed {
            // Для lifelines (без стрелок) используем 5,5, для dashed сообщений - 2,2
            let dash_pattern = if arrow_end || arrow_start {
                "2,2"
            } else {
                "5,5"
            };
            path = path.set("stroke-dasharray", dash_pattern);
        }

        // Выбираем маркер на основе типа связи
        if arrow_end {
            let marker = match edge_type {
                EdgeType::Inheritance | EdgeType::Realization => "url(#inheritance)",
                EdgeType::Composition => "url(#arrow)", // composition marker на start
                EdgeType::Aggregation => "url(#arrow)", // aggregation marker на start
                EdgeType::Dependency => "url(#arrow-open)",
                EdgeType::Association => "url(#arrow)",
                EdgeType::Link => "", // без маркера
            };
            if !marker.is_empty() {
                path = path.set("marker-end", marker);
            }
        }
        if arrow_start {
            let marker = match edge_type {
                EdgeType::Composition => "url(#composition)",
                EdgeType::Aggregation => "url(#aggregation)",
                _ => "url(#arrow)",
            };
            path = path.set("marker-start", marker);
        }

        group = group.add(path);

        // Метка сообщения (в стиле PlantUML: текст рядом с линией)
        // По умолчанию в PlantUML: skinparam sequenceMessageAlign left
        // Если есть autonumber — рендерим его отдельно слева, текст справа от него
        if label.is_some() || autonumber.is_some() {
            // Определяем тип линии
            let dx = points[points.len() - 1].x - points[0].x;
            let dy = points[points.len() - 1].y - points[0].y;

            let is_vertical = points.len() == 2 && dy.abs() > dx.abs() * 3.0;
            let is_horizontal = points.len() == 2 && dx.abs() > dy.abs() * 3.0;
            let is_diagonal = points.len() == 2 && !is_vertical && !is_horizontal;

            // Позиция текста зависит от типа линии
            let (base_x, text_y, anchor) = if is_self_message {
                // PlantUML: для self-message текст НАД верхней линией петли
                let text_start = points[0].x + 5.0;
                let top_y = points[0].y - 5.0;
                (text_start, top_y, "start")
            } else if points.len() == 4 {
                // Ортогональный путь (4 точки): это обратный переход
                // Структура: start_h -> corner1 -> corner2 -> end_h
                // points[0]: начало горизонтального сегмента
                // points[1]: угол (конец первого горизонтального сегмента)
                // points[2]: угол (начало последнего горизонтального сегмента)
                // points[3]: конец горизонтального сегмента

                // Метка располагается на ПЕРВОМ горизонтальном сегменте (исходящем),
                // справа от точки выхода, на уровне Y этого сегмента
                let text_x = points[0].x + 5.0; // справа от точки выхода
                let text_y = points[0].y - 5.0; // чуть выше линии
                (text_x, text_y, "start")
            } else if is_diagonal {
                // Диагональная линия (state diagrams): текст РЯДОМ с линией
                // PlantUML style: текст размещается вдоль стрелки, с внешней стороны

                // Позиция вдоль линии (40% от начала для лучшего разделения расходящихся стрелок)
                let t = 0.40;
                let text_x = points[0].x + dx * t;
                let text_y = points[0].y + dy * t;

                // Смещаем текст в сторону от линии (перпендикулярно, на ВНЕШНЮЮ сторону)
                // Для расходящихся из одной точки стрелок:
                // - Линия влево-вниз — текст СЛЕВА от линии
                // - Линия вправо-вниз — текст СПРАВА от линии
                let offset = 8.0; // отступ от линии
                if dx > 0.0 {
                    // Линия идёт вправо-вниз — текст СПРАВА от линии (внешняя сторона)
                    (text_x + offset, text_y, "start")
                } else {
                    // Линия идёт влево-вниз — текст СЛЕВА от линии (внешняя сторона)
                    (text_x - offset, text_y, "end")
                }
            } else if is_vertical {
                // Вертикальная линия: метка СПРАВА, ПОСЕРЕДИНЕ по высоте
                let mid_y = (points[0].y + points[1].y) / 2.0;
                let text_x = points[0].x + 5.0;
                (text_x, mid_y, "start")
            } else if is_horizontal {
                // Горизонтальная стрелка (sequence diagrams)
                let is_left_to_right = dx > 0.0;
                let left_x = if is_left_to_right {
                    points[0].x + 5.0
                } else {
                    points[1].x + 5.0
                };
                let top_y = points[0].y - 5.0;
                (left_x, top_y, "start")
            } else {
                // Fallback: середина первого сегмента
                let mid_x = (points[0].x + points[1].x) / 2.0;
                let mid_y = (points[0].y + points[1].y) / 2.0;
                (mid_x, mid_y - 5.0, "middle")
            };

            // PlantUML не использует белый фон для текста — текст просто над стрелкой
            // PlantUML использует font-size 13 для сообщений
            let font_size = 13.0;

            // Вычисляем позицию для текста (учитывая autonumber)
            let text_x = if let Some(num_text) = autonumber {
                // Рендерим autonumber отдельно
                // Номер уже отформатирован (например "[01]" или "1." в зависимости от формата)

                // Autonumber слева
                let autonumber_element = svg::node::element::Text::new(num_text)
                    .set("x", base_x)
                    .set("y", text_y)
                    .set("text-anchor", anchor)
                    .set("dominant-baseline", "auto")
                    .set("font-family", theme.font_family.as_str())
                    .set("font-size", font_size)
                    .set("fill", theme.text_color.to_css());
                group = group.add(autonumber_element);

                // Вычисляем ширину autonumber для смещения текста
                // PlantUML: ~7px на символ + небольшой отступ 3px
                let num_width = num_text.len() as f64 * 7.0 + 3.0;
                base_x + num_width
            } else {
                base_x
            };

            // Рендерим текст сообщения (если есть)
            if let Some(label) = label {
                // Поддержка многострочного текста через \n
                group = self
                    .render_multiline_text(text_x, text_y, label, anchor, font_size, theme, group);
            }
        }

        // Рендерим кардинальности (для class diagrams)
        // PlantUML: кардинальности располагаются СЛЕВА от вертикальной линии,
        // близко к точкам соединения с классами
        if points.len() >= 2 {
            let font_size = 13.0; // PlantUML использует 13px
            let horizontal_offset = 10.0; // отступ слева от линии
            let vertical_offset = 12.0; // отступ от точки соединения вниз/вверх

            // Определяем, это вертикальная линия
            let is_vertical = (points[1].y - points[0].y).abs() > (points[1].x - points[0].x).abs();

            // Кардинальность у начальной точки (from)
            if let Some(card) = from_cardinality {
                let p = &points[0];
                let (text_x, text_y) = if is_vertical {
                    // Вертикальная линия: текст СЛЕВА, чуть НИЖЕ точки соединения
                    (p.x - horizontal_offset, p.y + vertical_offset)
                } else {
                    // Горизонтальная линия: текст сверху
                    (p.x + vertical_offset, p.y - horizontal_offset / 2.0)
                };
                let text_elem = svg::node::element::Text::new(card)
                    .set("x", text_x)
                    .set("y", text_y)
                    .set("text-anchor", "end") // выравнивание по правому краю (к линии)
                    .set("dominant-baseline", "middle")
                    .set("font-family", theme.font_family.as_str())
                    .set("font-size", font_size)
                    .set("fill", theme.text_color.to_css());
                group = group.add(text_elem);
            }

            // Кардинальность у конечной точки (to)
            if let Some(card) = to_cardinality {
                let p = &points[points.len() - 1];
                let (text_x, text_y) = if is_vertical {
                    // Вертикальная линия: текст СЛЕВА, чуть ВЫШЕ точки соединения
                    (p.x - horizontal_offset, p.y - vertical_offset)
                } else {
                    // Горизонтальная линия: текст сверху
                    (p.x - vertical_offset, p.y - horizontal_offset / 2.0)
                };
                let text_elem = svg::node::element::Text::new(card)
                    .set("x", text_x)
                    .set("y", text_y)
                    .set("text-anchor", "end") // выравнивание по правому краю (к линии)
                    .set("dominant-baseline", "middle")
                    .set("font-family", theme.font_family.as_str())
                    .set("font-size", font_size)
                    .set("fill", theme.text_color.to_css());
                group = group.add(text_elem);
            }
        }

        group
    }

    /// Рендерит текст
    fn render_text(
        &self,
        bounds: &Rect,
        text_content: &str,
        font_size: f64,
        theme: &Theme,
        group: Group,
    ) -> Group {
        let text = svg::node::element::Text::new(text_content)
            .set("x", bounds.x)
            .set("y", bounds.y + font_size)
            .set("font-family", theme.font_family.as_str())
            .set("font-size", font_size)
            .set("fill", theme.text_color.to_css());

        group.add(text)
    }

    /// Рендерит многострочный текст с поддержкой \n
    /// PlantUML использует <tspan> для каждой строки с dy для смещения
    #[allow(clippy::too_many_arguments)]
    fn render_multiline_text(
        &self,
        x: f64,
        y: f64,
        label: &str,
        anchor: &str,
        font_size: f64,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        // Конвертируем escape-последовательность \n в реальные переносы строк
        let processed_label = label.replace("\\n", "\n");
        let lines: Vec<&str> = processed_label.split('\n').collect();

        if lines.len() == 1 {
            // Одна строка — простой текст
            let text = svg::node::element::Text::new(label)
                .set("x", x)
                .set("y", y)
                .set("text-anchor", anchor)
                .set("dominant-baseline", "auto")
                .set("font-family", theme.font_family.as_str())
                .set("font-size", font_size)
                .set("fill", theme.text_color.to_css());
            group = group.add(text);
        } else {
            // Многострочный текст — используем <text> с <tspan> для каждой строки
            // PlantUML: последняя строка на y (ближе к стрелке), предыдущие строки ВВЕРХ
            // Так текст располагается над стрелкой, и не наезжает на неё
            let line_height = font_size + 2.0;

            // Начинаем с верхней строки (которая будет самой верхней визуально)
            let top_y = y - (lines.len() as f64 - 1.0) * line_height;

            let mut text_element = svg::node::element::Text::new("")
                .set("x", x)
                .set("text-anchor", anchor)
                .set("dominant-baseline", "auto")
                .set("font-family", theme.font_family.as_str())
                .set("font-size", font_size)
                .set("fill", theme.text_color.to_css());

            for (i, line) in lines.iter().enumerate() {
                let tspan = svg::node::element::TSpan::new(*line)
                    .set("x", x)
                    .set("y", top_y + (i as f64) * line_height);
                text_element = text_element.add(tspan);
            }

            group = group.add(text_element);
        }

        group
    }

    /// Рендерит группу (устаревший, для совместимости)
    /// Рисует контейнер (узел deployment, пакет component) объёмной рамкой.
    ///
    /// PlantUML рисует узлы deployment трёхмерными: основной прямоугольник
    /// плюс скошенный верхний правый угол со смещением 10px. Раньше здесь
    /// был плоский прямоугольник с полосой заголовка.
    fn render_group(
        &self,
        bounds: &Rect,
        label: Option<&str>,
        children: &[LayoutElement],
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        let offset = NODE_3D_OFFSET;
        let stroke = theme.node_border.to_css();

        // Основной прямоугольник смещён вниз на величину скоса
        let left = bounds.x;
        let top = bounds.y + offset;
        let right = bounds.x + bounds.width - offset;
        let bottom = bounds.y + bounds.height;

        // Контур объёмной фигуры. Порядок точек снят с эталона
        // deployment_basic: (left,top) -> (left+offset,y) -> (right+offset,y)
        // -> (right+offset,bottom-offset) -> (right,bottom) -> (left,bottom).
        let polygon = format!(
            "{l},{t} {lo},{y} {ro},{y} {ro},{bo} {r},{b} {l},{b} {l},{t}",
            l = fmt(left),
            t = fmt(top),
            lo = fmt(left + offset),
            ro = fmt(right + offset),
            y = fmt(bounds.y),
            bo = fmt(bottom - offset),
            r = fmt(right),
            b = fmt(bottom),
        );

        group = group.add(
            svg::node::element::Polygon::new()
                .set("points", polygon)
                .set("fill", "none")
                .set("stroke", stroke.clone())
                .set("stroke-width", 1)
                .set("stroke-linejoin", "miter"),
        );

        // Внутренние рёбра объёма
        let edges = format!(
            "M{r},{t} L{ro},{y} M{l},{t} L{r},{t} M{r},{t} L{r},{b}",
            l = fmt(left),
            r = fmt(right),
            ro = fmt(right + offset),
            t = fmt(top),
            y = fmt(bounds.y),
            b = fmt(bottom),
        );
        group = group.add(
            svg::node::element::Path::new()
                .set("d", edges)
                .set("fill", "none")
                .set("stroke", stroke)
                .set("stroke-width", 1),
        );

        // Заголовок по центру основного прямоугольника
        if let Some(label) = label {
            let text = svg::node::element::Text::new(label)
                .set("x", (left + right) / 2.0)
                .set("y", top + NODE_TITLE_BASELINE)
                .set("text-anchor", "middle")
                .set("font-family", theme.font_family.as_str())
                .set("font-size", theme.font_size)
                .set("font-weight", "bold")
                .set("fill", theme.text_color.to_css());

            group = group.add(text);
        }

        // Дочерние элементы
        for child in children {
            group = group.add(self.render_element(child, theme));
        }

        group
    }

    /// Рендерит Combined Fragment (alt, opt, loop, etc.) в стиле PlantUML
    fn render_fragment(
        &self,
        bounds: &Rect,
        fragment_type: &str,
        sections: &[FragmentSection],
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        // 1. СПЛОШНАЯ рамка фрагмента (как в PlantUML)
        // PlantUML использует более толстую рамку для фрагментов (1.5px)
        // В эталоне PlantUML рамка фрагмента чёрная (#000), а не цвет
        // границ темы (#181818)
        let rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("fill", "none")
            .set("stroke", FRAGMENT_BORDER)
            .set("stroke-width", 1.5);

        group = group.add(rect);

        // 2. Пятиугольный заголовок (pentagon) в левом верхнем углу.
        // Геометрия по эталону: высота 17.133, зазубрина 10.
        let label_text = fragment_type;
        let label_width = {
            // Считаем символы, а не байты: тип фрагмента всегда латиница,
            // но правило общее. Оценка ширины текста — 0.481em, как в layout
            let measured = label_text.chars().count() as f64 * theme.font_size * 0.481;
            // В эталоне заголовок «alt» занимает 64.44px при тексте 19.44px,
            // то есть отступы по ~22px с каждой стороны
            (measured + 45.0).max(40.0)
        };
        let label_height = 17.133;
        let notch_size = 10.0; // размер «зазубрины» пятиугольника

        // Пятиугольник: верхний левый угол рамки -> вправо -> вниз с зазубриной -> влево -> вверх
        let pentagon_path = format!(
            "M{},{} L{},{} L{},{} L{},{} L{},{} Z",
            bounds.x,
            bounds.y, // верхний левый
            bounds.x + label_width,
            bounds.y, // верхний правый
            bounds.x + label_width,
            bounds.y + label_height - notch_size, // правый до зазубрины
            bounds.x + label_width - notch_size,
            bounds.y + label_height, // зазубрина
            bounds.x,
            bounds.y + label_height, // нижний левый
        );

        // Заливка заголовка в эталоне — светло-серая #EEE
        let pentagon = Path::new()
            .set("d", pentagon_path)
            .set("fill", FRAGMENT_HEADER_FILL)
            .set("stroke", FRAGMENT_BORDER)
            .set("stroke-width", 1.5);

        group = group.add(pentagon);

        // Текст типа фрагмента ("alt", "opt", etc.)
        let type_text = svg::node::element::Text::new(label_text)
            .set("x", bounds.x + 5.0)
            .set("y", bounds.y + 14.0)
            .set("font-family", theme.font_family.as_str())
            .set("font-size", theme.font_size)
            .set("font-weight", "bold")
            .set("fill", theme.text_color.to_css());

        group = group.add(type_text);

        // 3. Условие первой секции справа от пятиугольника
        if let Some(first_section) = sections.first() {
            if let Some(condition) = &first_section.condition {
                let cond_text = svg::node::element::Text::new(format!("[{}]", condition))
                    .set("x", bounds.x + label_width + 10.0)
                    .set("y", bounds.y + 14.0)
                    .set("font-family", theme.font_family.as_str())
                    .set("font-size", theme.font_size)
                    .set("fill", theme.text_color.to_css());

                group = group.add(cond_text);
            }
        }

        // 4. Разделители между секциями (пунктирные линии с условиями else)
        for (i, section) in sections.iter().enumerate() {
            // Разделитель перед секцией (кроме первой)
            if i > 0 {
                // Линия разделителя находится между секциями
                // section.start_y — это Y позиция первого сообщения в секции
                // Нам нужно разместить линию ВЫШЕ этого сообщения с учётом:
                // 1. Места для текста условия [else] над линией (~18px)
                // 2. Отступа от линии до текста сообщения под ней (~10px)
                // Итого: линия на section.start_y - 28px
                let separator_y = section.start_y - 28.0;

                // Пунктирная линия
                let separator_line = Path::new()
                    .set(
                        "d",
                        format!(
                            "M{},{} L{},{}",
                            bounds.x,
                            separator_y,
                            bounds.x + bounds.width,
                            separator_y
                        ),
                    )
                    .set("fill", "none")
                    .set("stroke", theme.node_border.to_css())
                    .set("stroke-width", 1)
                    // В эталоне PlantUML разделитель else — пунктир 2,2,
                    // а не 5,3
                    .set("stroke-dasharray", "2,2");

                group = group.add(separator_line);

                // Текст условия else слева, ПОД линией.
                // В эталоне PlantUML линия на y=139.695, текст на y=151.906,
                // то есть на ~12px ниже. Раньше текст рисовался над линией.
                let else_label = if let Some(cond) = &section.condition {
                    format!("[{cond}]")
                } else {
                    "[else]".to_string()
                };

                let else_text = svg::node::element::Text::new(else_label)
                    .set("x", bounds.x + 5.0)
                    .set("y", separator_y + 12.0)
                    .set("font-family", theme.font_family.as_str())
                    // В эталоне размер 11 и жирный
                    .set("font-size", theme.font_size - 2.0)
                    .set("font-weight", "bold")
                    .set("fill", theme.text_color.to_css());

                group = group.add(else_text);
            }

            // Дочерние элементы секции
            for child in &section.children {
                group = group.add(self.render_element(child, theme));
            }
        }

        group
    }

    /// Рендерит Participant Box (фоновый прямоугольник с заголовком)
    fn render_participant_box(
        &self,
        bounds: &Rect,
        title: Option<&str>,
        color: Option<&str>,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        // Фоновый цвет (по умолчанию светло-серый)
        let fill_color = color.unwrap_or("#EEEEEE");

        // Основной прямоугольник
        let rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("fill", fill_color)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1);
        group = group.add(rect);

        // Заголовок по центру сверху
        if let Some(title) = title {
            let title_y = bounds.y + 16.0;
            let title_text = svg::node::element::Text::new(title)
                .set("x", bounds.x + bounds.width / 2.0)
                .set("y", title_y)
                .set("text-anchor", "middle")
                .set("font-family", theme.font_family.as_str())
                .set("font-size", theme.font_size + 1.0)
                .set("font-weight", "bold")
                .set("fill", theme.text_color.to_css());
            group = group.add(title_text);
        }

        group
    }

    /// Рендерит Activation box (белый фон, чёрная рамка)
    fn render_activation(&self, bounds: &Rect, theme: &Theme, group: Group) -> Group {
        // Activation box: белый фон (как в PlantUML)
        let rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("fill", theme.background_color.to_css()) // белый фон
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1);

        group.add(rect)
    }

    /// Рендерит ClassBox (класс/интерфейс/enum) в стиле PlantUML
    #[allow(clippy::too_many_arguments)]
    fn render_class_box(
        &self,
        bounds: &Rect,
        classifier_type: ClassifierKind,
        name: &str,
        stereotype: Option<&str>,
        fields: &[ClassMember],
        methods: &[ClassMember],
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        let padding = 5.0;
        let line_height = 16.0;
        let icon_size = 11.0; // радиус иконки класса

        // 1. Рамка класса
        // Иконка классификатора. Цвет зависит от типа, но заливается только
        // сам кружок: тело класса в эталоне PlantUML всегда #F1F1F1
        // (светло-серый), независимо от типа.
        let (icon_fill, icon_letter) = match classifier_type {
            ClassifierKind::Class => ("#ADD1B2", "C"),     // зелёный
            ClassifierKind::Interface => ("#B4A7E5", "I"), // фиолетовый
            ClassifierKind::AbstractClass => ("#A9DCDF", "A"), // голубой
            ClassifierKind::Enum => ("#EB937F", "E"),      // оранжевый
            ClassifierKind::Annotation => ("#FFDD8C", "@"), // жёлтый
            ClassifierKind::Entity => ("#CCCCCC", "E"),    // серый
        };

        // Тело класса: в эталоне PlantUML — #F1F1F1, а не цвет фона темы
        // (#E2E2F0) и не цвет иконки
        let rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("rx", 2.5)
            .set("ry", 2.5)
            .set("fill", CLASS_BODY_FILL)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 0.5);
        group = group.add(rect);

        let mut current_y = bounds.y + padding;

        // 2. Иконка классификатора (PlantUML style)
        let icon_x = bounds.x + padding + icon_size;
        let icon_y = current_y + icon_size;

        // Круг иконки
        let icon_circle = svg::node::element::Ellipse::new()
            .set("cx", icon_x)
            .set("cy", icon_y)
            .set("rx", icon_size)
            .set("ry", icon_size)
            .set("fill", icon_fill)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1);
        group = group.add(icon_circle);

        // Буква в иконке
        let icon_text = svg::node::element::Text::new(icon_letter)
            .set("x", icon_x)
            .set("y", icon_y + 4.0)
            .set("text-anchor", "middle")
            .set("font-family", theme.font_family.as_str())
            .set("font-size", 12)
            .set("font-weight", "bold")
            .set("fill", "#000000");
        group = group.add(icon_text);

        // 3. Стереотип (если есть)
        let name_x = icon_x + icon_size + 5.0;
        if let Some(stereo) = stereotype {
            let stereo_text = svg::node::element::Text::new(format!("«{}»", stereo))
                .set("x", name_x)
                .set("y", current_y + 10.0)
                .set("font-family", theme.font_family.as_str())
                .set("font-size", 10)
                .set("fill", theme.text_color.to_css());
            group = group.add(stereo_text);
            current_y += 12.0;
        }

        // 4. Название класса.
        // В эталоне PlantUML имя класса не жирное (font-size 14, обычное
        // начертание) — жирный был лишним.
        let name_text = svg::node::element::Text::new(name)
            .set("x", name_x)
            .set("y", current_y + line_height - 2.0)
            .set("font-family", theme.font_family.as_str())
            .set("font-size", theme.font_size)
            .set("fill", theme.text_color.to_css());
        group = group.add(name_text);
        current_y += line_height + padding;

        // 5. Разделитель после имени
        let separator1 = svg::node::element::Line::new()
            .set("x1", bounds.x + 1.0)
            .set("y1", current_y)
            .set("x2", bounds.x + bounds.width - 1.0)
            .set("y2", current_y)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 0.5);
        group = group.add(separator1);
        current_y += padding;

        // 6. Поля
        for field in fields {
            group = self.render_class_member(
                bounds.x + padding,
                current_y,
                bounds.width - padding * 2.0,
                field,
                theme,
                group,
            );
            current_y += line_height;
        }

        // 7. Разделитель между полями и методами
        let separator2 = svg::node::element::Line::new()
            .set("x1", bounds.x + 1.0)
            .set("y1", current_y)
            .set("x2", bounds.x + bounds.width - 1.0)
            .set("y2", current_y)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 0.5);
        group = group.add(separator2);
        current_y += padding;

        // 8. Методы
        for method in methods {
            group = self.render_class_member(
                bounds.x + padding,
                current_y,
                bounds.width - padding * 2.0,
                method,
                theme,
                group,
            );
            current_y += line_height;
        }

        group
    }

    /// Рендерит член класса (поле или метод) с иконкой видимости
    fn render_class_member(
        &self,
        x: f64,
        y: f64,
        _width: f64,
        member: &ClassMember,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        let icon_radius = 3.0;
        let icon_x = x + icon_radius;
        let icon_y = y + 8.0;

        // Иконка видимости (цветной кружок)
        let (fill_color, stroke_color) = match member.visibility {
            MemberVisibility::Public => ("#84BE84", "#038048"), // зелёный
            MemberVisibility::Private => ("#C82829", "#C80000"), // красный
            MemberVisibility::Protected => ("#FFCC00", "#B38600"), // жёлтый
            MemberVisibility::Package => ("#66CCFF", "#0099CC"), // голубой
        };

        let icon = svg::node::element::Ellipse::new()
            .set("cx", icon_x)
            .set("cy", icon_y)
            .set("rx", icon_radius)
            .set("ry", icon_radius)
            .set("fill", fill_color)
            .set("stroke", stroke_color)
            .set("stroke-width", 1);
        group = group.add(icon);

        // Текст члена
        let text_x = icon_x + icon_radius + 5.0;
        let mut text = svg::node::element::Text::new(&member.text)
            .set("x", text_x)
            .set("y", y + 12.0)
            .set("font-family", theme.font_family.as_str())
            .set("font-size", theme.font_size)
            .set("fill", theme.text_color.to_css());

        // Статический - подчёркивание
        if member.is_static {
            text = text.set("text-decoration", "underline");
        }

        // Абстрактный - курсив
        if member.is_abstract {
            text = text.set("font-style", "italic");
        }

        group.add(text)
    }
}

impl Default for SvgRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for SvgRenderer {
    type Output = String;

    fn render(&self, layout: &LayoutResult, theme: &Theme) -> String {
        let doc = self.create_document(layout, theme);

        // PlantUML оборачивает всё содержимое в один `<g>` с общим шрифтом
        // и режимом подгонки текста. Группа нужна потребителям вывода:
        // по ней наследуется font-family, а lengthAdjust="spacing" задаёт
        // способ подгонки по textLength.
        let mut group = Group::new()
            .set("font-family", theme.font_family.as_str())
            .set("lengthAdjust", "spacing");

        // Сортируем элементы по z-layer для правильного порядка рендеринга
        // Элементы с меньшим z-layer рендерятся первыми (внизу)
        let mut sorted_elements: Vec<_> = layout.elements.iter().collect();
        sorted_elements.sort_by_key(|e| ZLayer::from_element(e));

        // Уникализируем идентификаторы.
        //
        // Layout-движки формируют id из статических частей без счётчика
        // (`msg_Alice_Bob`, `edge_User_Order`, `trans_A_B`), поэтому два
        // сообщения между одной парой участников дают одинаковые id.
        // В SVG атрибут id обязан быть уникальным: дубликаты — невалидный
        // документ, ломающий селекторы, якоря и `<use>`.
        let mut seen: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();

        for element in sorted_elements {
            let unique_id = {
                let base = element.id.as_str();
                let counter = seen.entry(base).or_insert(0);
                *counter += 1;
                if *counter == 1 {
                    base.to_string()
                } else {
                    format!("{base}_{counter}")
                }
            };
            let rendered = self.render_element_with_id(element, theme, &unique_id);
            group = group.add(rendered);
        }

        let doc = doc.add(group);

        // Собственного XML-заголовка PlantUML не пишет ни в одном из
        // эталонов: документ начинается сразу с `<svg>`. Опция оставлена
        // для потребителей, которым заголовок нужен.
        let svg = if self.options.xml_header {
            format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", doc)
        } else {
            doc.to_string()
        };

        // PlantUML проставляет textLength каждому текстовому узлу — это
        // ширина строки в метриках шрифта. Атрибут добавляется одной
        // постобработкой, а не в каждом из методов отрисовки текста:
        // так его невозможно забыть при добавлении нового метода.
        annotate_text_length(&svg)
    }
}

/// Высота заголовка состояния: в эталоне разделитель на 113.297 при
/// верхней границе 87, то есть 26.297.
const STATE_HEADER_HEIGHT: f64 = 26.297;

/// Величина скоса объёмной рамки узла deployment.
const NODE_3D_OFFSET: f64 = 10.0;

/// Отступ базовой линии заголовка узла от верха основного прямоугольника.
const NODE_TITLE_BASELINE: f64 = 16.0;

/// Смещение тени от фигуры (`skinparam shadowing true`).
/// Форматирует координату для атрибута SVG.
///
/// Убирает хвостовые нули: PlantUML пишет `26.5`, а не `26.500000`.
fn fmt(value: f64) -> String {
    let text = format!("{value:.3}");
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() || trimmed == "-" {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

// === Геометрия фигур участников sequence ===
//
// Все числа сняты с эталона `sequence_participants` (PlantUML 1.2026.9beta4):
// верхняя полоса подписи 55..85.297, нижняя 249.961..280.258.

/// Радиус головы стик-фигуры.
const ACTOR_HEAD_RADIUS: f64 = 8.0;

/// Полуразнос рук и ног стик-фигуры.
const ACTOR_LIMB_SPREAD: f64 = 13.0;

/// Смещения стик-фигуры от верха элемента (верхний блок).
const ACTOR_WAIST_OFFSET: f64 = 43.0;
const ACTOR_ARMS_OFFSET: f64 = 24.0;
const ACTOR_FEET_OFFSET: f64 = 58.0;

/// Смещения стик-фигуры от верха элемента (нижний блок, зеркально).
const ACTOR_FOOTER_HEAD_CY: f64 = 24.797;
const ACTOR_FOOTER_NECK: f64 = 32.797;
const ACTOR_FOOTER_WAIST: f64 = 59.797;
const ACTOR_FOOTER_ARMS: f64 = 40.797;
const ACTOR_FOOTER_FEET: f64 = 74.797;

/// Полуширина цилиндра базы данных.
const DATABASE_RX: f64 = 18.0;

/// Полувысота крышки цилиндра.
const DATABASE_CAP_RY: f64 = 10.0;

/// Высота тела цилиндра между крышками.
const DATABASE_BODY_HEIGHT: f64 = 26.0;

/// Смещения крышек цилиндра в нижнем блоке.
const DATABASE_FOOTER_TOP_CAP: f64 = 36.0;
const DATABASE_FOOTER_BOTTOM_CAP: f64 = 10.0;

/// Радиус кружка у boundary/control/entity.
const PARTICIPANT_ICON_RADIUS: f64 = 12.0;

/// Отступ подписи от низа верхней полосы.
const PARTICIPANT_LABEL_BASELINE_GAP: f64 = 2.302;

/// Отступ подписи от верха нижней полосы.
const PARTICIPANT_FOOTER_LABEL_BASELINE: f64 = 12.995;

/// Вынос скобки boundary влево от центра.
const BOUNDARY_BRACKET_OFFSET: f64 = 29.0;

/// Смещение центра кружка control от верха элемента.
const CONTROL_CIRCLE_OFFSET: f64 = 17.0;

/// Смещение стрелки control от верха элемента.
const CONTROL_ARROW_MID: f64 = 5.0;

/// Смещение стрелки control в нижнем блоке от низа элемента.
const CONTROL_FOOTER_ARROW_MID: f64 = 24.0;

/// Размеры стрелки control.
const CONTROL_ARROW_BACK: f64 = 4.0;
const CONTROL_ARROW_TIP: f64 = 2.0;
const CONTROL_ARROW_HALF: f64 = 5.0;

/// Смещение подчёркивания entity от верха элемента.
const ENTITY_UNDERLINE_OFFSET: f64 = 26.0;

/// Зазор между кружком entity и подчёркиванием в нижнем блоке.
const ENTITY_FOOTER_GAP: f64 = 2.0;

/// Смещение тени от фигуры (`skinparam shadowing true`).
const SHADOW_OFFSET: f64 = 3.0;

/// Цвет тела класса в эталоне PlantUML.
const CLASS_BODY_FILL: &str = "#F1F1F1";

/// Округляет до трёх знаков: PlantUML пишет `33.667`, а не `33.6670001`.
fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

/// Оценивает ширину текста в пикселях (для атрибута `textLength`).
///
/// Используется тот же измеритель, что и в layout-движках. Раньше здесь
/// была своя константа 0.481 em на любой символ, из-за чего `textLength`
/// в готовом SVG расходился с ширинами, по которым строилась раскладка.
fn measure_text(text: &str, font_size: f64) -> f64 {
    plantuml_layout::text::TextMeasurer::default().width(text, font_size)
}

/// Извлекает значение `font-size` из строки тега.
fn extract_font_size(tag: &str) -> Option<f64> {
    let marker = "font-size=\"";
    let start = tag.find(marker)? + marker.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}

/// Добавляет атрибут `textLength` каждому текстовому узлу, где его нет.
///
/// PlantUML проставляет его всегда: это ширина строки в метриках шрифта.
/// Разбор разметки простым поиском достаточен, потому что рендерер сам
/// формирует теги `<text>` и не вставляет экранированные `>` в атрибуты.
fn annotate_text_length(svg: &str) -> String {
    let mut out = String::with_capacity(svg.len() + svg.len() / 20);
    let mut rest = svg;

    while let Some(open) = rest.find("<text") {
        out.push_str(&rest[..open]);
        let after = &rest[open..];

        let Some(tag_end) = after.find('>') else {
            out.push_str(after);
            return out;
        };
        let tag = &after[..tag_end];

        // Уже есть — оставляем как есть
        if tag.contains("textLength") {
            out.push_str(&after[..=tag_end]);
            rest = &after[tag_end + 1..];
            continue;
        }

        let content_start = tag_end + 1;
        let content = &after[content_start..];
        let Some(close) = content.find("</text>") else {
            out.push_str(after);
            return out;
        };

        let label = content[..close].trim();
        let font_size = extract_font_size(tag).unwrap_or(13.0);
        // Многострочный текст: берём самую длинную строку
        let measured = label
            .lines()
            .map(|line| measure_text(line, font_size))
            .fold(0.0_f64, f64::max);

        out.push_str(tag);
        out.push_str(&format!(" textLength=\"{}\"", round3(measured)));
        out.push('>');
        out.push_str(&content[..close]);
        out.push_str("</text>");

        rest = &content[close + "</text>".len()..];
    }

    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_layout::ElementType;

    #[test]
    fn test_render_empty() {
        let renderer = SvgRenderer::new();
        let layout = LayoutResult::empty();
        let theme = Theme::default();

        let svg = renderer.render(&layout, &theme);
        assert!(svg.contains("<?xml"));
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn test_render_rectangle() {
        let renderer = SvgRenderer::new();
        let layout = LayoutResult {
            elements: vec![LayoutElement::new(
                "test",
                Rect::new(10.0, 10.0, 100.0, 50.0),
                ElementType::Rectangle {
                    label: "Hello".to_string(),
                    corner_radius: 5.0,
                },
            )],
            bounds: Rect::new(0.0, 0.0, 120.0, 70.0),
        };
        let theme = Theme::default();

        let svg = renderer.render(&layout, &theme);
        assert!(svg.contains("<rect"));
        assert!(svg.contains("Hello"));
    }
}
