use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FillModel {
    Immediate,
    NextOpen,
    SlippageBps(u32),
}

/// Simulate the execution price for an order.
///
/// #61: a limit order can never fill worse than its limit price — a buy is
/// capped at the limit price and a sell is floored at the limit price. Slippage
/// only widens the fill when the limit permits (i.e. for market orders where
/// `order_price` is `None`).
pub fn simulate_fill_price(
    order_price: Option<Decimal>,
    market_price: Decimal,
    is_buy: bool,
    model: FillModel,
) -> Decimal {
    match model {
        FillModel::Immediate => order_price.unwrap_or(market_price),
        FillModel::NextOpen => market_price,
        FillModel::SlippageBps(bps) => {
            let slip_rate = Decimal::from(bps) / dec!(10000);
            let base = order_price.unwrap_or(market_price);
            let slipped = if is_buy {
                base * (dec!(1) + slip_rate)
            } else {
                base * (dec!(1) - slip_rate)
            };
            // #61: cap slippage at the limit price only when a limit exists;
            // market orders (order_price is None) slip freely.
            match order_price {
                Some(limit) if is_buy => slipped.min(limit),
                Some(limit) => slipped.max(limit),
                _ => slipped,
            }
        }
    }
}
