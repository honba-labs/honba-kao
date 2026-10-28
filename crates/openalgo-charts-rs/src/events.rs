use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CrosshairMoveEvent {
    pub time: Option<i64>,
    pub price: Option<f64>,
    pub point_x: f64,
    pub point_y: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VisibleTimeRangeChangedEvent {
    pub from: i64,
    pub to: i64,
}
