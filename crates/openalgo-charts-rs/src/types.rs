use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CandleData {
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TimeSeriesPoint {
    pub time: i64,
    pub value: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TimeSeries {
    pub name: String,
    pub points: Vec<TimeSeriesPoint>,
    pub color: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ViewportState {
    pub from_time: i64,
    pub to_time: i64,
    pub min_price: f64,
    pub max_price: f64,
}
