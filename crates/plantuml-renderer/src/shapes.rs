//! Примитивы для рисования фигур

use crate::Point;

/// Строит путь для прямоугольника со скруглёнными углами
pub fn rounded_rect_path(x: f64, y: f64, width: f64, height: f64, radius: f64) -> String {
    if radius <= 0.0 {
        return format!("M{},{} h{} v{} h{} Z", x, y, width, height, -width);
    }

    let r = radius.min(width / 2.0).min(height / 2.0);

    format!(
        "M{},{} h{} a{},{} 0 0 1 {},{} v{} a{},{} 0 0 1 {},{} h{} a{},{} 0 0 1 {},{} v{} a{},{} 0 0 1 {},{} Z",
        x + r, y,
        width - 2.0 * r,
        r, r, r, r,
        height - 2.0 * r,
        r, r, -r, r,
        -(width - 2.0 * r),
        r, r, -r, -r,
        -(height - 2.0 * r),
        r, r, r, -r
    )
}

/// Строит путь для стрелки
pub fn arrow_path(from: Point, to: Point, head_size: f64) -> String {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let angle = dy.atan2(dx);

    let head_angle = std::f64::consts::PI / 6.0; // 30 градусов

    let x1 = to.x - head_size * (angle - head_angle).cos();
    let y1 = to.y - head_size * (angle - head_angle).sin();
    let x2 = to.x - head_size * (angle + head_angle).cos();
    let y2 = to.y - head_size * (angle + head_angle).sin();

    format!(
        "M{},{} L{},{} M{},{} L{},{} L{},{}",
        from.x, from.y, to.x, to.y, to.x, to.y, x1, y1, x2, y2
    )
}

/// Строит путь для ромба (diamond)
pub fn diamond_path(cx: f64, cy: f64, size: f64) -> String {
    let half = size / 2.0;
    format!(
        "M{},{} L{},{} L{},{} L{},{} Z",
        cx,
        cy - half, // Верх
        cx + half,
        cy, // Право
        cx,
        cy + half, // Низ
        cx - half,
        cy // Лево
    )
}

/// Строит путь для актёра (человечек)
pub fn actor_path(cx: f64, cy: f64, scale: f64) -> String {
    let head_r = 8.0 * scale;
    let body_h = 20.0 * scale;
    let arms_w = 20.0 * scale;
    let legs_h = 15.0 * scale;

    // Голова (круг)
    let head = format!(
        "M{},{} a{},{} 0 1 0 {},0 a{},{} 0 1 0 {},0",
        cx - head_r,
        cy - body_h - head_r,
        head_r,
        head_r,
        head_r * 2.0,
        head_r,
        head_r,
        -head_r * 2.0
    );

    // Тело
    let body = format!("M{},{} L{},{}", cx, cy - body_h, cx, cy);

    // Руки
    let arms = format!(
        "M{},{} L{},{}",
        cx - arms_w / 2.0,
        cy - body_h / 2.0,
        cx + arms_w / 2.0,
        cy - body_h / 2.0
    );

    // Ноги
    let legs = format!(
        "M{},{} L{},{} M{},{} L{},{}",
        cx,
        cy,
        cx - arms_w / 3.0,
        cy + legs_h,
        cx,
        cy,
        cx + arms_w / 3.0,
        cy + legs_h
    );

    format!("{} {} {} {}", head, body, arms, legs)
}

/// Строит путь для базы данных (цилиндр)
pub fn database_path(x: f64, y: f64, width: f64, height: f64) -> String {
    let ellipse_h = height * 0.15;

    // Верхний эллипс
    let top = format!(
        "M{},{} a{},{} 0 1 0 {},0 a{},{} 0 1 0 {},0",
        x,
        y + ellipse_h,
        width / 2.0,
        ellipse_h,
        width,
        width / 2.0,
        ellipse_h,
        -width
    );

    // Боковые линии
    let sides = format!(
        "M{},{} L{},{} M{},{} L{},{}",
        x,
        y + ellipse_h,
        x,
        y + height - ellipse_h,
        x + width,
        y + ellipse_h,
        x + width,
        y + height - ellipse_h
    );

    // Нижний эллипс (половина)
    let bottom = format!(
        "M{},{} a{},{} 0 0 0 {},0",
        x,
        y + height - ellipse_h,
        width / 2.0,
        ellipse_h,
        width
    );

    format!("{} {} {}", top, sides, bottom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rounded_rect() {
        let path = rounded_rect_path(0.0, 0.0, 100.0, 50.0, 5.0);
        assert!(path.contains("M5,0"));
        assert!(path.contains("a5,5"));
    }

    #[test]
    fn test_diamond() {
        let path = diamond_path(50.0, 50.0, 20.0);
        assert!(path.contains("M50,40")); // Верх
        assert!(path.contains("L60,50")); // Право
    }

    /// Прямоугольник без скругления рисуется упрощённым путём.
    ///
    /// Ветка нужна отдельно: у PlantUML состояния без скруглённых углов
    /// (база данных, разделители классов), и при нулевом радиусе путь
    /// обязан оставаться замкнутым, без дуг.
    #[test]
    fn test_rounded_rect_without_radius_uses_simple_path() {
        let path = rounded_rect_path(10.0, 20.0, 100.0, 50.0, 0.0);
        assert_eq!(path, "M10,20 h100 v50 h-100 Z");
        assert!(
            !path.contains('a'),
            "при нулевом радиусе дуг быть не должно"
        );
    }

    /// Отрицательный радиус тоже считается отсутствием скругления.
    #[test]
    fn test_rounded_rect_negative_radius_uses_simple_path() {
        let path = rounded_rect_path(0.0, 0.0, 40.0, 20.0, -3.0);
        assert_eq!(path, "M0,0 h40 v20 h-40 Z");
    }

    /// Радиус ограничивается половиной меньшей стороны.
    ///
    /// Без этого ограничения на вытянутой фигуре дуги вылезали бы за края
    /// и рамка выглядела бы сломанной.
    #[test]
    fn test_rounded_rect_radius_clamped_to_half_height() {
        // Высота 20 при радиусе 50: дуги не должны превышать 10.
        let path = rounded_rect_path(0.0, 0.0, 200.0, 20.0, 50.0);
        assert!(
            path.contains("a10,10"),
            "радиус должен быть ограничен половиной высоты: {path}"
        );
        assert!(
            path.contains("h180"),
            "длина прямой должна считаться от радиуса"
        );
    }

    /// Стрелка состоит из линии к цели и двух лучей наконечника.
    #[test]
    fn test_arrow_path_has_line_and_two_heads() {
        let from = Point::new(0.0, 0.0);
        let to = Point::new(100.0, 0.0);
        let path = arrow_path(from, to, 10.0);

        // Прямая линия от начала к концу.
        assert!(path.starts_with("M0,0 L100,0"), "нет линии: {path}");
        // Наконечник рисуется отдельным контуром, оба луча выходят из
        // точки конца: `M100,0 L<левый> L<правый>`.
        assert!(
            path.contains("M100,0 L"),
            "наконечник должен начинаться в точке конца: {path}"
        );
        let head = path
            .rsplit("M100,0 ")
            .next()
            .expect("разделитель не найден");
        assert_eq!(
            head.matches('L').count(),
            2,
            "в наконечнике должно быть два луча: {path}"
        );
    }

    /// Стрелка под углом: наконечник обязан разойтись в стороны.
    #[test]
    fn test_arrow_path_angled() {
        let from = Point::new(0.0, 0.0);
        let to = Point::new(100.0, 100.0);
        let path = arrow_path(from, to, 10.0);
        assert!(path.starts_with("M0,0 L100,100"));
        assert!(path.contains("L100,100 "), "оба луча из точки конца");
    }

    /// Ромб состоит из четырёх вершин, последняя замыкается.
    #[test]
    fn test_diamond_path_has_four_vertices() {
        let path = diamond_path(50.0, 50.0, 20.0);
        assert_eq!(path, "M50,40 L60,50 L50,60 L40,50 Z");
    }

    /// Ромб нулевого размера не должен давать NaN.
    #[test]
    fn test_diamond_path_zero_size_has_no_nan() {
        let path = diamond_path(10.0, 10.0, 0.0);
        assert!(!path.contains("NaN"), "NaN в пути ромба: {path}");
        assert!(!path.contains("inf"), "бесконечность в пути ромба: {path}");
    }

    /// Фигура актёра: голова, тело, руки и ноги.
    #[test]
    fn test_actor_path_has_four_parts() {
        let path = actor_path(50.0, 100.0, 1.0);
        // Голова рисуется дугами `a8,8`, тело и конечности — отрезками `L`.
        assert!(path.contains("a8,8"), "нет головы: {path}");
        assert!(path.contains("M50,80 L50,100"), "нет тела: {path}");
        assert!(path.contains("M40,90 L60,90"), "нет рук: {path}");
        assert!(path.contains("L43.333333333333336,115"), "нет ног: {path}");
        // Пять отдельных контуров: голова, тело, руки и две ноги —
        // каждая нога начинается своим `M`.
        assert_eq!(
            path.matches('M').count(),
            5,
            "ожидались голова, тело, руки и две ноги: {path}"
        );
    }

    /// Масштаб актёра умножает все размеры.
    #[test]
    fn test_actor_path_scales() {
        let small = actor_path(50.0, 100.0, 1.0);
        let big = actor_path(50.0, 100.0, 2.0);
        assert!(small.contains("a8,8"), "радиус головы при масштабе 1");
        assert!(big.contains("a16,16"), "радиус головы при масштабе 2");
        assert_ne!(small, big, "масштаб не подействовал");
    }

    /// Цилиндр базы данных: верхний эллипс, бока и нижняя дуга.
    #[test]
    fn test_database_path_structure() {
        let path = database_path(10.0, 20.0, 60.0, 40.0);
        // Верхний эллипс рисуется дугами, бока — отрезками.
        assert!(path.contains("a30,6"), "нет верхнего эллипса: {path}");
        assert!(path.contains("M10,26 L10,54"), "нет левой боковой: {path}");
        assert!(path.contains("M70,26 L70,54"), "нет правой боковой: {path}");
        // Нижний эллипс — половина дуги без флага крупной дуги.
        assert!(path.contains("a30,6 0 0 0 60,0"), "нет нижней дуги: {path}");
    }

    /// Высота эллипса — 15% от высоты цилиндра.
    #[test]
    fn test_database_path_ellipse_is_fifteen_percent() {
        let path = database_path(0.0, 0.0, 100.0, 200.0);
        assert!(
            path.contains("a50,30"),
            "радиус дуги должен быть 30: {path}"
        );
    }
}
