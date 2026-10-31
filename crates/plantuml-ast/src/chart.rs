//! AST диаграммы `@startchart`: оси и ряды данных.

use serde::{Deserialize, Serialize};

use crate::common::DiagramMetadata;

/// Вид ряда
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeriesKind {
    Bar,
    Line,
    Area,
}

/// Ряд данных
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Series {
    pub kind: SeriesKind,
    pub name: String,
    pub values: Vec<f64>,
    /// Цвет `#rrggbb` или имя; без него берётся цвет по умолчанию
    pub color: Option<String>,
    /// Подписывать значения
    pub labels: bool,
}

/// Горизонтальная ось
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HAxis {
    pub title: Option<String>,
    /// Подписи категорий `[a, b, c]`
    pub categories: Vec<String>,
}

/// Вертикальная ось
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VAxis {
    pub title: Option<String>,
    /// Границы `min --> max`; без них вычисляются по данным
    pub range: Option<(f64, f64)>,
    pub spacing: Option<f64>,
}

/// Положение легенды
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegendPosition {
    Left,
    Right,
    Top,
    Bottom,
}

/// Диаграмма `@startchart`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ChartDiagram {
    pub h_axis: HAxis,
    pub v_axis: VAxis,
    pub series: Vec<Series>,
    pub legend: Option<LegendPosition>,
    /// `stackMode stacked`
    pub stacked: bool,
    pub metadata: DiagramMetadata,
}
