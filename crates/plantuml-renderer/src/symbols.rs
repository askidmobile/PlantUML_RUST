//! Фигуры диаграмм class, component, deployment и use case.
//!
//! Геометрия снята с эталонов PlantUML (`feat_component_deployment`,
//! `feat_component_package_node_cloud`, `real_component_real`): здесь только
//! контуры, текст кладёт рендерер по возвращённой точке.

use svg::node::element::{Ellipse, Line, Path, Polygon, Rectangle};
use svg::Node;

use crate::svg_renderer::fmt;
use crate::Rect;

/// Узел SVG, готовый к добавлению в группу.
pub type Shape = Box<dyn Node>;

/// Краски фигуры.
pub struct Ink<'a> {
    pub fill: &'a str,
    pub stroke: &'a str,
    pub width: f64,
}

/// Где стоит блок подписи: центр по осям.
pub struct TextSpot {
    pub cx: f64,
    pub cy: f64,
}

/// Радиус скругления углов рамок и тел фигур.
const ROUND: f64 = 2.5;
/// Скос объёмной рамки node.
const NODE_BEVEL: f64 = 10.0;
/// Высота выпуклости облака и цилиндра.
const CAP: f64 = 10.0;

fn line(x1: f64, y1: f64, x2: f64, y2: f64, ink: &Ink) -> Shape {
    Box::new(
        Line::new()
            .set("x1", fmt(x1))
            .set("y1", fmt(y1))
            .set("x2", fmt(x2))
            .set("y2", fmt(y2))
            .set("stroke", ink.stroke)
            .set("stroke-width", fmt(ink.width)),
    )
}

fn outline(d: String, fill: &str, ink: &Ink) -> Shape {
    Box::new(
        Path::new()
            .set("d", d)
            .set("fill", fill)
            .set("stroke", ink.stroke)
            .set("stroke-width", fmt(ink.width)),
    )
}

fn round_rect(x: f64, y: f64, w: f64, h: f64, fill: &str, ink: &Ink) -> Shape {
    Box::new(
        Rectangle::new()
            .set("x", fmt(x))
            .set("y", fmt(y))
            .set("width", fmt(w))
            .set("height", fmt(h))
            .set("rx", ROUND)
            .set("ry", ROUND)
            .set("fill", fill)
            .set("stroke", ink.stroke)
            .set("stroke-width", fmt(ink.width)),
    )
}

fn polygon(points: &[(f64, f64)], fill: &str, ink: &Ink) -> Shape {
    let list: Vec<String> = points
        .iter()
        .map(|(x, y)| format!("{},{}", fmt(*x), fmt(*y)))
        .collect();
    Box::new(
        Polygon::new()
            .set("points", list.join(" "))
            .set("fill", fill)
            .set("stroke", ink.stroke)
            .set("stroke-width", fmt(ink.width))
            .set("stroke-linejoin", "miter"),
    )
}

/// Цилиндр: боковины с верхней крышкой и дуга ободка.
fn cylinder(b: &Rect, cap: f64, fill: &str, ink: &Ink) -> Vec<Shape> {
    let (l, r) = (b.x, b.x + b.width);
    let (cx, top, bottom) = (b.x + b.width / 2.0, b.y, b.y + b.height);
    let body = format!(
        "M{l},{a} C{l},{t} {cx},{t} {cx},{t} C{cx},{t} {r},{t} {r},{a} L{r},{c} \
         C{r},{bt} {cx},{bt} {cx},{bt} C{cx},{bt} {l},{bt} {l},{c} L{l},{a}",
        l = fmt(l),
        r = fmt(r),
        cx = fmt(cx),
        a = fmt(top + cap),
        t = fmt(top),
        c = fmt(bottom - cap),
        bt = fmt(bottom),
    );
    let rim = format!(
        "M{l},{a} C{l},{m} {cx},{m} {cx},{m} C{cx},{m} {r},{m} {r},{a}",
        l = fmt(l),
        r = fmt(r),
        cx = fmt(cx),
        a = fmt(top + cap),
        m = fmt(top + cap * 2.0),
    );
    vec![outline(body, fill, ink), outline(rim, "none", ink)]
}

/// Цилиндр базы данных по границам фигуры.
pub fn database(b: &Rect, ink: &Ink) -> Vec<Shape> {
    cylinder(b, CAP, ink.fill, ink)
}

/// Облако: цепочка дуг по периметру, выпуклых наружу.
fn cloud_path(b: &Rect) -> String {
    let inset = 5.0;
    let (x0, y0) = (b.x + inset, b.y + inset);
    let (x1, y1) = (b.x + b.width - inset, b.y + b.height - inset);
    let mut d = format!("M{},{}", fmt(x0), fmt(y0));
    // Обход по часовой стрелке: дуга со sweep=1 выгибается наружу.
    let sides = [
        ((x0, y0), (x1, y0)),
        ((x1, y0), (x1, y1)),
        ((x1, y1), (x0, y1)),
        ((x0, y1), (x0, y0)),
    ];
    for ((ax, ay), (bx, by)) in sides {
        let len = (bx - ax).hypot(by - ay);
        let count = ((len / 16.0).round() as usize).max(2);
        let step = len / count as f64;
        let (ux, uy) = ((bx - ax) / len, (by - ay) / len);
        for i in 1..=count {
            let t = step * i as f64;
            d.push_str(&format!(
                " A{r},{r} 0 0 1 {x},{y}",
                r = fmt(step * 0.62),
                x = fmt(ax + ux * t),
                y = fmt(ay + uy * t)
            ));
        }
    }
    d
}

/// Рамка контейнера и точка заголовка.
///
/// `tab_w` и `tab_h` — размеры подписи; вкладка рамки считается от них.
/// Возвращает фигуры, точку начала подписи и признак «по центру».
pub fn cluster_frame(
    shape: &str,
    b: &Rect,
    tab_w: f64,
    tab_h: f64,
    stroke: &str,
) -> (Vec<Shape>, f64, f64, bool) {
    let (x, y, w, h) = (b.x, b.y, b.width, b.height);
    let (r, bottom) = (x + w, y + h);
    let thin = Ink {
        fill: "none",
        stroke,
        width: 1.0,
    };
    match shape {
        "package" | "folder" => {
            // Вкладка слева сверху: подпись плюс отступы, скос вправо вниз.
            let th = tab_h + 6.0;
            let d = format!(
                "M{a},{y} L{b},{y} A3.75,3.75 0 0 1 {c},{ya} L{e},{yt} L{rr},{yt} \
                 A{q},{q} 0 0 1 {r},{yq} L{r},{bq} A{q},{q} 0 0 1 {rr},{bottom} \
                 L{a},{bottom} A{q},{q} 0 0 1 {x},{bq} L{x},{ya} A{q},{q} 0 0 1 {a},{y}",
                a = fmt(x + ROUND),
                b = fmt(x + tab_w + 3.5),
                c = fmt(x + tab_w + 6.0),
                e = fmt(x + tab_w + 13.0),
                rr = fmt(r - ROUND),
                r = fmt(r),
                x = fmt(x),
                y = fmt(y),
                ya = fmt(y + ROUND),
                yt = fmt(y + th),
                yq = fmt(y + th + ROUND),
                bq = fmt(bottom - ROUND),
                bottom = fmt(bottom),
                q = ROUND,
            );
            let ink = Ink {
                fill: "none",
                stroke: "#000",
                width: 1.5,
            };
            (
                vec![
                    outline(d, "none", &ink),
                    line(x, y + th, x + tab_w + 13.0, y + th, &ink),
                ],
                x + 4.0,
                y + 15.0,
                false,
            )
        }
        "frame" => {
            let th = tab_h + 3.0;
            let notch = format!(
                "M{},{} L{},{} L{},{} L{},{}",
                fmt(x + tab_w + 10.0),
                fmt(y),
                fmt(x + tab_w + 10.0),
                fmt(y + th - 10.0),
                fmt(x + tab_w),
                fmt(y + th),
                fmt(x),
                fmt(y + th)
            );
            (
                vec![
                    Box::new(
                        Rectangle::new()
                            .set("x", fmt(x))
                            .set("y", fmt(y))
                            .set("width", fmt(w))
                            .set("height", fmt(h))
                            .set("rx", ROUND)
                            .set("ry", ROUND)
                            .set("fill", "none")
                            .set("stroke", stroke)
                            .set("stroke-width", 1),
                    ),
                    outline(notch, "none", &thin),
                ],
                x + 3.0,
                y + 14.0,
                false,
            )
        }
        "component" => (
            vec![
                round_rect(x, y, w, h, "none", &thin),
                rect_plain(r - 20.0, y + 5.0, 15.0, 10.0, &thin),
                rect_plain(r - 22.0, y + 7.0, 4.0, 2.0, &thin),
                rect_plain(r - 22.0, y + 11.0, 4.0, 2.0, &thin),
            ],
            x + w / 2.0,
            y + 18.0,
            true,
        ),
        "cloud" => (
            vec![outline(cloud_path(b), "none", &thin)],
            x + w / 2.0,
            y + 25.0,
            true,
        ),
        "database" => (cylinder(b, CAP, "none", &thin), x + w / 2.0, y + 35.0, true),
        "node" => {
            let (ob, o) = (bottom, NODE_BEVEL);
            let outline_points = polygon(
                &[
                    (x, y + o),
                    (x + o, y),
                    (r, y),
                    (r, ob - o),
                    (r - o, ob),
                    (x, ob),
                    (x, y + o),
                ],
                "none",
                &thin,
            );
            (
                vec![
                    outline_points,
                    line(r - o, y + o, r, y, &thin),
                    line(x, y + o, r - o, y + o, &thin),
                    line(r - o, y + o, r - o, ob, &thin),
                ],
                x + (w - o) / 2.0 + 1.0,
                y + o + 16.0,
                true,
            )
        }
        // Простая рамка: rectangle и всё неизвестное.
        _ => (
            vec![Box::new(
                Rectangle::new()
                    .set("x", fmt(x))
                    .set("y", fmt(y))
                    .set("width", fmt(w))
                    .set("height", fmt(h))
                    .set("rx", ROUND)
                    .set("ry", ROUND)
                    .set("fill", "none")
                    .set("stroke", stroke)
                    .set("stroke-width", 1),
            )],
            x + w / 2.0,
            y + 15.0,
            true,
        ),
    }
}

/// Фигура элемента по виду: контур и центр подписи. `None` — вид не знаком.
/// Сторона квадрата порта и зазор до подписи (эталон `feat_component_porty`).
pub const PORT_SIDE: f64 = 12.0;
pub const PORT_LABEL_GAP: f64 = 4.0;

pub fn symbol(kind: &str, b: &Rect, ink: &Ink) -> Option<(Vec<Shape>, TextSpot)> {
    let (x, y, w, h) = (b.x, b.y, b.width, b.height);
    let (cx, cy) = (x + w / 2.0, y + h / 2.0);
    let (r, bottom) = (x + w, y + h);
    let plain = TextSpot { cx, cy };
    Some(match kind {
        // Порт: квадрат 12 на 12 у левого края, подпись правее в оставшейся ширине.
        "port" => (
            vec![rect_plain(
                x,
                cy - PORT_SIDE / 2.0,
                PORT_SIDE,
                PORT_SIDE,
                ink,
            )],
            TextSpot {
                cx: x + PORT_SIDE + PORT_LABEL_GAP + (w - PORT_SIDE - PORT_LABEL_GAP) / 2.0,
                cy,
            },
        ),
        "component" => {
            // Значок компонента: прямоугольник с двумя «штырьками».
            let shapes: Vec<Shape> = vec![
                round_rect(x, y, w, h, ink.fill, ink),
                rect_plain(r - 20.0, y + 5.0, 15.0, 10.0, ink),
                rect_plain(r - 22.0, y + 7.0, 4.0, 2.0, ink),
                rect_plain(r - 22.0, y + 11.0, 4.0, 2.0, ink),
            ];
            (
                shapes,
                TextSpot {
                    cx: cx - 5.0,
                    cy: cy + 5.0,
                },
            )
        }
        "artifact" => {
            let mut shapes: Vec<Shape> = vec![round_rect(x, y, w, h, ink.fill, ink)];
            let (ix, iy) = (r - 17.0, y + 5.0);
            shapes.push(polygon(
                &[
                    (ix, iy),
                    (ix, iy + 14.0),
                    (ix + 12.0, iy + 14.0),
                    (ix + 12.0, iy + 6.0),
                    (ix + 6.0, iy),
                ],
                ink.fill,
                ink,
            ));
            shapes.push(line(ix + 6.0, iy, ix + 6.0, iy + 6.0, ink));
            shapes.push(line(ix + 12.0, iy + 6.0, ix + 6.0, iy + 6.0, ink));
            (
                shapes,
                TextSpot {
                    cx: cx - 5.0,
                    cy: cy + 1.5,
                },
            )
        }
        "card" | "agent" | "rectangle" => (vec![round_rect(x, y, w, h, ink.fill, ink)], plain),
        "collections" => {
            let (fw, fh) = (w - 4.0, h - 4.0);
            (
                vec![
                    round_rect(x + 4.0, y + 4.0, fw, fh, ink.fill, ink),
                    round_rect(x, y, fw, fh, ink.fill, ink),
                ],
                TextSpot {
                    cx: x + fw / 2.0,
                    cy: y + fh / 2.0,
                },
            )
        }
        "file" => {
            let fold = 10.0;
            let body = format!(
                "M{l},{a} L{l},{b} A2.5,2.5 0 0 0 {la},{bt} L{ra},{bt} A2.5,2.5 0 0 0 {r},{b} \
                 L{r},{f} L{rf},{t} L{la},{t} A2.5,2.5 0 0 0 {l},{a}",
                l = fmt(x),
                r = fmt(r),
                la = fmt(x + ROUND),
                ra = fmt(r - ROUND),
                a = fmt(y + ROUND),
                b = fmt(bottom - ROUND),
                bt = fmt(bottom),
                t = fmt(y),
                f = fmt(y + fold),
                rf = fmt(r - fold),
            );
            let corner = format!(
                "M{},{} L{},{} A2.5,2.5 0 0 0 {},{} L{},{}",
                fmt(r - fold),
                fmt(y),
                fmt(r - fold),
                fmt(y + fold - ROUND),
                fmt(r - fold + ROUND),
                fmt(y + fold),
                fmt(r),
                fmt(y + fold)
            );
            (
                vec![outline(body, ink.fill, ink), outline(corner, ink.fill, ink)],
                plain,
            )
        }
        "hexagon" => {
            let inset = 9.72;
            (
                vec![polygon(
                    &[
                        (r, cy),
                        (r - inset, y),
                        (x + inset, y),
                        (x, cy),
                        (x + inset, bottom),
                        (r - inset, bottom),
                        (r, cy),
                    ],
                    ink.fill,
                    ink,
                )],
                plain,
            )
        }
        "queue" => {
            let cap = 5.0;
            let body = format!(
                "M{a},{t} L{ra},{t} C{r},{t} {r},{cy} {r},{cy} C{r},{cy} {r},{bt} {ra},{bt} \
                 L{a},{bt} C{l},{bt} {l},{cy} {l},{cy} C{l},{cy} {l},{t} {a},{t}",
                a = fmt(x + cap),
                ra = fmt(r - cap),
                l = fmt(x),
                r = fmt(r),
                t = fmt(y),
                bt = fmt(bottom),
                cy = fmt(cy),
            );
            let rim = format!(
                "M{ra},{t} C{ri},{t} {ri},{cy} {ri},{cy} C{ri},{bt} {ra},{bt} {ra},{bt}",
                ra = fmt(r - cap),
                ri = fmt(r - cap * 2.0),
                t = fmt(y),
                bt = fmt(bottom),
                cy = fmt(cy),
            );
            (
                vec![outline(body, ink.fill, ink), outline(rim, "none", ink)],
                TextSpot { cx: cx - 5.0, cy },
            )
        }
        "stack" => {
            let side = 15.0;
            let (il, ir) = (x + side, r - side);
            let inner = Rectangle::new()
                .set("x", fmt(il))
                .set("y", fmt(y))
                .set("width", fmt(ir - il))
                .set("height", fmt(h))
                .set("rx", ROUND)
                .set("ry", ROUND)
                .set("fill", ink.fill)
                .set("stroke", "none");
            let d = format!(
                "M{x},{y} L{a},{y} A2.5,2.5 0 0 1 {il},{ya} L{il},{bq} A2.5,2.5 0 0 0 {ib},{bt} \
                 L{ia},{bt} A2.5,2.5 0 0 0 {ir},{bq} L{ir},{ya} A2.5,2.5 0 0 1 {ic},{y} L{r},{y}",
                x = fmt(x),
                r = fmt(r),
                y = fmt(y),
                a = fmt(il - ROUND),
                il = fmt(il),
                ir = fmt(ir),
                ia = fmt(ir - ROUND),
                ib = fmt(il + ROUND),
                ic = fmt(ir + ROUND),
                ya = fmt(y + ROUND),
                bq = fmt(bottom - ROUND),
                bt = fmt(bottom),
            );
            (vec![Box::new(inner), outline(d, "none", ink)], plain)
        }
        "storage" => (
            vec![Box::new(
                Rectangle::new()
                    .set("x", fmt(x))
                    .set("y", fmt(y))
                    .set("width", fmt(w))
                    .set("height", fmt(h))
                    // Радиус больше полуосей: SVG сводит скругление к эллипсу.
                    .set("rx", 35)
                    .set("ry", 35)
                    .set("fill", ink.fill)
                    .set("stroke", ink.stroke)
                    .set("stroke-width", fmt(ink.width)),
            )],
            plain,
        ),
        "usecase" => (
            vec![Box::new(
                Ellipse::new()
                    .set("cx", fmt(cx))
                    .set("cy", fmt(cy))
                    .set("rx", fmt(w / 2.0))
                    .set("ry", fmt(h / 2.0))
                    .set("fill", ink.fill)
                    .set("stroke", ink.stroke)
                    .set("stroke-width", fmt(ink.width)),
            )],
            plain,
        ),
        // Облако как фигура элемента: подпись по центру.
        "cloud" => (vec![outline(cloud_path(b), ink.fill, ink)], plain),
        "node" => {
            let o = NODE_BEVEL;
            let body = polygon(
                &[
                    (x, y + o),
                    (x + o, y),
                    (r, y),
                    (r, bottom - o),
                    (r - o, bottom),
                    (x, bottom),
                    (x, y + o),
                ],
                ink.fill,
                ink,
            );
            (
                vec![
                    body,
                    line(r - o, y + o, r, y, ink),
                    line(x, y + o, r - o, y + o, ink),
                    line(r - o, y + o, r - o, bottom, ink),
                ],
                TextSpot {
                    cx: x + (w - o) / 2.0,
                    cy: y + o + (h - o) / 2.0,
                },
            )
        }
        "folder" => {
            // Папка: вкладка слева сверху и тело.
            let tab_w = (w * 0.4).clamp(20.0, 50.0);
            let tab_h = 8.0;
            let d = format!(
                "M{x},{ya} L{x},{bq} A2.5,2.5 0 0 0 {xa},{bt} L{ra},{bt} A2.5,2.5 0 0 0 {r},{bq} \
                 L{r},{yt} L{tx},{yt} L{tb},{y} L{xa},{y} A2.5,2.5 0 0 0 {x},{ya}",
                x = fmt(x),
                r = fmt(r),
                xa = fmt(x + ROUND),
                ra = fmt(r - ROUND),
                y = fmt(y),
                ya = fmt(y + ROUND),
                bq = fmt(bottom - ROUND),
                bt = fmt(bottom),
                yt = fmt(y + tab_h),
                tx = fmt(x + tab_w + tab_h),
                tb = fmt(x + tab_w),
            );
            (
                vec![outline(d, ink.fill, ink)],
                TextSpot {
                    cx,
                    cy: y + tab_h + (h - tab_h) / 2.0,
                },
            )
        }
        _ => return None,
    })
}

fn rect_plain(x: f64, y: f64, w: f64, h: f64, ink: &Ink) -> Shape {
    Box::new(
        Rectangle::new()
            .set("x", fmt(x))
            .set("y", fmt(y))
            .set("width", fmt(w))
            .set("height", fmt(h))
            .set("fill", ink.fill)
            .set("stroke", ink.stroke)
            .set("stroke-width", fmt(ink.width)),
    )
}
