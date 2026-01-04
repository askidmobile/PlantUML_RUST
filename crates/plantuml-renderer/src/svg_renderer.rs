//! SVG рендерер

use svg::node::element::{Definitions, Group, Marker, Path, Rectangle};
use svg::Document;

use crate::{
    ClassMember, ClassifierKind, EdgeType, ElementType, FragmentSection, LayoutElement, LayoutResult, 
    MemberVisibility, Point, Rect, RenderOptions, Renderer, ZLayer,
};
use plantuml_themes::Theme;

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

        let mut doc = Document::new()
            .set("width", width)
            .set("height", height)
            .set(
                "viewBox",
                (
                    bounds.x - margin,
                    bounds.y - margin,
                    bounds.width + margin * 2.0,
                    bounds.height + margin * 2.0,
                ),
            )
            .set("xmlns", "http://www.w3.org/2000/svg");

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
        let mut group = Group::new().set("id", element.id.as_str());

        match &element.element_type {
            ElementType::Rectangle {
                label,
                corner_radius,
            } => {
                group = self.render_rectangle(&element.bounds, label, *corner_radius, theme, group);
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
                group = self.render_uml_state(&element.bounds, name, description.as_deref(), theme, group);
            }
            ElementType::Actor { label } => {
                group = self.render_actor(&element.bounds, label, theme, group);
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
                // Рендерим как прямоугольник со скруглёнными углами
                let label = element.text.as_deref().unwrap_or("");
                group = self.render_rectangle(&element.bounds, label, 8.0, theme, group);
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
    ) -> Group {
        // PlantUML использует stroke-width: 0.5 для участников
        let rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("rx", corner_radius)
            .set("ry", corner_radius)
            .set("fill", theme.node_background.to_css())
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 0.5);

        group = group.add(rect);

        // Текст по центру
        let text = svg::node::element::Text::new(label)
            .set("x", bounds.x + bounds.width / 2.0)
            .set("y", bounds.y + bounds.height / 2.0)
            .set("text-anchor", "middle")
            .set("dominant-baseline", "middle")
            .set("font-family", theme.font_family.as_str())
            .set("font-size", theme.font_size)
            .set("fill", theme.text_color.to_css());

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
    fn render_initial_state(
        &self,
        bounds: &Rect,
        theme: &Theme,
        group: Group,
    ) -> Group {
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
    fn render_final_state(
        &self,
        bounds: &Rect,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
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
        let corner_radius = 10.0;
        let header_height = 25.0;
        
        // 1. Основной прямоугольник со скруглёнными углами
        let rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("rx", corner_radius)
            .set("ry", corner_radius)
            .set("fill", theme.node_background.to_css())
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1);

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
            .set("font-weight", "bold")
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

    /// Рендерит актёра (stick figure) для UseCase диаграмм
    fn render_actor(
        &self,
        bounds: &Rect,
        label: &str,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        let cx = bounds.x + bounds.width / 2.0;
        let top_y = bounds.y;
        
        // Размеры stick figure
        let head_radius = 8.0;
        let body_length = 20.0;
        let arm_width = 18.0;
        let leg_length = 15.0;
        let leg_spread = 10.0;
        
        // Позиции
        let head_cy = top_y + head_radius + 2.0;
        let neck_y = head_cy + head_radius;
        let waist_y = neck_y + body_length;
        let arms_y = neck_y + body_length * 0.3;
        let feet_y = waist_y + leg_length;
        
        // 1. Голова (круг)
        let head = svg::node::element::Ellipse::new()
            .set("cx", cx)
            .set("cy", head_cy)
            .set("rx", head_radius)
            .set("ry", head_radius)
            .set("fill", theme.node_background.to_css())
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1.5);
        group = group.add(head);
        
        // 2. Тело (вертикальная линия)
        let body = svg::node::element::Line::new()
            .set("x1", cx)
            .set("y1", neck_y)
            .set("x2", cx)
            .set("y2", waist_y)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1.5);
        group = group.add(body);
        
        // 3. Руки (горизонтальная линия)
        let arms = svg::node::element::Line::new()
            .set("x1", cx - arm_width / 2.0)
            .set("y1", arms_y)
            .set("x2", cx + arm_width / 2.0)
            .set("y2", arms_y)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1.5);
        group = group.add(arms);
        
        // 4. Левая нога
        let left_leg = svg::node::element::Line::new()
            .set("x1", cx)
            .set("y1", waist_y)
            .set("x2", cx - leg_spread)
            .set("y2", feet_y)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1.5);
        group = group.add(left_leg);
        
        // 5. Правая нога
        let right_leg = svg::node::element::Line::new()
            .set("x1", cx)
            .set("y1", waist_y)
            .set("x2", cx + leg_spread)
            .set("y2", feet_y)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1.5);
        group = group.add(right_leg);
        
        // 6. Текст имени под человечком
        let text_y = feet_y + 15.0;
        let text = svg::node::element::Text::new(label)
            .set("x", cx)
            .set("y", text_y)
            .set("text-anchor", "middle")
            .set("font-family", theme.font_family.as_str())
            .set("font-size", theme.font_size)
            .set("fill", theme.text_color.to_css());
        group = group.add(text);
        
        group
    }

    /// Рендерит систему/пакет (rectangle с заголовком сверху) для UseCase диаграмм
    fn render_system(
        &self,
        bounds: &Rect,
        title: &str,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
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
            let dash_pattern = if arrow_end || arrow_start { "2,2" } else { "5,5" };
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
                // Ортогональный путь (4 точки): текст на вертикальном сегменте
                // Вертикальный сегмент: points[1] -> points[2]
                let mid_y = (points[1].y + points[2].y) / 2.0;
                let text_x = points[1].x + 5.0; // справа от вертикальной линии
                (text_x, mid_y, "start")
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
                group = self.render_multiline_text(
                    text_x, text_y, label, anchor, font_size, theme, group
                );
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
    fn render_group(
        &self,
        bounds: &Rect,
        label: Option<&str>,
        children: &[LayoutElement],
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        // Рамка группы - СПЛОШНАЯ (как в PlantUML)
        let rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("fill", "none")
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1);

        group = group.add(rect);

        // Заголовок
        if let Some(label) = label {
            let header_bg = Rectangle::new()
                .set("x", bounds.x)
                .set("y", bounds.y)
                .set("width", bounds.width)
                .set("height", 20.0)
                .set("fill", theme.node_background.to_css());

            let text = svg::node::element::Text::new(label)
                .set("x", bounds.x + 5.0)
                .set("y", bounds.y + 14.0)
                .set("font-family", theme.font_family.as_str())
                .set("font-size", theme.font_size)
                .set("font-weight", "bold")
                .set("fill", theme.text_color.to_css());

            group = group.add(header_bg).add(text);
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
        let rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("fill", "none")
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1.5);

        group = group.add(rect);

        // 2. Пятиугольный заголовок (pentagon) в левом верхнем углу
        // Размеры: ширина ~50px для "alt", высота ~20px
        let label_text = fragment_type;
        let label_width = (label_text.len() as f64 * 8.0 + 16.0).max(40.0);
        let label_height = 20.0;
        let notch_size = 8.0; // размер "зазубрины" пятиугольника

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

        let pentagon = Path::new()
            .set("d", pentagon_path)
            .set("fill", theme.node_background.to_css())
            .set("stroke", theme.node_border.to_css())
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
                    .set("stroke-dasharray", "5,3");

                group = group.add(separator_line);

                // Текст условия else слева, НАД линией (с достаточным отступом)
                let else_label = if let Some(cond) = &section.condition {
                    format!("[{}]", cond)
                } else {
                    "[else]".to_string()
                };

                let else_text = svg::node::element::Text::new(else_label)
                    .set("x", bounds.x + 5.0)
                    .set("y", separator_y - 5.0) // текст над линией
                    .set("font-family", theme.font_family.as_str())
                    .set("font-size", theme.font_size - 1.0)
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
        let rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("rx", 2.5)
            .set("ry", 2.5)
            .set("fill", theme.node_background.to_css())
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 0.5);
        group = group.add(rect);

        let mut current_y = bounds.y + padding;

        // 2. Иконка классификатора (PlantUML style)
        let icon_x = bounds.x + padding + icon_size;
        let icon_y = current_y + icon_size;
        let (icon_fill, icon_letter) = match classifier_type {
            ClassifierKind::Class => ("#ADD1B2", "C"),         // зелёный
            ClassifierKind::Interface => ("#B4A7E5", "I"),     // фиолетовый
            ClassifierKind::AbstractClass => ("#A9DCDF", "A"), // голубой
            ClassifierKind::Enum => ("#EB937F", "E"),          // оранжевый
            ClassifierKind::Annotation => ("#FFDD8C", "@"),    // жёлтый
            ClassifierKind::Entity => ("#CCCCCC", "E"),        // серый
        };

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

        // 4. Название класса
        let name_text = svg::node::element::Text::new(name)
            .set("x", name_x)
            .set("y", current_y + line_height - 2.0)
            .set("font-family", theme.font_family.as_str())
            .set("font-size", theme.font_size)
            .set("font-weight", "bold")
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
            MemberVisibility::Public => ("#84BE84", "#038048"),     // зелёный
            MemberVisibility::Private => ("#C82829", "#C80000"),    // красный  
            MemberVisibility::Protected => ("#FFCC00", "#B38600"),  // жёлтый
            MemberVisibility::Package => ("#66CCFF", "#0099CC"),    // голубой
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
        let mut doc = self.create_document(layout, theme);

        // Сортируем элементы по z-layer для правильного порядка рендеринга
        // Элементы с меньшим z-layer рендерятся первыми (внизу)
        let mut sorted_elements: Vec<_> = layout.elements.iter().collect();
        sorted_elements.sort_by_key(|e| ZLayer::from_element(e));

        for element in sorted_elements {
            let rendered = self.render_element(element, theme);
            doc = doc.add(rendered);
        }

        if self.options.xml_header {
            format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", doc)
        } else {
            doc.to_string()
        }
    }
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
