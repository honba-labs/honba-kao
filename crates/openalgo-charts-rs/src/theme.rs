use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChartTheme {
    pub background: String,
    pub text_color: String,
    pub grid_color: String,
    pub up_color: String,
    pub down_color: String,
    pub border_up_color: String,
    pub border_down_color: String,
    pub wick_up_color: String,
    pub wick_down_color: String,
}

impl Default for ChartTheme {
    fn default() -> Self {
        Self {
            background: "#0b0e14".to_string(),
            text_color: "#d1d5db".to_string(),
            grid_color: "#1f2937".to_string(),
            up_color: "#10b981".to_string(),
            down_color: "#ef4444".to_string(),
            border_up_color: "#10b981".to_string(),
            border_down_color: "#ef4444".to_string(),
            wick_up_color: "#10b981".to_string(),
            wick_down_color: "#ef4444".to_string(),
        }
    }
}
