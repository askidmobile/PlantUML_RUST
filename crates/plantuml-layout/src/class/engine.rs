//! ClassLayoutEngine - layout engine для диаграмм классов.

use plantuml_ast::class::{ClassDiagram, ClassifierType, NoteTarget, Package, RelationshipType};
use plantuml_ast::common::Direction;
use plantuml_model::{Point, Rect};

use crate::traits::LayoutEngine;
use crate::{
    ClassMember, ClassifierKind, EdgeType, ElementType, LayoutConfig, LayoutElement, LayoutResult,
    MemberVisibility,
};

use super::config::ClassLayoutConfig;

/// Цвет заметки в PlantUML — светло-жёлтый.
const CLASS_NOTE_BACKGROUND: &str = "#FEFFDD";

/// Поля текста внутри заметки.
const CLASS_NOTE_PADDING: f64 = 6.0;

/// Размер среза угла заметки.
const CLASS_NOTE_FOLD: f64 = 10.0;

/// Кегль текста заметки.
const CLASS_NOTE_FONT: f64 = 13.0;

/// Запас вокруг содержимого пакета и высота его заголовка.
const PACKAGE_PADDING: f64 = 10.0;
const PACKAGE_HEADER: f64 = 24.0;

/// Зазор между заметкой и элементом, к которому она привязана.
const CLASS_NOTE_GAP: f64 = 10.0;
use super::graph::Graph;
use super::sugiyama::SugiyamaLayout;

/// Layout engine для Class Diagrams
#[derive(Debug, Clone)]
pub struct ClassLayoutEngine {
    /// Конфигурация layout
    config: ClassLayoutConfig,
}

impl ClassLayoutEngine {
    /// Создаёт новый engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: ClassLayoutConfig::default(),
        }
    }

    /// Создаёт engine с заданной конфигурацией
    pub fn with_config(config: ClassLayoutConfig) -> Self {
        Self { config }
    }

    /// Выполняет layout диаграммы классов
    pub fn layout_diagram(&self, diagram: &ClassDiagram) -> LayoutResult {
        // Диаграмма из одних спрайтов тоже валидна.
        if diagram.classifiers.is_empty()
            && diagram.packages.is_empty()
            && diagram.sprites.is_empty()
            && diagram.notes.is_empty()
        {
            return LayoutResult::empty();
        }

        // Строим граф и выполняем Sugiyama layout
        let mut graph = Graph::from_diagram(diagram, &self.config);
        let mut sugiyama = SugiyamaLayout::new(&mut graph, &self.config);
        sugiyama.run();

        // Преобразуем результат в LayoutElements
        let mut elements = Vec::new();

        // Объявленные спрайты НЕ рисуются отдельными блоками.
        //
        // Проверено на сервере: `sprite $a {...}` без подстановки не
        // оставляет в выводе ничего — спрайт появляется только там, где
        // стоит `<$имя>`. Прежний код размещал каждый объявленный спрайт
        // отдельным блоком, из-за чего диаграмма разрасталась: для двух
        // объявленных спрайтов выходило 14 прямоугольников против 3 в
        // эталоне.
        //
        // Подстановка в подписи обрабатывается ниже: движок кладёт
        // спрайты в свойство элемента, а рендерер рисует их в тексте.
        //
        // Добавляем узлы (классы)
        for node in &graph.nodes {
            // Ищем оригинальный classifier для получения деталей
            let classifier = diagram
                .classifiers
                .iter()
                .find(|c| c.id.name == node.classifier_name)
                .or_else(|| {
                    // Ищем в пакетах
                    Self::find_classifier_in_packages(&diagram.packages, &node.classifier_name)
                });

            let element = self.create_class_element(node, classifier, diagram);
            elements.push(element);
        }

        // Добавляем рёбра (отношения)
        for edge in &graph.edges {
            let from_node = &graph.nodes[edge.from];
            let to_node = &graph.nodes[edge.to];

            let edge_element = self.create_edge_element(edge, from_node, to_node);
            elements.push(edge_element);
        }

        // `left to right direction` МЕНЯЕТ ОСИ раскладки.
        //
        // Проверено на сервере: для `class A; class B; A --> B` эталон
        // даёт 63x178 без директивы (рамки друг под другом) и 165x70 с
        // ней (рамки рядом). Направление сохранялось в AST, но раскладка
        // его не читала, поэтому вывод был одинаковым.
        if diagram.metadata.direction == Some(Direction::LeftToRight) {
            transpose_elements(&mut elements);
        }

        // Пакеты и заметки строятся по готовым положениям классов: так им не
        // мешает транспонирование и они видят итоговые координаты.
        let packages = self.layout_packages(diagram, &elements);
        elements.splice(0..0, packages);
        // Заметки. Раньше поле `ClassDiagram::notes` не читалось нигде:
        // грамматика заметки принимала, парсер их разбирал, а в раскладку
        // они не попадали, и в выводе не было ни текста, ни рамки.
        self.layout_notes(diagram, &mut elements);

        let mut result = LayoutResult {
            elements,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        };

        result.calculate_bounds();

        // Добавляем margin к bounds.
        //
        // Поля эталона АСИММЕТРИЧНЫ: слева и сверху 7.00, справа 14.28,
        // снизу 14.41 (измерено по class_inheritance; «один класс» и
        // «два класса» дают те же пропорции). Суммарное поле 21.3
        // складывается из margin этого движка и поля рендерера (7.0),
        // поэтому здесь margin равен 3.5, а не 7.0: вместе они дают
        // нужную сумму. Распределение слева/справа при этом не совпадает
        // с эталонным — это записано в журнале как открытый пункт.
        result.bounds.width += self.config.margin * 2.0;
        result.bounds.height += self.config.margin * 2.0;

        result
    }

    /// Рамки пакетов: охватывают свои классы и вложенные пакеты.
    ///
    /// Рамка строится по уже разложенным классам, поэтому пакет без классов
    /// получает минимальный размер и встаёт под диаграммой.
    fn layout_packages(
        &self,
        diagram: &ClassDiagram,
        elements: &[LayoutElement],
    ) -> Vec<LayoutElement> {
        let mut out = Vec::new();
        let mut spare_y = elements
            .iter()
            .map(|e| e.bounds.y + e.bounds.height)
            .fold(0.0_f64, f64::max)
            + PACKAGE_PADDING * 2.0;
        for package in &diagram.packages {
            self.layout_package(package, "", elements, &mut spare_y, &mut out);
        }
        out
    }

    /// Рамка одного пакета; возвращает занятый ею прямоугольник.
    fn layout_package(
        &self,
        package: &Package,
        parent_path: &str,
        elements: &[LayoutElement],
        spare_y: &mut f64,
        out: &mut Vec<LayoutElement>,
    ) -> Rect {
        let path = format!("{parent_path}/{}", package.name);
        // Дочерние пакеты кладутся в `out` раньше родителя; порядок рисования
        // выправляется ниже: родитель идёт первым и не закрывает потомков.
        let mut inner: Vec<LayoutElement> = Vec::new();
        let mut parts: Vec<Rect> = package
            .classifiers
            .iter()
            .filter_map(|c| elements.iter().find(|e| e.id == c.id.name))
            .map(|e| e.bounds)
            .collect();
        for child in &package.packages {
            parts.push(self.layout_package(child, &path, elements, spare_y, &mut inner));
        }

        let title = package_title(package);
        let title_width = self.config.text.width(&title, self.config.font_size + 1.0);
        let mut bounds = match union_of(&parts) {
            Some(content) => Rect::new(
                content.x - PACKAGE_PADDING,
                content.y - PACKAGE_PADDING - PACKAGE_HEADER,
                content.width + PACKAGE_PADDING * 2.0,
                content.height + PACKAGE_PADDING * 2.0 + PACKAGE_HEADER,
            ),
            None => {
                let rect = Rect::new(
                    0.0,
                    *spare_y,
                    title_width + PACKAGE_PADDING * 2.0,
                    PACKAGE_HEADER + PACKAGE_PADDING * 2.0,
                );
                *spare_y += rect.height + PACKAGE_PADDING;
                rect
            }
        };
        // Заголовок должен помещаться в рамку.
        if bounds.width < title_width + PACKAGE_PADDING * 2.0 {
            let extra = title_width + PACKAGE_PADDING * 2.0 - bounds.width;
            bounds.x -= extra / 2.0;
            bounds.width += extra;
        }

        let mut properties = std::collections::HashMap::new();
        properties.insert("fill".to_string(), "none".to_string());
        properties.insert("rx".to_string(), "2.5".to_string());
        if let Some(color) = &package.background_color {
            properties.insert("fill".to_string(), color.to_css());
        }
        out.push(LayoutElement {
            id: format!("package_{}", package.name),
            bounds,
            text: None,
            properties,
            element_type: ElementType::System { title },
        });
        out.append(&mut inner);
        bounds
    }

    /// Размещает заметки диаграммы и проводит к ним линии.
    ///
    /// Заметка привязывается к элементу через `anchors`; плавающая (без
    /// якоря) встаёт под диаграммой. Размер даёт текст, а не константа, чтобы
    /// ни одна строка не терялась. Если место у якоря занято, заметка
    /// отодвигается от него. Цвет в PlantUML светло-жёлтый (#FEFFDD).
    fn layout_notes(&self, diagram: &ClassDiagram, elements: &mut Vec<LayoutElement>) {
        if diagram.notes.is_empty() {
            return;
        }

        let mut bottom = elements
            .iter()
            .map(|e| e.bounds.y + e.bounds.height)
            .fold(0.0_f64, f64::max);
        let mut placed: Vec<Rect> = Vec::new();
        let mut extra: Vec<LayoutElement> = Vec::new();
        let mut note_bounds: Vec<Option<Rect>> = vec![None; diagram.notes.len()];
        // Пакеты и рёбра не мешают размещению, а классы и заметки — мешают.
        let blockers = |elements: &[LayoutElement]| -> Vec<Rect> {
            elements
                .iter()
                .filter(|e| matches!(e.element_type, ElementType::ClassBox { .. }))
                .map(|e| e.bounds)
                .collect()
        };
        placed.extend(blockers(elements));

        for (index, note) in diagram.notes.iter().enumerate() {
            let lines: Vec<&str> = note.text.split('\n').collect();
            let text_width = lines
                .iter()
                .map(|line| self.config.text.width(line, CLASS_NOTE_FONT))
                .fold(0.0_f64, f64::max);
            let line_height = CLASS_NOTE_FONT + 4.0;
            let width = text_width + CLASS_NOTE_PADDING * 2.0 + CLASS_NOTE_FOLD / 2.0;
            let height = lines.len() as f64 * line_height + CLASS_NOTE_PADDING;

            // Заметка стоит у якоря, у линка `note on link` либо под схемой.
            let anchor = note
                .anchors
                .first()
                .and_then(|name| find_bounds(elements, name));
            let link_point = diagram.note_links.iter().find_map(|link| {
                if link.note != index {
                    return None;
                }
                match &link.target {
                    NoteTarget::Link { from, to } => edge_midpoint(elements, from, to),
                    NoteTarget::Element(_) => None,
                }
            });

            let (mut x, mut y, dx, dy) = match (anchor, link_point) {
                (Some(b), _) => match note.position {
                    plantuml_ast::common::NotePosition::Left => {
                        (b.x - width - CLASS_NOTE_GAP, b.y, -1.0, 0.0)
                    }
                    plantuml_ast::common::NotePosition::Right
                    | plantuml_ast::common::NotePosition::Over => {
                        (b.x + b.width + CLASS_NOTE_GAP, b.y, 1.0, 0.0)
                    }
                    plantuml_ast::common::NotePosition::Top => {
                        (b.x, b.y - height - CLASS_NOTE_GAP, 0.0, -1.0)
                    }
                    plantuml_ast::common::NotePosition::Bottom => {
                        (b.x, b.y + b.height + CLASS_NOTE_GAP, 0.0, 1.0)
                    }
                },
                (None, Some(p)) => (p.x + CLASS_NOTE_GAP, p.y - height / 2.0, 1.0, 0.0),
                (None, None) => (0.0, bottom + CLASS_NOTE_GAP, 0.0, 1.0),
            };

            // Сдвигаем, пока не освободится место.
            for _ in 0..60 {
                let candidate = Rect::new(x, y, width, height);
                if !placed.iter().any(|r| overlaps(r, &candidate)) {
                    break;
                }
                x += dx * 10.0;
                y += dy * 10.0;
                if dx == 0.0 && dy == 0.0 {
                    break;
                }
            }

            let rect = Rect::new(x, y, width, height);
            placed.push(rect);
            note_bounds[index] = Some(rect);
            if anchor.is_none() {
                bottom = bottom.max(y + height);
            }

            let mut properties = std::collections::HashMap::new();
            let fill = note
                .background_color
                .as_ref()
                .map(|c| c.to_css())
                .unwrap_or_else(|| CLASS_NOTE_BACKGROUND.to_string());
            properties.insert("fill".to_string(), fill);
            extra.push(LayoutElement {
                id: format!("note_{index}"),
                bounds: rect,
                text: None,
                properties,
                element_type: ElementType::Note {
                    label: String::new(),
                    fold: CLASS_NOTE_FOLD,
                },
            });
            for (row, line) in lines.iter().enumerate() {
                let mut text_properties = std::collections::HashMap::new();
                text_properties.insert("baseline".to_string(), {
                    let top = y + CLASS_NOTE_PADDING / 2.0 + row as f64 * line_height;
                    format!("{}", top + CLASS_NOTE_FONT)
                });
                extra.push(LayoutElement {
                    id: format!("note_{index}_line_{row}"),
                    bounds: Rect::new(
                        x + CLASS_NOTE_PADDING,
                        y + CLASS_NOTE_PADDING / 2.0 + row as f64 * line_height,
                        self.config.text.width(line, CLASS_NOTE_FONT),
                        line_height,
                    ),
                    text: None,
                    properties: text_properties,
                    element_type: ElementType::Text {
                        text: (*line).to_string(),
                        font_size: CLASS_NOTE_FONT,
                    },
                });
            }

            // Линия к якорю (если заметка привязана к элементу).
            if let Some(b) = anchor {
                extra.push(note_connector(
                    format!("note_link_{index}"),
                    rect,
                    b,
                    index,
                    LineStyleHint::Dashed,
                ));
            }
        }

        // Линии `A .. N` и `note on link`: от заметки к элементу или к середине связи.
        for (number, link) in diagram.note_links.iter().enumerate() {
            let Some(Some(note_rect)) = note_bounds.get(link.note) else {
                continue;
            };
            let target = match &link.target {
                NoteTarget::Element(name) => find_bounds(elements, name),
                NoteTarget::Link { from, to } => {
                    edge_midpoint(elements, from, to).map(|p| Rect::new(p.x, p.y, 0.0, 0.0))
                }
            };
            if let Some(target) = target {
                let dashed = !matches!(link.line_style, plantuml_ast::common::LineStyle::Solid);
                extra.push(note_connector(
                    format!("note_edge_{number}"),
                    *note_rect,
                    target,
                    link.note,
                    if dashed {
                        LineStyleHint::Dashed
                    } else {
                        LineStyleHint::Solid
                    },
                ));
            }
        }

        elements.extend(extra);
    }

    /// Ищет classifier в пакетах рекурсивно
    fn find_classifier_in_packages<'a>(
        packages: &'a [plantuml_ast::class::Package],
        name: &str,
    ) -> Option<&'a plantuml_ast::class::Classifier> {
        for package in packages {
            if let Some(c) = package.classifiers.iter().find(|c| c.id.name == name) {
                return Some(c);
            }
            if let Some(c) = Self::find_classifier_in_packages(&package.packages, name) {
                return Some(c);
            }
        }
        None
    }

    /// Создаёт LayoutElement для класса
    fn create_class_element(
        &self,
        node: &super::graph::Node,
        classifier: Option<&plantuml_ast::class::Classifier>,
        diagram: &ClassDiagram,
    ) -> LayoutElement {
        // Определяем тип классификатора и стереотип
        let (classifier_kind, stereotype) = classifier
            .map(|c| {
                let kind = match c.classifier_type {
                    ClassifierType::Interface => ClassifierKind::Interface,
                    ClassifierType::AbstractClass => ClassifierKind::AbstractClass,
                    ClassifierType::Enum => ClassifierKind::Enum,
                    ClassifierType::Annotation => ClassifierKind::Annotation,
                    ClassifierType::Entity => ClassifierKind::Entity,
                    ClassifierType::Struct => ClassifierKind::Struct,
                    ClassifierType::Protocol => ClassifierKind::Protocol,
                    ClassifierType::Exception => ClassifierKind::Exception,
                    ClassifierType::Metaclass => ClassifierKind::Metaclass,
                    ClassifierType::Stereotype => ClassifierKind::Stereotype,
                    ClassifierType::Circle => ClassifierKind::Circle,
                    ClassifierType::Diamond => ClassifierKind::Diamond,
                    ClassifierType::Class => ClassifierKind::Class,
                };
                // Авто-стереотипа у видов элементов нет: PlantUML рисует только
                // значок (проверено на сервере: `interface I` -> ['I'],
                // `struct S` -> ['S']). Рисуется лишь заданный в `<<...>>`.
                let stereo = super::graph::stereotype_label(c);
                (kind, stereo)
            })
            .unwrap_or((ClassifierKind::Class, None));

        let convert = |m: &plantuml_ast::class::Member| ClassMember {
            visibility: Self::convert_visibility(&m.visibility),
            text: m.display_text(),
            is_static: m.is_static,
            is_abstract: m.is_abstract,
            separator: m.separator,
        };
        let fields: Vec<ClassMember> = classifier
            .map(|c| c.fields.iter().map(convert).collect())
            .unwrap_or_default();
        let methods: Vec<ClassMember> = classifier
            .map(|c| c.methods.iter().map(convert).collect())
            .unwrap_or_default();

        // Спрайты, на которые ссылается подпись через `<$имя>`.
        //
        // Рендерер заменяет такие вставки прямоугольниками палитры —
        // PlantUML рисует спрайт прямо внутри строки текста, разбивая её
        // на часть до и часть после.
        let mut properties = std::collections::HashMap::new();
        if let Some(c) = classifier {
            if let Some(generics) = c.generics.as_deref().filter(|g| !g.is_empty()) {
                properties.insert("generics".to_string(), generics.to_string());
            }
            if let Some(stereotype) = &c.stereotype {
                if let Some(letter) = stereotype.spot {
                    properties.insert("spot".to_string(), letter.to_string());
                }
                if let Some(color) = &stereotype.background_color {
                    properties.insert("spot-fill".to_string(), color.to_css());
                }
            }
            if let Some(color) = &c.background_color {
                properties.insert("fill".to_string(), color.to_css());
            }
            if let Some(color) = &c.border_color {
                properties.insert("stroke".to_string(), color.to_css());
            }
        }
        if !diagram.sprites.is_empty() && node.classifier_name.contains("<$") {
            let encoded: Vec<String> = diagram
                .sprites
                .iter()
                .map(|sprite| {
                    format!(
                        "{}|{}x{}|{}",
                        sprite.name,
                        sprite.width,
                        sprite.height,
                        sprite.rows.join(",")
                    )
                })
                .collect();
            properties.insert("sprites".to_string(), encoded.join(";"));

            // Векторные спрайты передаются ОТДЕЛЬНЫМИ свойствами, по
            // одному на спрайт: тело SVG содержит разделители `|`, `;`
            // и `,`, поэтому упаковка в общую строку сломала бы разбор.
            // Ключ — `sprite-svg-<имя>`, значение — атрибуты и тело.
            for sprite in &diagram.sprites {
                let Some(vector) = &sprite.svg else {
                    continue;
                };
                properties.insert(
                    format!("sprite-svg-{}", sprite.name),
                    format!("{}\n{}", vector.attrs, vector.body),
                );
            }
        }

        LayoutElement {
            id: node.id.clone(),
            bounds: Rect::new(node.x, node.y, node.size.width, node.size.height),
            text: None,
            properties,
            element_type: ElementType::ClassBox {
                classifier_type: classifier_kind,
                name: node.classifier_name.clone(),
                stereotype,
                fields,
                methods,
            },
        }
    }

    /// Конвертирует Visibility из AST в MemberVisibility
    fn convert_visibility(visibility: &plantuml_ast::class::Visibility) -> MemberVisibility {
        match visibility {
            plantuml_ast::class::Visibility::Public => MemberVisibility::Public,
            plantuml_ast::class::Visibility::Private => MemberVisibility::Private,
            plantuml_ast::class::Visibility::Protected => MemberVisibility::Protected,
            plantuml_ast::class::Visibility::Package => MemberVisibility::Package,
        }
    }

    /// Создаёт LayoutElement для ребра (отношения)
    fn create_edge_element(
        &self,
        edge: &super::graph::Edge,
        from_node: &super::graph::Node,
        to_node: &super::graph::Node,
    ) -> LayoutElement {
        // Определяем визуальное направление стрелки
        // В графе: from_node = родитель (слой 0, вверху), to_node = потомок (ниже)
        // Для наследования стрелка должна идти ОТ потомка К родителю (снизу вверх)
        // Для композиции/агрегации стрелка идёт ОТ владельца К части
        let (visual_from, visual_to) = match edge.relationship_type {
            RelationshipType::Inheritance | RelationshipType::Realization => {
                // Стрелка от потомка к родителю
                (to_node, from_node)
            }
            _ => (from_node, to_node),
        };

        // Вычисляем точки соединения (передаём тип связи для правильного выбора грани)
        let (start_point, end_point) =
            self.calculate_connection_points(visual_from, visual_to, edge.relationship_type);

        // Создаём путь с ортогональными линиями
        let points = self.create_orthogonal_path(start_point, end_point, visual_from, visual_to);

        // Определяем стрелки и тип линии на основе типа отношения
        // arrow_end = маркер на конце линии (у целевого узла)
        let (arrow_start, arrow_end, dashed, edge_type) = match edge.relationship_type {
            RelationshipType::Inheritance => (false, true, false, EdgeType::Inheritance), // --|>
            RelationshipType::Realization => (false, true, true, EdgeType::Realization),  // ..|>
            RelationshipType::Composition => (true, false, false, EdgeType::Composition), // *--
            RelationshipType::Aggregation => (true, false, false, EdgeType::Aggregation), // o--
            RelationshipType::Association => (false, true, false, EdgeType::Association), // -->
            RelationshipType::Dependency => (false, true, true, EdgeType::Dependency),    // ..>
            RelationshipType::Link => (false, false, false, EdgeType::Link),              // --
        };

        // Если ребро было обращено при удалении циклов, меняем местами стрелки
        let (arrow_start, arrow_end) = if edge.reversed {
            (arrow_end, arrow_start)
        } else {
            (arrow_start, arrow_end)
        };

        // Для кардинальностей: если ребро было инвертировано для визуала, меняем местами
        let (from_card, to_card) = match edge.relationship_type {
            RelationshipType::Inheritance | RelationshipType::Realization => {
                // Визуально стрелка идёт от to_node к from_node
                (edge.to_cardinality.clone(), edge.from_cardinality.clone())
            }
            _ => (edge.from_cardinality.clone(), edge.to_cardinality.clone()),
        };

        LayoutElement {
            id: format!("edge_{}_{}", from_node.id, to_node.id),
            bounds: self.calculate_edge_bounds(&points),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Edge {
                points,
                label: edge.label.clone(),
                arrow_start,
                arrow_end,
                dashed,
                edge_type,
                from_cardinality: from_card,
                to_cardinality: to_card,
            },
        }
    }

    /// Вычисляет точки соединения между двумя узлами
    fn calculate_connection_points(
        &self,
        from: &super::graph::Node,
        to: &super::graph::Node,
        relationship_type: RelationshipType,
    ) -> (Point, Point) {
        let from_center_x = from.x + from.size.width / 2.0;
        let to_center_x = to.x + to.size.width / 2.0;

        // Для наследования и реализации ВСЕГДА используем верхнюю/нижнюю грань
        // независимо от горизонтального расположения узлов
        match relationship_type {
            RelationshipType::Inheritance | RelationshipType::Realization => {
                // from = потомок (снизу), to = родитель (сверху)
                // Стрелка выходит из верхней грани потомка, входит в нижнюю грань родителя
                let start = Point::new(from_center_x, from.y); // верх потомка
                let end = Point::new(to_center_x, to.y + to.size.height); // низ родителя
                (start, end)
            }
            _ => {
                // Для других типов связей - автоопределение направления
                let from_center_y = from.y + from.size.height / 2.0;
                let to_center_y = to.y + to.size.height / 2.0;
                let dx = to_center_x - from_center_x;
                let dy = to_center_y - from_center_y;

                if dy.abs() > dx.abs() {
                    // Вертикальное соединение
                    if dy > 0.0 {
                        (
                            Point::new(from_center_x, from.y + from.size.height),
                            Point::new(to_center_x, to.y),
                        )
                    } else {
                        (
                            Point::new(from_center_x, from.y),
                            Point::new(to_center_x, to.y + to.size.height),
                        )
                    }
                } else {
                    // Горизонтальное соединение
                    if dx > 0.0 {
                        (
                            Point::new(from.x + from.size.width, from_center_y),
                            Point::new(to.x, to_center_y),
                        )
                    } else {
                        (
                            Point::new(from.x, from_center_y),
                            Point::new(to.x + to.size.width, to_center_y),
                        )
                    }
                }
            }
        }
    }

    /// Создаёт ортогональный путь между точками (с коленом)
    fn create_orthogonal_path(
        &self,
        start: Point,
        end: Point,
        _from: &super::graph::Node,
        _to: &super::graph::Node,
    ) -> Vec<Point> {
        let dx = end.x - start.x;
        let dy = end.y - start.y;

        // Если точки почти на одной линии - прямая
        if dx.abs() < 1.0 || dy.abs() < 1.0 {
            return vec![start, end];
        }

        // Ортогональный путь с коленом
        // Для вертикального наследования (потомок снизу, родитель сверху):
        // start = верх потомка, end = низ родителя
        // Линия: вверх от потомка → горизонтально → вниз к родителю

        // Вычисляем Y для горизонтального сегмента
        // Это должно быть между нижней гранью родителя и верхней гранью потомка
        let mid_y = if dy < 0.0 {
            // end выше start (типичное наследование: потомок внизу)
            // mid_y = середина между end.y и start.y
            end.y + (start.y - end.y) / 2.0
        } else {
            // end ниже start
            start.y + (end.y - start.y) / 2.0
        };

        vec![
            start,
            Point::new(start.x, mid_y),
            Point::new(end.x, mid_y),
            end,
        ]
    }

    /// Вычисляет bounds для ребра
    fn calculate_edge_bounds(&self, points: &[Point]) -> Rect {
        if points.is_empty() {
            return Rect::new(0.0, 0.0, 0.0, 0.0);
        }

        let min_x = points.iter().map(|p| p.x).fold(f64::MAX, f64::min);
        let min_y = points.iter().map(|p| p.y).fold(f64::MAX, f64::min);
        let max_x = points.iter().map(|p| p.x).fold(f64::MIN, f64::max);
        let max_y = points.iter().map(|p| p.y).fold(f64::MIN, f64::max);

        Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }
}

#[derive(Clone, Copy)]
enum LineStyleHint {
    Solid,
    Dashed,
}

/// Подпись рамки пакета: имя и, если есть, стереотип.
fn package_title(package: &Package) -> String {
    let stereotype: Vec<&str> = package
        .stereotype
        .iter()
        .flat_map(|s| s.names.iter())
        .map(|n| n.trim())
        .filter(|n| !n.is_empty())
        .collect();
    if stereotype.is_empty() {
        package.name.clone()
    } else {
        format!("{} «{}»", package.name, stereotype.join(", "))
    }
}

/// Прямоугольник, охватывающий все данные.
fn union_of(parts: &[Rect]) -> Option<Rect> {
    let first = parts.first()?;
    let (mut x0, mut y0) = (first.x, first.y);
    let (mut x1, mut y1) = (first.x + first.width, first.y + first.height);
    for r in parts {
        x0 = x0.min(r.x);
        y0 = y0.min(r.y);
        x1 = x1.max(r.x + r.width);
        y1 = y1.max(r.y + r.height);
    }
    Some(Rect::new(x0, y0, x1 - x0, y1 - y0))
}

/// Пересекаются ли прямоугольники (касание не считается).
fn overlaps(a: &Rect, b: &Rect) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

/// Рамка класса или пакета по имени.
fn find_bounds(elements: &[LayoutElement], name: &str) -> Option<Rect> {
    elements
        .iter()
        .find(|e| e.id == name || e.id == format!("package_{name}"))
        .map(|e| e.bounds)
}

/// Середина связи между двумя элементами (в любом направлении записи).
fn edge_midpoint(elements: &[LayoutElement], from: &str, to: &str) -> Option<Point> {
    let direct = format!("edge_{from}_{to}");
    let reverse = format!("edge_{to}_{from}");
    elements.iter().find_map(|e| {
        if e.id != direct && e.id != reverse {
            return None;
        }
        match &e.element_type {
            ElementType::Edge { points, .. } if !points.is_empty() => {
                let a = points.first()?;
                let b = points.last()?;
                Some(Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0))
            }
            _ => None,
        }
    })
}

/// Точка границы прямоугольника на луче из его центра к `target`.
fn boundary_point(rect: &Rect, target: Point) -> Point {
    let center = Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    let (dx, dy) = (target.x - center.x, target.y - center.y);
    if dx == 0.0 && dy == 0.0 {
        return center;
    }
    let tx = if dx != 0.0 {
        rect.width / 2.0 / dx.abs()
    } else {
        f64::MAX
    };
    let ty = if dy != 0.0 {
        rect.height / 2.0 / dy.abs()
    } else {
        f64::MAX
    };
    let t = tx.min(ty).min(1.0);
    Point::new(center.x + dx * t, center.y + dy * t)
}

/// Линия между заметкой и целью без стрелок.
fn note_connector(
    id: String,
    note: Rect,
    target: Rect,
    _note_index: usize,
    style: LineStyleHint,
) -> LayoutElement {
    let target_center = Point::new(
        target.x + target.width / 2.0,
        target.y + target.height / 2.0,
    );
    let note_center = Point::new(note.x + note.width / 2.0, note.y + note.height / 2.0);
    let start = boundary_point(&note, target_center);
    let end = boundary_point(&target, note_center);
    let bounds = Rect::new(
        start.x.min(end.x),
        start.y.min(end.y),
        (start.x - end.x).abs(),
        (start.y - end.y).abs(),
    );
    LayoutElement {
        id,
        bounds,
        text: None,
        properties: std::collections::HashMap::new(),
        element_type: ElementType::Edge {
            points: vec![start, end],
            label: None,
            arrow_start: false,
            arrow_end: false,
            dashed: matches!(style, LineStyleHint::Dashed),
            edge_type: EdgeType::Link,
            from_cardinality: None,
            to_cardinality: None,
        },
    }
}

impl Default for ClassLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine for ClassLayoutEngine {
    type Input = ClassDiagram;

    fn layout(&self, input: &Self::Input, _config: &LayoutConfig) -> LayoutResult {
        self.layout_diagram(input)
    }
}

/// Меняет оси раскладки: `(x, y, ширина, высота)` → `(y, x, высота, ширина)`.
///
/// Так реализуется `left to right direction`: диаграмма строится обычным
/// образом (сверху вниз), а затем оси меняются местами — PlantUML даёт
/// именно транспонированный результат. Точки рёбер переносятся тоже,
/// иначе стрелки остались бы на прежних местах.
fn transpose_elements(elements: &mut [crate::LayoutElement]) {
    for element in elements.iter_mut() {
        let bounds = element.bounds;
        element.bounds = Rect::new(bounds.y, bounds.x, bounds.height, bounds.width);

        if let crate::ElementType::Edge { points, .. } = &mut element.element_type {
            for point in points.iter_mut() {
                let (x, y) = (point.x, point.y);
                point.x = y;
                point.y = x;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_ast::class::{Classifier, Member, Relationship, Visibility};

    #[test]
    fn test_empty_diagram() {
        let engine = ClassLayoutEngine::new();
        let diagram = ClassDiagram::new();

        let result = engine.layout_diagram(&diagram);

        assert!(result.elements.is_empty());
    }

    #[test]
    fn test_single_class() {
        let engine = ClassLayoutEngine::new();
        let mut diagram = ClassDiagram::new();

        let mut user = Classifier::new("User");
        user.add_field(Member::field("id", "Long").with_visibility(Visibility::Private));
        user.add_method(Member::method("getId").with_visibility(Visibility::Public));
        diagram.add_class(user);

        let result = engine.layout_diagram(&diagram);

        assert_eq!(result.elements.len(), 1);
        assert!(result.bounds.width > 0.0);
        assert!(result.bounds.height > 0.0);
    }

    #[test]
    fn test_inheritance_hierarchy() {
        let engine = ClassLayoutEngine::new();
        let mut diagram = ClassDiagram::new();

        // Animal -> Dog, Cat
        diagram.add_class(Classifier::new("Animal"));
        diagram.add_class(Classifier::new("Dog"));
        diagram.add_class(Classifier::new("Cat"));

        diagram.add_relationship(Relationship::inheritance("Dog", "Animal"));
        diagram.add_relationship(Relationship::inheritance("Cat", "Animal"));

        let result = engine.layout_diagram(&diagram);

        // 3 класса + 2 ребра
        assert_eq!(result.elements.len(), 5);

        // Проверяем, что все элементы имеют валидные bounds
        for elem in &result.elements {
            assert!(elem.bounds.width >= 0.0);
            assert!(elem.bounds.height >= 0.0);
        }
    }

    #[test]
    fn test_interface_implementation() {
        let engine = ClassLayoutEngine::new();
        let mut diagram = ClassDiagram::new();

        diagram.add_class(Classifier::interface("Serializable"));
        diagram.add_class(Classifier::new("User"));

        diagram.add_relationship(Relationship::realization("User", "Serializable"));

        let result = engine.layout_diagram(&diagram);

        // 2 класса + 1 ребро
        assert_eq!(result.elements.len(), 3);
    }

    #[test]
    fn test_complex_diagram() {
        let engine = ClassLayoutEngine::new();
        let mut diagram = ClassDiagram::new();

        // Более сложная иерархия
        diagram.add_class(Classifier::interface("Repository"));
        diagram.add_class(Classifier::abstract_class("AbstractRepository"));
        diagram.add_class(Classifier::new("UserRepository"));
        diagram.add_class(Classifier::new("ProductRepository"));
        diagram.add_class(Classifier::new("User"));
        diagram.add_class(Classifier::new("Product"));

        // Отношения
        diagram.add_relationship(Relationship::realization(
            "AbstractRepository",
            "Repository",
        ));
        diagram.add_relationship(Relationship::inheritance(
            "UserRepository",
            "AbstractRepository",
        ));
        diagram.add_relationship(Relationship::inheritance(
            "ProductRepository",
            "AbstractRepository",
        ));
        diagram.add_relationship(Relationship::new(
            "UserRepository",
            "User",
            RelationshipType::Association,
        ));
        diagram.add_relationship(Relationship::new(
            "ProductRepository",
            "Product",
            RelationshipType::Association,
        ));

        let result = engine.layout_diagram(&diagram);

        // 6 классов + 5 рёбер
        assert_eq!(result.elements.len(), 11);

        // Проверяем, что bounds охватывает все элементы
        assert!(result.bounds.width > 0.0);
        assert!(result.bounds.height > 0.0);
    }
}
