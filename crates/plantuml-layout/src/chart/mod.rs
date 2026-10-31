//! Раскладка `@startchart`: оси, столбцы, линии, области и легенда.
//!
//! Геометрия снята с эталона `feat_other_chart`: поле графика 400x300 с
//! началом в (70, 60), риски по 5, подписи рисок на 22.995 ниже оси,
//! столбец занимает 60% ячейки категории, ось по умолчанию 0..100.

use plantuml_ast::chart::{ChartDiagram, LegendPosition, Series, SeriesKind};
use plantuml_model::Rect;

use crate::text::TextMeasurer;
use crate::traits::LayoutResult;
use crate::{ElementType, LayoutElement};

const FONT: f64 = 14.0;
const SMALL_FONT: f64 = 12.0;
const INK: &str = "#181818";
const PLOT_W: f64 = 400.0;
const PLOT_H: f64 = 300.0;
const TOP: f64 = 60.0;
const MIN_LEFT: f64 = 70.0;
const RIGHT_PAD: f64 = 31.0;
const BOTTOM_PAD: f64 = 71.0;
/// Доля ячейки категории, занятая столбцами.
const BAR_SHARE: f64 = 0.6;
/// Минимальная ширина ячейки категории: подписи не должны слипаться.
const MIN_SLOT: f64 = 50.0;
const FILLS: [&str; 6] = [
    "#F1F1F1", "#A6CEE3", "#B2DF8A", "#FDBF6F", "#CAB2D6", "#FB9A99",
];
const STROKES: [&str; 6] = [
    "#181818", "#1F78B4", "#33A02C", "#E31A1C", "#FF7F00", "#6A3D9A",
];
const LEGEND_ROW: f64 = 20.0;
const SWATCH: f64 = 12.0;

fn n(value: f64) -> String {
    let s = format!("{value:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Подпись числа: целые без дробной части.
fn fmt_value(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 {
        format!("{}", v.round() as i64)
    } else {
        n(v)
    }
}

/// Верхняя граница шкалы: 1, 2, 5 на степень десяти.
fn nice_ceil(v: f64) -> f64 {
    let magnitude = 10f64.powf(v.log10().floor());
    [1.0, 2.0, 5.0, 10.0]
        .iter()
        .map(|m| m * magnitude)
        .find(|c| *c >= v)
        .unwrap_or(v)
}

struct Scale {
    min: f64,
    max: f64,
    step: f64,
}

impl Scale {
    fn of(chart: &ChartDiagram) -> Self {
        let values = chart.series.iter().flat_map(|s| s.values.iter().copied());
        let (mut lo, mut hi) = values.fold((0.0_f64, f64::MIN), |(a, b), v| (a.min(v), b.max(v)));
        if chart.stacked {
            let slots = chart
                .series
                .iter()
                .map(|s| s.values.len())
                .max()
                .unwrap_or(0);
            for i in 0..slots {
                let total: f64 = chart
                    .series
                    .iter()
                    .filter(|s| s.kind == SeriesKind::Bar)
                    .filter_map(|s| s.values.get(i))
                    .filter(|v| **v > 0.0)
                    .sum();
                hi = hi.max(total);
            }
        }
        let (min, max) = chart.v_axis.range.unwrap_or_else(|| {
            // Без явной оси шкала 0..100, как у PlantUML, и шире только если данные не влезают.
            (lo.min(0.0), if hi > 100.0 { nice_ceil(hi) } else { 100.0 })
        });
        lo = min;
        let max = if max > lo { max } else { lo + 1.0 };
        let step = chart
            .v_axis
            .spacing
            .filter(|s| *s > 0.0)
            .unwrap_or_else(|| nice_ceil((max - lo) / 5.0));
        Self { min: lo, max, step }
    }

    fn y(&self, bottom: f64, v: f64) -> f64 {
        let clamped = v.clamp(self.min, self.max);
        bottom - (clamped - self.min) / (self.max - self.min) * PLOT_H
    }
}

struct Out {
    elements: Vec<LayoutElement>,
    text: TextMeasurer,
}

impl Out {
    fn id(&self, kind: &str) -> String {
        format!("chart_{kind}_{}", self.elements.len())
    }

    fn path(&mut self, d: String, bounds: Rect, fill: &str, stroke: &str, width: &str) {
        let id = self.id("path");
        self.elements.push(
            LayoutElement::new(id, bounds, ElementType::Path)
                .with_property("path", d)
                .with_property("fill", fill)
                .with_property("stroke", stroke)
                .with_property("stroke-width", width),
        );
    }

    fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.path(
            format!("M{},{} L{},{}", n(x1), n(y1), n(x2), n(y2)),
            Rect::new(x1.min(x2), y1.min(y2), (x2 - x1).abs(), (y2 - y1).abs()),
            "none",
            INK,
            "0.5",
        );
    }

    fn rect(&mut self, rect: Rect, fill: &str) {
        let id = self.id("rect");
        self.elements.push(
            LayoutElement::new(
                id,
                rect,
                ElementType::Rectangle {
                    label: String::new(),
                    corner_radius: 0.0,
                },
            )
            .with_property("fill", fill)
            .with_property("stroke-width", "0.5"),
        );
    }

    fn text(&mut self, x: f64, baseline: f64, text: &str, size: f64) {
        let id = self.id("text");
        let width = self.text.width(text, size);
        self.elements.push(
            LayoutElement::new(
                id,
                Rect::new(x, baseline - size, width, size),
                ElementType::Text {
                    text: text.to_string(),
                    font_size: size,
                },
            )
            .with_property("baseline", n(baseline)),
        );
    }

    /// Текст, центрированный по `cx`.
    fn centered(&mut self, cx: f64, baseline: f64, text: &str, size: f64) {
        let w = self.text.width(text, size);
        self.text(cx - w / 2.0, baseline, text, size);
    }

    fn dot(&mut self, cx: f64, cy: f64, color: &str) {
        let id = self.id("dot");
        self.elements.push(
            LayoutElement::new(
                id,
                Rect::new(cx - 3.0, cy - 3.0, 6.0, 6.0),
                ElementType::Ellipse { label: None },
            )
            .with_property("fill", color)
            .with_property("stroke", color)
            .with_property("stroke-width", "1"),
        );
    }
}

fn fill_of(series: &Series, index: usize) -> String {
    series
        .color
        .clone()
        .unwrap_or_else(|| FILLS[index % FILLS.len()].to_string())
}

fn stroke_of(series: &Series, index: usize) -> String {
    series
        .color
        .clone()
        .unwrap_or_else(|| STROKES[index % STROKES.len()].to_string())
}

/// Цвет образца легенды: тот же, которым ряд нарисован.
fn swatch_of(series: &Series, index: usize) -> String {
    match series.kind {
        SeriesKind::Line => stroke_of(series, index),
        _ => fill_of(series, index),
    }
}

/// Раскладывает диаграмму `@startchart`.
pub fn layout_chart(chart: &ChartDiagram) -> LayoutResult {
    let mut out = Out {
        elements: Vec::new(),
        text: TextMeasurer::default(),
    };
    let scale = Scale::of(chart);
    let slots = chart
        .h_axis
        .categories
        .len()
        .max(
            chart
                .series
                .iter()
                .map(|s| s.values.len())
                .max()
                .unwrap_or(0),
        )
        .max(1);
    let plot_w = PLOT_W.max(slots as f64 * MIN_SLOT);
    let slot = plot_w / slots as f64;

    // Подписи вертикальной оси определяют левое поле.
    let ticks: Vec<f64> = {
        let mut v = Vec::new();
        let mut t = scale.min;
        while t <= scale.max + scale.step * 1e-6 && v.len() < 200 {
            v.push(t);
            t += scale.step;
        }
        v
    };
    let label_w = ticks
        .iter()
        .map(|t| out.text.width(&fmt_value(*t), FONT))
        .fold(0.0_f64, f64::max);

    // Легенда: размеры по именам рядов.
    let names: Vec<&str> = chart.series.iter().map(|s| s.name.as_str()).collect();
    let name_w = names
        .iter()
        .map(|s| out.text.width(s, FONT))
        .fold(0.0_f64, f64::max);
    let item_w = SWATCH + 6.0 + name_w;
    let side_w = if matches!(
        chart.legend,
        Some(LegendPosition::Left | LegendPosition::Right)
    ) {
        item_w + 30.0
    } else {
        0.0
    };
    let row_h = if matches!(
        chart.legend,
        Some(LegendPosition::Top | LegendPosition::Bottom)
    ) {
        LEGEND_ROW + 10.0
    } else {
        0.0
    };

    let left_axis = MIN_LEFT.max(label_w + 40.0);
    let left_shift = if chart.legend == Some(LegendPosition::Left) {
        side_w
    } else {
        0.0
    };
    let x0 = left_shift + left_axis;
    let top = TOP
        + if chart.legend == Some(LegendPosition::Top) {
            row_h
        } else {
            0.0
        };
    let bottom = top + PLOT_H;
    let right = x0 + plot_w;

    // Оси и риски.
    out.line(x0, top, x0, bottom);
    for t in &ticks {
        let y = scale.y(bottom, *t);
        out.line(x0, y, x0 - 5.0, y);
        let label = fmt_value(*t);
        let w = out.text.width(&label, FONT);
        out.text(x0 - 10.0 - w, y + 4.847, &label, FONT);
    }
    out.line(x0, bottom, right, bottom);
    for i in 0..slots {
        let cx = x0 + (i as f64 + 0.5) * slot;
        out.line(cx, bottom, cx, bottom + 5.0);
        let label = chart
            .h_axis
            .categories
            .get(i)
            .cloned()
            .unwrap_or_else(|| (i + 1).to_string());
        out.centered(cx, bottom + 22.995, &label, FONT);
    }
    if let Some(title) = &chart.v_axis.title {
        let w = out.text.width(title, FONT);
        out.text((x0 - w / 2.0).max(5.0), top - 15.0, title, FONT);
    }
    if let Some(title) = &chart.h_axis.title {
        out.centered(x0 + plot_w / 2.0, bottom + 50.0, title, FONT);
    }

    // Области под столбцами и линиями, затем столбцы, затем линии.
    let zero = scale.y(bottom, 0.0);
    for (i, s) in chart.series.iter().enumerate() {
        if s.kind != SeriesKind::Area || s.values.is_empty() {
            continue;
        }
        let points: Vec<(f64, f64)> = s
            .values
            .iter()
            .enumerate()
            .map(|(k, v)| (x0 + (k as f64 + 0.5) * slot, scale.y(bottom, *v)))
            .collect();
        let mut d = format!("M{},{}", n(points[0].0), n(zero));
        for (px, py) in &points {
            d.push_str(&format!(" L{},{}", n(*px), n(*py)));
        }
        d.push_str(&format!(
            " L{},{} Z",
            n(points[points.len() - 1].0),
            n(zero)
        ));
        out.path(
            d,
            Rect::new(x0, top, plot_w, PLOT_H),
            &fill_of(s, i),
            INK,
            "0.5",
        );
    }

    let bars: Vec<(usize, &Series)> = chart
        .series
        .iter()
        .enumerate()
        .filter(|(_, s)| s.kind == SeriesKind::Bar)
        .collect();
    let group_w = slot * BAR_SHARE;
    let mut stack = vec![0.0_f64; slots];
    for (rank, (i, s)) in bars.iter().enumerate() {
        for (k, v) in s.values.iter().enumerate().take(slots) {
            let cx = x0 + (k as f64 + 0.5) * slot;
            let (bx, bw) = if chart.stacked {
                (cx - group_w / 2.0, group_w)
            } else {
                let w = group_w / bars.len() as f64;
                (cx - group_w / 2.0 + rank as f64 * w, w)
            };
            let (from, to) = if chart.stacked {
                let base = stack[k];
                stack[k] += v;
                (base, base + v)
            } else {
                (0.0, *v)
            };
            let (y1, y2) = (scale.y(bottom, from), scale.y(bottom, to));
            out.rect(
                Rect::new(bx, y1.min(y2), bw, (y1 - y2).abs()),
                &fill_of(s, *i),
            );
            if s.labels {
                out.centered(bx + bw / 2.0, y1.min(y2) - 5.0, &fmt_value(*v), SMALL_FONT);
            }
        }
    }

    for (i, s) in chart.series.iter().enumerate() {
        if s.kind != SeriesKind::Line || s.values.is_empty() {
            continue;
        }
        let color = stroke_of(s, i);
        let points: Vec<(f64, f64)> = s
            .values
            .iter()
            .enumerate()
            .map(|(k, v)| (x0 + (k as f64 + 0.5) * slot, scale.y(bottom, *v)))
            .collect();
        let d = points
            .iter()
            .enumerate()
            .map(|(k, (px, py))| {
                format!("{}{},{}", if k == 0 { "M" } else { " L" }, n(*px), n(*py))
            })
            .collect::<String>();
        out.path(d, Rect::new(x0, top, plot_w, PLOT_H), "none", &color, "1.5");
        for ((px, py), v) in points.iter().zip(&s.values) {
            out.dot(*px, *py, &color);
            if s.labels {
                out.centered(*px, py - 8.0, &fmt_value(*v), SMALL_FONT);
            }
        }
    }
    for (i, s) in chart.series.iter().enumerate() {
        if s.kind == SeriesKind::Area && s.labels {
            for (k, v) in s.values.iter().enumerate() {
                let cx = x0 + (k as f64 + 0.5) * slot;
                out.centered(cx, scale.y(bottom, *v) - 5.0, &fmt_value(*v), SMALL_FONT);
            }
            let _ = i;
        }
    }

    // Легенда.
    let mut width = right + RIGHT_PAD;
    let mut height = bottom + BOTTOM_PAD;
    if let Some(pos) = chart.legend {
        let draw_item = |out: &mut Out, x: f64, y: f64, i: usize, s: &Series| {
            out.rect(Rect::new(x, y, SWATCH, SWATCH), &swatch_of(s, i));
            out.text(x + SWATCH + 6.0, y + 11.0, &s.name, FONT);
        };
        match pos {
            LegendPosition::Right | LegendPosition::Left => {
                let x = if pos == LegendPosition::Right {
                    right + 20.0
                } else {
                    20.0
                };
                let mut y = top;
                for (i, s) in chart.series.iter().enumerate() {
                    draw_item(&mut out, x, y, i, s);
                    y += LEGEND_ROW;
                }
                if pos == LegendPosition::Right {
                    width = right + 20.0 + item_w + RIGHT_PAD;
                }
            }
            LegendPosition::Top | LegendPosition::Bottom => {
                let total: f64 = chart
                    .series
                    .iter()
                    .map(|s| SWATCH + 6.0 + out.text.width(&s.name, FONT) + 20.0)
                    .sum::<f64>()
                    - 20.0;
                let mut x = x0 + (plot_w - total) / 2.0;
                let y = if pos == LegendPosition::Top {
                    top - row_h
                } else {
                    height
                };
                for (i, s) in chart.series.iter().enumerate() {
                    draw_item(&mut out, x, y, i, s);
                    x += SWATCH + 6.0 + out.text.width(&s.name, FONT) + 20.0;
                }
                if pos == LegendPosition::Bottom {
                    height += row_h;
                }
            }
        }
    }

    LayoutResult {
        elements: out.elements,
        bounds: Rect::new(0.0, 0.0, width.ceil(), height.ceil()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_ast::chart::{HAxis, Series};

    fn reference() -> ChartDiagram {
        ChartDiagram {
            h_axis: HAxis {
                title: None,
                categories: vec!["a1".into(), "b1".into()],
            },
            series: vec![Series {
                kind: SeriesKind::Bar,
                name: "s1".into(),
                values: vec![1.0, 2.0],
                color: None,
                labels: false,
            }],
            ..ChartDiagram::default()
        }
    }

    #[test]
    fn canvas_and_bars_match_reference() {
        let r = layout_chart(&reference());
        assert_eq!((r.bounds.width, r.bounds.height), (501.0, 431.0));
        let bars: Vec<_> = r
            .elements
            .iter()
            .filter(|e| matches!(e.element_type, ElementType::Rectangle { .. }))
            .collect();
        assert_eq!(bars.len(), 2);
        assert_eq!((bars[0].bounds.x, bars[0].bounds.width), (110.0, 120.0));
        assert_eq!((bars[1].bounds.y, bars[1].bounds.height), (354.0, 6.0));
    }

    #[test]
    fn legend_and_scale_follow_data() {
        let mut chart = reference();
        chart.series[0].values = vec![300.0, 450.0];
        chart.legend = Some(LegendPosition::Right);
        let r = layout_chart(&chart);
        assert!(r.elements.iter().any(|e| e.text.is_none()
            && matches!(&e.element_type, ElementType::Text { text, .. } if text == "s1")));
        assert!(r
            .elements
            .iter()
            .any(|e| matches!(&e.element_type, ElementType::Text { text, .. } if text == "500")));
        assert!(r.bounds.width > 501.0);
    }
}
