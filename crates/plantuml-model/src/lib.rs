//! # plantuml-model
//!
//! Типизированные модели для layout и рендеринга.
//! Преобразование AST в модели, готовые для визуализации.

pub use plantuml_ast as ast;

/// Точка в 2D пространстве
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Размер в 2D пространстве
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    pub fn zero() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }
}

/// Прямоугольник (bounding box)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_point_size(point: Point, size: Size) -> Self {
        Self {
            x: point.x,
            y: point.y,
            width: size.width,
            height: size.height,
        }
    }

    pub fn center(&self) -> Point {
        Point {
            x: self.x + self.width / 2.0,
            y: self.y + self.height / 2.0,
        }
    }

    pub fn top_left(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }

    pub fn top_right(&self) -> Point {
        Point {
            x: self.x + self.width,
            y: self.y,
        }
    }

    pub fn bottom_left(&self) -> Point {
        Point {
            x: self.x,
            y: self.y + self.height,
        }
    }

    pub fn bottom_right(&self) -> Point {
        Point {
            x: self.x + self.width,
            y: self.y + self.height,
        }
    }

    pub fn left_center(&self) -> Point {
        Point {
            x: self.x,
            y: self.y + self.height / 2.0,
        }
    }

    pub fn right_center(&self) -> Point {
        Point {
            x: self.x + self.width,
            y: self.y + self.height / 2.0,
        }
    }

    pub fn top_center(&self) -> Point {
        Point {
            x: self.x + self.width / 2.0,
            y: self.y,
        }
    }

    pub fn bottom_center(&self) -> Point {
        Point {
            x: self.x + self.width / 2.0,
            y: self.y + self.height,
        }
    }

    /// Создаёт прямоугольник из двух точек
    pub fn from_points(p1: Point, p2: Point) -> Self {
        let min_x = p1.x.min(p2.x);
        let min_y = p1.y.min(p2.y);
        let max_x = p1.x.max(p2.x);
        let max_y = p1.y.max(p2.y);
        Self {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_center() {
        let rect = Rect::new(0.0, 0.0, 100.0, 50.0);
        let center = rect.center();
        assert_eq!(center.x, 50.0);
        assert_eq!(center.y, 25.0);
    }

    #[test]
    fn test_point_new_and_zero() {
        let point = Point::new(3.0, -4.0);
        assert_eq!((point.x, point.y), (3.0, -4.0));
        assert_eq!(Point::zero(), Point::new(0.0, 0.0));
        // Значения по умолчанию совпадают с нулём.
        assert_eq!(Point::default(), Point::zero());
    }

    #[test]
    fn test_size_new_and_zero() {
        let size = Size::new(120.0, 60.0);
        assert_eq!((size.width, size.height), (120.0, 60.0));
        assert_eq!(Size::zero(), Size::new(0.0, 0.0));
        assert_eq!(Size::default(), Size::zero());
    }

    /// Центр прямоугольника — середина его габаритов, а не начала координат.
    #[test]
    fn test_rect_center_accounts_for_offset() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        let center = rect.center();
        assert_eq!((center.x, center.y), (60.0, 45.0));
    }

    /// Все восемь опорных точек считаются от начала рамки.
    #[test]
    fn test_rect_corners_and_edge_midpoints() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);

        assert_eq!((rect.top_left().x, rect.top_left().y), (10.0, 20.0));
        assert_eq!((rect.top_right().x, rect.top_right().y), (110.0, 20.0));
        assert_eq!((rect.bottom_left().x, rect.bottom_left().y), (10.0, 70.0));
        assert_eq!(
            (rect.bottom_right().x, rect.bottom_right().y),
            (110.0, 70.0)
        );

        assert_eq!((rect.left_center().x, rect.left_center().y), (10.0, 45.0));
        assert_eq!(
            (rect.right_center().x, rect.right_center().y),
            (110.0, 45.0)
        );
        assert_eq!((rect.top_center().x, rect.top_center().y), (60.0, 20.0));
        assert_eq!(
            (rect.bottom_center().x, rect.bottom_center().y),
            (60.0, 70.0)
        );
    }

    /// Прямоугольник собирается из точки и размера.
    #[test]
    fn test_rect_from_point_size() {
        let rect = Rect::from_point_size(Point::new(5.0, 6.0), Size::new(10.0, 20.0));
        assert_eq!(
            (rect.x, rect.y, rect.width, rect.height),
            (5.0, 6.0, 10.0, 20.0)
        );
    }

    /// Порядок точек не важен: берётся ограничивающий прямоугольник.
    ///
    /// Этим пользуется раскладка, когда получает две произвольные точки
    /// и должна получить габариты между ними.
    #[test]
    fn test_rect_from_points_handles_any_order() {
        let expected = Rect::new(10.0, 20.0, 30.0, 40.0);
        let first = Point::new(40.0, 60.0);
        let second = Point::new(10.0, 20.0);

        assert_eq!(Rect::from_points(first, second), expected);
        assert_eq!(
            Rect::from_points(second, first),
            expected,
            "порядок точек не должен влиять на результат"
        );
    }

    /// Совпадающие точки дают вырожденный прямоугольник нулевого размера.
    #[test]
    fn test_rect_from_same_points_is_degenerate() {
        let point = Point::new(7.0, 7.0);
        let rect = Rect::from_points(point, point);
        assert_eq!((rect.width, rect.height), (0.0, 0.0));
        assert_eq!(rect.center(), point);
    }
}
