//! ClassLayoutEngine - layout engine для диаграмм классов.

use plantuml_ast::class::{ClassDiagram, ClassifierType, NoteTarget, Package, RelationshipType};
use plantuml_ast::common::{ArrowShape, Direction, LineStyle};
use plantuml_model::{Point, Rect};
use std::collections::HashMap;

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
use crate::graph::scene::{Scene, SceneEdge};
use crate::graph::Direction as GraphDirection;

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
        let graph = Graph::from_diagram(diagram, &self.config);
        let mut scene = Scene::new(
            self.config.node_horizontal_spacing,
            self.config.layer_vertical_spacing,
            PACKAGE_PADDING,
            self.config.text,
        );
        if diagram.metadata.direction == Some(Direction::LeftToRight) {
            scene.set_direction(GraphDirection::LeftToRight);
        }

        // Пакеты — кластеры, поэтому их рамки не накрывают чужие классы.
        let mut cluster_of: HashMap<&str, usize> = HashMap::new();
        for package in &diagram.packages {
            self.add_package(package, None, &mut scene, &mut cluster_of);
        }
        for node in &graph.nodes {
            let classifier = diagram
                .classifiers
                .iter()
                .find(|c| c.id.name == node.classifier_name)
                .or_else(|| {
                    Self::find_classifier_in_packages(&diagram.packages, &node.classifier_name)
                });
            let element = self.create_class_element(node, classifier, diagram);
            let cluster = cluster_of.get(node.id.as_str()).copied();
            scene.add_node(cluster, &[node.id.as_str()], vec![element]);
        }
        for rel in &diagram.relationships {
            scene.add_edge(self.create_edge(rel));
        }

        let mut result = scene.build();
        let mut elements = std::mem::take(&mut result.elements);
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

    /// Размещает заметки диаграммы и проводит к ним линии.
    ///
    /// Заметка привязывается к элементу через `anchors`; плавающая (без
    /// якоря) встаёт под диаграммой. Размер даёт текст, а не константа, чтобы
    /// ни одна строка не терялась. Если место у якоря занято, заметка
    /// отодвигается от него. Цвет в PlantUML светло-жёлтый (#FEFFDD).
    /// Объявляет пакет и его вложенные пакеты кластерами.
    fn add_package<'a>(
        &self,
        package: &'a Package,
        parent: Option<usize>,
        scene: &mut Scene,
        cluster_of: &mut HashMap<&'a str, usize>,
    ) {
        let title = package_title(package);
        let title_width = self.config.text.width(&title, self.config.font_size + 1.0);
        let mut properties = HashMap::new();
        properties.insert("fill".to_string(), "none".to_string());
        properties.insert("rx".to_string(), "2.5".to_string());
        // Пакет класса — рамка с вкладкой-именем слева сверху.
        properties.insert("cluster-shape".to_string(), "package".to_string());
        if let Some(color) = &package.background_color {
            properties.insert("fill".to_string(), color.to_css());
        }
        let cluster = scene.add_cluster(
            parent,
            &[package.name.as_str()],
            PACKAGE_HEADER,
            title_width + PACKAGE_PADDING * 2.0,
            LayoutElement {
                id: format!("package_{}", package.name),
                bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
                text: None,
                properties,
                element_type: ElementType::System { title },
            },
        );
        for classifier in &package.classifiers {
            cluster_of.insert(classifier.id.name.as_str(), cluster);
        }
        for nested in &package.packages {
            self.add_package(nested, Some(cluster), scene, cluster_of);
        }
    }

    /// Связь: концы и украшения берутся из записи стрелки, а не из типа.
    fn create_edge(&self, rel: &plantuml_ast::class::Relationship) -> SceneEdge {
        let (from, to, arrow, from_card, to_card) = if rel.declared_by_keyword {
            // `class Dog extends Animal`: родитель сверху, значок у него.
            let arrow = ArrowShape {
                head_from: true,
                head_to: false,
                ..ArrowShape::default()
            };
            (
                &rel.to,
                &rel.from,
                arrow,
                &rel.to_cardinality,
                &rel.from_cardinality,
            )
        } else {
            (
                &rel.from,
                &rel.to,
                rel.arrow,
                &rel.from_cardinality,
                &rel.to_cardinality,
            )
        };
        let mut edge = SceneEdge::new(from.clone(), to.clone(), arrow);
        edge.id = Some(format!("edge_{from}_{to}"));
        edge.label = rel.label.clone();
        edge.from_cardinality = from_card.clone();
        edge.to_cardinality = to_card.clone();
        (edge.dashed, edge.edge_type) = match rel.relationship_type {
            RelationshipType::Inheritance => (false, EdgeType::Inheritance),
            RelationshipType::Realization => (true, EdgeType::Realization),
            RelationshipType::Composition => (false, EdgeType::Composition),
            RelationshipType::Aggregation => (false, EdgeType::Aggregation),
            RelationshipType::Association => (false, EdgeType::Association),
            RelationshipType::Dependency => (true, EdgeType::Dependency),
            RelationshipType::Link => (false, EdgeType::Link),
        };
        if rel.line_style == LineStyle::Dashed {
            edge.dashed = true;
        }
        edge
    }

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
            // Буквальный `\n` в тексте заметки — тоже перенос строки.
            let text = note.text.replace("\\n", "\n");
            let lines: Vec<&str> = text.split('\n').collect();
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
            no_visibility: m.no_visibility,
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
            // Скрытые значок и отсеки: рендерер не рисует их, а размер уже учтён.
            for (flag, key) in [
                (c.hide_circle, "hide-circle"),
                (c.hide_fields, "hide-fields"),
                (c.hide_methods, "hide-methods"),
            ] {
                if flag {
                    properties.insert(key.to_string(), "1".to_string());
                }
            }
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
