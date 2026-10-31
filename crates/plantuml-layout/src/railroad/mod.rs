//! Раскладка железнодорожных диаграмм (`@startregex`, `@startebnf`).
//!
//! Каждый узел занимает ширину `w` и тянется от главной линии вверх на `up`
//! и вниз на `down`. Размеры сняты с эталона `feat_other_ebnf`: рамка
//! терминала 26.297 по высоте, текст с отступом 5, дуги радиуса 12, ветки
//! альтернатив через 10, начальный кружок и стрелка перед конечным.

use plantuml_ast::railroad::{EbnfDiagram, RailNode, RegexDiagram};
use plantuml_model::Rect;

use crate::text::TextMeasurer;
use crate::traits::LayoutResult;
use crate::{ElementType, LayoutElement};

const FONT: f64 = 14.0;
const SMALL_FONT: f64 = 12.0;
/// Высота рамки терминала (эталон: 26.297).
const BOX_H: f64 = 26.297;
const HALF: f64 = BOX_H / 2.0;
/// Отступ текста от рамки с каждой стороны.
const BOX_PAD: f64 = 5.0;
/// Зазор между ветками по вертикали.
const V_GAP: f64 = 10.0;
/// Радиус дуги ответвления.
const R: f64 = 12.0;
/// Зазор между соседними элементами последовательности.
const SEQ_GAP: f64 = 12.0;
/// Отступ содержимого группы от её рамки.
const GROUP_PAD: f64 = 10.0;
const INK: &str = "#181818";
/// Заливка классов символов: отличает их от литералов.
const CHARSET_FILL: &str = "#F1F1F1";
/// Базис текста в рамке: от её верха (эталон: 17.995).
const TEXT_BASE: f64 = 17.995;

/// Габариты узла относительно главной линии.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Dim {
    w: f64,
    up: f64,
    down: f64,
}

struct Canvas {
    elements: Vec<LayoutElement>,
    text: TextMeasurer,
}

/// Число без хвостовых нулей: короткий и стабильный вывод в `d` пути.
fn n(value: f64) -> String {
    let s = format!("{value:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

impl Canvas {
    fn new() -> Self {
        Self {
            elements: Vec::new(),
            text: TextMeasurer::default(),
        }
    }

    fn id(&self, kind: &str) -> String {
        format!("rail_{kind}_{}", self.elements.len())
    }

    fn tw(&self, text: &str) -> f64 {
        self.text.width(text, FONT)
    }

    fn push_rect(&mut self, rect: Rect, fill: &str, radius: f64) {
        let id = self.id("box");
        self.elements.push(
            LayoutElement::new(
                id,
                rect,
                ElementType::Rectangle {
                    label: String::new(),
                    corner_radius: radius,
                },
            )
            .with_property("fill", fill)
            .with_property("stroke-width", "0.5"),
        );
    }

    fn push_text(&mut self, x: f64, baseline: f64, text: &str, size: f64, bold: bool) {
        let id = self.id("text");
        let width = self.text.width(text, size);
        let mut element = LayoutElement::new(
            id,
            Rect::new(x, baseline - size, width, size),
            ElementType::Text {
                text: text.to_string(),
                font_size: size,
            },
        )
        .with_property("baseline", n(baseline));
        if bold {
            element = element.with_property("font-weight", "bold");
        }
        self.elements.push(element);
    }

    fn push_path(&mut self, d: String, bounds: Rect, fill: &str, width: &str) {
        let id = self.id("path");
        self.elements.push(
            LayoutElement::new(id, bounds, ElementType::Path)
                .with_property("path", d)
                .with_property("fill", fill)
                .with_property("stroke", INK)
                .with_property("stroke-width", width),
        );
    }

    /// Горизонтальный отрезок дорожки.
    fn line(&mut self, x1: f64, x2: f64, y: f64) {
        if x2 - x1 > 0.01 {
            self.push_path(
                format!("M{},{} L{},{}", n(x1), n(y), n(x2), n(y)),
                Rect::new(x1, y, x2 - x1, 0.0),
                "none",
                "1.5",
            );
        }
    }

    fn curve(&mut self, d: String, bounds: Rect) {
        self.push_path(d, bounds, "none", "1.5");
    }

    fn push_ellipse(&mut self, cx: f64, cy: f64, filled: bool) {
        let id = self.id("end");
        self.elements.push(
            LayoutElement::new(
                id,
                Rect::new(cx - 4.0, cy - 4.0, 8.0, 8.0),
                ElementType::Ellipse { label: None },
            )
            .with_property("fill", if filled { INK } else { "none" })
            .with_property("stroke", INK)
            .with_property("stroke-width", if filled { "1" } else { "2" }),
        );
    }

    // ----- габариты -----

    fn dim(&self, node: &RailNode) -> Dim {
        let boxed = |w: f64| Dim {
            w,
            up: HALF,
            down: HALF,
        };
        match node {
            RailNode::Empty => Dim {
                w: 0.0,
                up: 0.0,
                down: 0.0,
            },
            RailNode::Terminal(t) | RailNode::NonTerminal(t) | RailNode::Charset(t) => {
                boxed(self.tw(t) + 2.0 * BOX_PAD)
            }
            RailNode::Special(t) => boxed(self.tw(t) + 4.0 * BOX_PAD),
            RailNode::Label(t) => Dim {
                w: self.tw(t),
                up: 8.0,
                down: 8.0,
            },
            RailNode::Seq(items) => {
                let mut dim = Dim {
                    w: SEQ_GAP * items.len().saturating_sub(1) as f64,
                    up: 0.0,
                    down: 0.0,
                };
                for item in items {
                    let d = self.dim(item);
                    dim.w += d.w;
                    dim.up = dim.up.max(d.up);
                    dim.down = dim.down.max(d.down);
                }
                dim
            }
            RailNode::Alt(branches) => {
                let dims: Vec<Dim> = branches.iter().map(|b| self.dim(b)).collect();
                let offsets = branch_offsets(&dims);
                let w = dims.iter().fold(0.0_f64, |m, d| m.max(d.w));
                let last = dims.len().saturating_sub(1);
                Dim {
                    w: w + 4.0 * R,
                    up: dims.first().map_or(0.0, |d| d.up),
                    down: offsets.last().copied().unwrap_or(0.0)
                        + dims.get(last).map_or(0.0, |d| d.down),
                }
            }
            RailNode::Loop { body, label } => {
                let d = self.dim(body);
                let back = (d.down + V_GAP).max(2.0 * R);
                Dim {
                    w: d.w + 2.0 * R,
                    up: d.up,
                    down: back + if label.is_some() { 18.0 } else { 0.0 },
                }
            }
            RailNode::Group { body, label } => {
                let d = self.dim(body);
                Dim {
                    w: d.w + 4.0 * GROUP_PAD,
                    up: d.up + GROUP_PAD + if label.is_some() { 16.0 } else { 0.0 },
                    down: d.down + GROUP_PAD,
                }
            }
        }
    }

    // ----- рисование -----

    fn draw(&mut self, node: &RailNode, x: f64, cy: f64) {
        let dim = self.dim(node);
        match node {
            RailNode::Empty => {}
            RailNode::Terminal(t) => self.draw_box(t, x, cy, dim.w, "none", 0.0),
            RailNode::NonTerminal(t) => self.draw_box(t, x, cy, dim.w, "none", 9.0),
            RailNode::Charset(t) => self.draw_box(t, x, cy, dim.w, CHARSET_FILL, 0.0),
            RailNode::Special(t) => {
                let (top, bottom, r) = (cy - HALF, cy + HALF, x + dim.w);
                let d = format!(
                    "M{},{} L{},{} L{},{} L{},{} L{},{} L{},{} Z",
                    n(x),
                    n(cy),
                    n(x + 10.0),
                    n(top),
                    n(r - 10.0),
                    n(top),
                    n(r),
                    n(cy),
                    n(r - 10.0),
                    n(bottom),
                    n(x + 10.0),
                    n(bottom)
                );
                self.push_path(d, Rect::new(x, top, dim.w, BOX_H), "none", "0.5");
                self.push_text(x + 2.0 * BOX_PAD, cy - HALF + TEXT_BASE, t, FONT, false);
            }
            RailNode::Label(t) => self.push_text(x, cy + 4.847, t, SMALL_FONT, false),
            RailNode::Seq(items) => {
                let mut cursor = x;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        self.line(cursor - SEQ_GAP, cursor, cy);
                    }
                    self.draw(item, cursor, cy);
                    cursor += self.dim(item).w + SEQ_GAP;
                }
            }
            RailNode::Alt(branches) => self.draw_alt(branches, x, cy, dim),
            RailNode::Loop { body, label } => self.draw_loop(body, label.as_deref(), x, cy, dim),
            RailNode::Group { body, label } => {
                let top = cy - dim.up;
                self.push_rect(Rect::new(x, top, dim.w, dim.up + dim.down), "none", 8.0);
                if let Some(label) = label {
                    self.push_text(x + 6.0, top + 13.0, label, SMALL_FONT, false);
                }
                let inner = self.dim(body);
                let inner_x = x + 2.0 * GROUP_PAD;
                self.line(x, inner_x, cy);
                self.draw(body, inner_x, cy);
                self.line(inner_x + inner.w, x + dim.w, cy);
            }
        }
    }

    fn draw_box(&mut self, text: &str, x: f64, cy: f64, w: f64, fill: &str, radius: f64) {
        let top = cy - HALF;
        self.push_rect(Rect::new(x, top, w, BOX_H), fill, radius);
        self.push_text(x + BOX_PAD, top + TEXT_BASE, text, FONT, false);
    }

    fn draw_alt(&mut self, branches: &[RailNode], x: f64, cy: f64, dim: Dim) {
        let dims: Vec<Dim> = branches.iter().map(|b| self.dim(b)).collect();
        let offsets = branch_offsets(&dims);
        let (xl, xr) = (x, x + dim.w);
        let (inner_l, inner_r) = (xl + 2.0 * R, xr - 2.0 * R);
        for (i, branch) in branches.iter().enumerate() {
            let by = cy + offsets[i];
            self.draw(branch, inner_l, by);
            let end = inner_l + dims[i].w;
            if i == 0 {
                self.line(xl, inner_l, cy);
                self.line(end, xr, cy);
                continue;
            }
            self.line(end, inner_r, by);
            let bounds = Rect::new(xl, cy, dim.w, by - cy);
            self.curve(
                format!(
                    "M{},{} C{},{} {},{} {},{} L{},{} C{},{} {},{} {},{}",
                    n(xl),
                    n(cy),
                    n(xl + 9.0),
                    n(cy),
                    n(xl + R),
                    n(cy + 3.0),
                    n(xl + R),
                    n(cy + R),
                    n(xl + R),
                    n(by - R),
                    n(xl + R),
                    n(by - 3.0),
                    n(xl + R + 3.0),
                    n(by),
                    n(inner_l),
                    n(by)
                ),
                bounds,
            );
            self.curve(
                format!(
                    "M{},{} C{},{} {},{} {},{} L{},{} C{},{} {},{} {},{}",
                    n(inner_r),
                    n(by),
                    n(inner_r + 9.0),
                    n(by),
                    n(xr - R),
                    n(by - 3.0),
                    n(xr - R),
                    n(by - R),
                    n(xr - R),
                    n(cy + R),
                    n(xr - R),
                    n(cy + 3.0),
                    n(xr - 9.0),
                    n(cy),
                    n(xr),
                    n(cy)
                ),
                bounds,
            );
        }
    }

    fn draw_loop(&mut self, body: &RailNode, label: Option<&str>, x: f64, cy: f64, dim: Dim) {
        let inner = self.dim(body);
        let (xs, xe) = (x + R, x + R + inner.w);
        let back = (inner.down + V_GAP).max(2.0 * R);
        let yb = cy + back;
        self.line(x, xs, cy);
        self.draw(body, xs, cy);
        self.line(xe, x + dim.w, cy);
        // Обратная дорожка под телом: вправо-вниз, назад влево, вверх ко входу.
        self.curve(
            format!(
                "M{xe},{cy} C{},{cy} {},{} {},{} L{},{} C{},{} {},{yb} {xe},{yb} \
                 L{xs},{yb} C{},{yb} {},{} {},{} L{},{} C{},{} {},{cy} {xs},{cy}",
                n(xe + 9.0),
                n(xe + R),
                n(cy + 3.0),
                n(xe + R),
                n(cy + R),
                n(xe + R),
                n(yb - R),
                n(xe + R),
                n(yb - 3.0),
                n(xe + 9.0),
                n(xs - 9.0),
                n(xs - R),
                n(yb - 3.0),
                n(xs - R),
                n(yb - R),
                n(xs - R),
                n(cy + R),
                n(xs - R),
                n(cy + 3.0),
                n(xs - 9.0),
                xe = n(xe),
                xs = n(xs),
                cy = n(cy),
                yb = n(yb),
            ),
            Rect::new(x, cy, dim.w, back),
        );
        // Стрелка на обратной дорожке показывает направление повтора.
        let mx = (xs + xe) / 2.0;
        self.push_path(
            format!(
                "M{},{} L{},{} L{},{} Z",
                n(mx + 3.0),
                n(yb - 3.0),
                n(mx - 3.0),
                n(yb),
                n(mx + 3.0),
                n(yb + 3.0)
            ),
            Rect::new(mx - 3.0, yb - 3.0, 6.0, 6.0),
            INK,
            "1",
        );
        if let Some(label) = label {
            let w = self.text.width(label, SMALL_FONT);
            self.push_text(mx - w / 2.0, yb + 15.0, label, SMALL_FONT, false);
        }
    }
}

/// Смещение главной линии каждой ветки от линии первой.
fn branch_offsets(dims: &[Dim]) -> Vec<f64> {
    let mut offsets = Vec::with_capacity(dims.len());
    let mut y = 0.0;
    for (i, d) in dims.iter().enumerate() {
        if i > 0 {
            y += (dims[i - 1].down + V_GAP + d.up).max(2.0 * R);
        }
        offsets.push(y);
    }
    offsets
}

fn finish(canvas: Canvas, width: f64, height: f64) -> LayoutResult {
    LayoutResult {
        elements: canvas.elements,
        bounds: Rect::new(0.0, 0.0, width.ceil(), height.ceil()),
    }
}

/// Раскладывает диаграмму `@startregex`.
pub fn layout_regex(diagram: &RegexDiagram) -> LayoutResult {
    const PAD: f64 = 15.0;
    let mut canvas = Canvas::new();
    let (mut y, mut width) = (PAD, 0.0_f64);
    for pattern in &diagram.patterns {
        let dim = canvas.dim(pattern);
        let cy = y + dim.up;
        canvas.draw(pattern, PAD, cy);
        width = width.max(PAD + dim.w + PAD);
        y = cy + dim.down + PAD;
    }
    finish(canvas, width, y)
}

/// Раскладывает диаграмму `@startebnf`.
pub fn layout_ebnf(diagram: &EbnfDiagram) -> LayoutResult {
    const PAD: f64 = 10.0;
    const NAME_H: f64 = 16.297;
    const NOTE_H: f64 = 16.0;
    // Дорожка: кружок 8, провод 22 до тела, 26 после (в нём стрелка), кружок 8.
    const LEAD: f64 = 22.0;
    const TRAIL: f64 = 26.0;
    let mut canvas = Canvas::new();
    let (mut y, mut width) = (PAD, 0.0_f64);
    for rule in &diagram.rules {
        canvas.push_text(PAD, y + 12.995, &rule.name, FONT, true);
        width = width.max(PAD + canvas.text.width_bold(&rule.name, FONT));
        y += NAME_H;
        for note in &rule.notes {
            canvas.push_text(PAD, y + 12.0, note, SMALL_FONT, false);
            width = width.max(PAD + canvas.text.width(note, SMALL_FONT));
            y += NOTE_H;
        }
        y += PAD;
        let dim = canvas.dim(&rule.body);
        let cy = y + dim.up;
        let body_x = PAD + 8.0 + LEAD;
        let body_end = body_x + dim.w;
        let end_cx = body_end + TRAIL + 4.0;
        canvas.push_ellipse(PAD + 4.0, cy, false);
        canvas.line(PAD + 8.0, body_x, cy);
        canvas.draw(&rule.body, body_x, cy);
        canvas.line(body_end, end_cx - 4.0, cy);
        canvas.push_ellipse(end_cx, cy, true);
        let ax = body_end + 10.0;
        canvas.push_path(
            format!(
                "M{},{} L{},{} L{},{} L{},{} Z",
                n(ax),
                n(cy),
                n(ax),
                n(cy - 3.0),
                n(ax + 6.0),
                n(cy),
                n(ax),
                n(cy + 3.0)
            ),
            Rect::new(ax, cy - 3.0, 6.0, 6.0),
            INK,
            "1",
        );
        width = width.max(end_cx + 4.0 + 6.2);
        y = cy + dim.down + 15.0;
    }
    for note in &diagram.notes {
        canvas.push_text(PAD, y + 12.0, note, SMALL_FONT, false);
        width = width.max(PAD + canvas.text.width(note, SMALL_FONT));
        y += NOTE_H;
    }
    // Снизу эталон оставляет 25.1 от последней рамки: 15 между правилами и ещё 10.
    finish(canvas, width, y + PAD)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn svgish(r: &LayoutResult) -> Vec<&LayoutElement> {
        r.elements.iter().collect()
    }

    #[test]
    fn regex_literal_matches_reference_geometry() {
        let d = RegexDiagram {
            patterns: vec![RailNode::Terminal("abc".into())],
            metadata: Default::default(),
        };
        let r = layout_regex(&d);
        // Ширина текста — по таблице глифов, на доли пикселя от эталонных 25.163.
        assert!((65.0..=66.0).contains(&r.bounds.width));
        assert_eq!(r.bounds.height, 57.0);
        let b = svgish(&r)[0].bounds;
        assert_eq!((b.x, b.y), (15.0, 15.0));
        assert!((b.height - 26.297).abs() < 1e-9);
    }

    #[test]
    fn alternatives_stack_below_main_line() {
        let c = Canvas::new();
        let alt = RailNode::Alt(vec![
            RailNode::Terminal("a1".into()),
            RailNode::Terminal("b1".into()),
        ]);
        let dim = c.dim(&alt);
        assert!((dim.down - (HALF + V_GAP + BOX_H)).abs() < 1e-9);
        assert!((dim.w - (c.tw("b1").max(c.tw("a1")) + 10.0 + 48.0)).abs() < 1e-9);
    }
}
