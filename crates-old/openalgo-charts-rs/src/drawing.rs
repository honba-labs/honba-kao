//! Drawing tools, persistence, and annotations.

use crate::sys;

/// Standard drawing tools supported by openalgo-charts drawing controller.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum DrawingTool {
    #[default]
    Cursor,
    Trendline,
    Ray,
    HorizontalLine,
    VerticalLine,
    ParallelChannel,
    FibonacciRetracement,
    Rectangle,
    Circle,
    Brush,
    Text,
}

impl DrawingTool {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cursor => "",
            Self::Trendline => "trendline",
            Self::Ray => "ray",
            Self::HorizontalLine => "horizontal-line",
            Self::VerticalLine => "vertical-line",
            Self::ParallelChannel => "parallel-channel",
            Self::FibonacciRetracement => "fib-retracement",
            Self::Rectangle => "rectangle",
            Self::Circle => "circle",
            Self::Brush => "brush",
            Self::Text => "text",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id {
            "trendline" => Self::Trendline,
            "ray" => Self::Ray,
            "horizontal-line" => Self::HorizontalLine,
            "vertical-line" => Self::VerticalLine,
            "parallel-channel" => Self::ParallelChannel,
            "fib-retracement" => Self::FibonacciRetracement,
            "rectangle" => Self::Rectangle,
            "circle" => Self::Circle,
            "brush" => Self::Brush,
            "text" => Self::Text,
            _ => Self::Cursor,
        }
    }
}

/// Arm a drawing tool (`""` or `"cursor"` = cursor mode), returning whether the engine accepted it.
pub fn set_drawing_tool(id: f64, tool: &str) -> bool {
    sys::bridge_set_drawing_tool(id, tool)
        .map(|ok| ok.as_bool().unwrap_or(false))
        .unwrap_or(false)
}

/// The chart's active drawing tool id (`"cursor"` when none is armed).
pub fn drawing_tool(id: f64) -> String {
    sys::bridge_drawing_tool(id)
        .ok()
        .and_then(|val| val.as_string())
        .unwrap_or_else(|| "cursor".to_string())
}

/// Undo the most recent drawing operation.
pub fn undo_drawing(id: f64) {
    let _ = sys::bridge_undo_drawing(id);
}

/// Redo the undone drawing operation.
pub fn redo_drawing(id: f64) {
    let _ = sys::bridge_redo_drawing(id);
}

/// Clear all drawings from the active chart pane.
pub fn clear_drawings(id: f64) {
    let _ = sys::bridge_clear_drawings(id);
}

/// Set instrument symbol context for drawings persistence (saved per symbol in local storage).
pub fn set_symbol_context(id: f64, symbol: &str) {
    let _ = sys::bridge_set_symbol_context(id, symbol);
}
