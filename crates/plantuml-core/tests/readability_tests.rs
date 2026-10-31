//! Читаемость графовых диаграмм по всему корпусу: узлы не пересекаются, всё лежит внутри
//! холста, рамка охватывает своё содержимое, связь подходит к своей фигуре.

use std::path::PathBuf;

use plantuml_core::{layout_for_checks, Diagram, RenderOptions};
use plantuml_layout::{ElementType, LayoutElement, LayoutResult};

const TOLERANCE: f64 = 0.5;

fn corpus() -> Vec<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/corpus");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(root)
        .expect("корпус на месте")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().is_some_and(|e| e == "puml"))
        .collect();
    paths.sort();
    paths
}

fn is_graph(diagram: &Diagram) -> bool {
    matches!(
        diagram,
        Diagram::Class(_)
            | Diagram::State(_)
            | Diagram::Component(_)
            | Diagram::UseCase(_)
            | Diagram::Object(_)
            | Diagram::Er(_)
    )
}

fn is_frame(element: &LayoutElement) -> bool {
    matches!(
        element.element_type,
        ElementType::Group { .. } | ElementType::System { .. } | ElementType::CompositeState { .. }
    )
}

fn is_node(element: &LayoutElement) -> bool {
    // Значок и маркеры сущности ER лежат на её же заливке.
    let part = element.id.ends_with("_spot") || element.id.contains("_marker");
    !part
        && !is_frame(element)
        && !matches!(
            element.element_type,
            ElementType::Edge { .. } | ElementType::Text { .. } | ElementType::Note { .. }
        )
}

fn overlap(a: &LayoutElement, b: &LayoutElement) -> bool {
    let (a, b) = (a.bounds, b.bounds);
    a.x + TOLERANCE < b.x + b.width
        && b.x + TOLERANCE < a.x + a.width
        && a.y + TOLERANCE < b.y + b.height
        && b.y + TOLERANCE < a.y + a.height
}

fn check(result: &LayoutResult) -> Vec<String> {
    let mut problems = Vec::new();
    let nodes: Vec<&LayoutElement> = result.elements.iter().filter(|e| is_node(e)).collect();
    for (index, a) in nodes.iter().enumerate() {
        for b in &nodes[index + 1..] {
            if overlap(a, b) {
                problems.push(format!("узлы пересекаются: {} и {}", a.id, b.id));
            }
        }
    }

    let canvas = result.bounds;
    for element in result
        .elements
        .iter()
        .filter(|e| !matches!(e.element_type, ElementType::Edge { .. }))
    {
        let b = element.bounds;
        if b.x < canvas.x - 1.0
            || b.y < canvas.y - 1.0
            || b.x + b.width > canvas.x + canvas.width + 1.0
            || b.y + b.height > canvas.y + canvas.height + 1.0
        {
            problems.push(format!("вне холста: {} {b:?} на {canvas:?}", element.id));
        }
    }

    for frame in result.elements.iter().filter(|e| is_frame(e)) {
        let f = frame.bounds;
        for node in &nodes {
            let b = node.bounds;
            let (cx, cy) = (b.x + b.width / 2.0, b.y + b.height / 2.0);
            let center_inside = cx > f.x && cx < f.x + f.width && cy > f.y && cy < f.y + f.height;
            let fully_inside = b.x >= f.x - 1.0
                && b.y >= f.y - 1.0
                && b.x + b.width <= f.x + f.width + 1.0
                && b.y + b.height <= f.y + f.height + 1.0;
            let seated = node.properties.get("shape").is_some_and(|s| s == "port");
            if center_inside && !fully_inside && !seated {
                problems.push(format!("{} выходит за рамку {}", node.id, frame.id));
            }
        }
    }

    let shapes: Vec<&LayoutElement> = result
        .elements
        .iter()
        .filter(|e| is_node(e) || is_frame(e) || matches!(e.element_type, ElementType::Note { .. }))
        .collect();
    for edge in &result.elements {
        let ElementType::Edge { points, .. } = &edge.element_type else {
            continue;
        };
        // Заметка на связи подходит к середине связи, а не к фигуре.
        if edge.id.starts_with("note_edge") {
            continue;
        }
        let (Some(first), Some(last)) = (points.first(), points.last()) else {
            continue;
        };
        for end in [first, last] {
            let near = shapes.iter().any(|s| {
                let b = s.bounds;
                end.x >= b.x - 4.0
                    && end.x <= b.x + b.width + 4.0
                    && end.y >= b.y - 4.0
                    && end.y <= b.y + b.height + 4.0
            });
            if !near {
                problems.push(format!(
                    "связь {} висит в воздухе у ({}, {})",
                    edge.id, end.x, end.y
                ));
            }
        }
    }
    problems
}

#[test]
fn corpus_graph_diagrams_are_readable() {
    let mut failures = Vec::new();
    let mut checked = 0;
    for path in corpus() {
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok((diagram, result)) = layout_for_checks(&source, &RenderOptions::default()) else {
            continue;
        };
        if !is_graph(&diagram) {
            continue;
        }
        checked += 1;
        let name = path.file_stem().map(|s| s.to_string_lossy().into_owned());
        for problem in check(&result) {
            failures.push(format!("{}: {problem}", name.clone().unwrap_or_default()));
        }
    }
    println!("проверено графовых схем: {checked}");
    assert!(
        checked > 80,
        "корпус должен покрывать графовые схемы: {checked}"
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
