use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ChartIndicatorOverlay {
    EMA {
        period: usize,
        color: String,
    },
    SMA {
        period: usize,
        color: String,
    },
    BollingerBands {
        period: usize,
        std_dev: f64,
        color: String,
    },
    VWAP {
        color: String,
    },
}
