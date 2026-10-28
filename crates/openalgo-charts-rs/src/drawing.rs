use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DrawingTool {
    TrendLine,
    HorizontalLine,
    Ray,
    FibonacciRetracement,
    Rectangle,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DrawingItem {
    pub id: String,
    pub tool: DrawingTool,
    pub color: String,
    pub points: Vec<(i64, f64)>, // (time, price)
}
