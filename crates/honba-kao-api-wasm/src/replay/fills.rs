use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FillModel {
    Immediate,
    NextOpen,
    SlippageBps(u32),
}

pub fn simulate_fill_price(
    order_price: Option<f64>,
    market_price: f64,
    is_buy: bool,
    model: FillModel,
) -> f64 {
    match model {
        FillModel::Immediate => order_price.unwrap_or(market_price),
        FillModel::NextOpen => market_price,
        FillModel::SlippageBps(bps) => {
            let slip_rate = bps as f64 / 10000.0;
            let base = order_price.unwrap_or(market_price);
            if is_buy {
                base * (1.0 + slip_rate)
            } else {
                base * (1.0 - slip_rate)
            }
        }
    }
}
