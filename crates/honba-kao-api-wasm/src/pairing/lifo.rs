use super::trade_trip::{Fill, TradeTrip};

pub fn pair_fills_lifo(fills: &[Fill]) -> Vec<TradeTrip> {
    let mut trips = Vec::new();
    let mut buy_stack: Vec<Fill> = Vec::new();
    let mut sell_stack: Vec<Fill> = Vec::new();

    for fill in fills {
        if fill.side == "BUY" {
            let mut remaining = fill.quantity;
            while remaining > 1e-6 && !sell_stack.is_empty() {
                let sell = sell_stack.last_mut().unwrap();
                let matched = remaining.min(sell.quantity);

                let pnl = (sell.price - fill.price) * matched;
                let pnl_pct = if sell.price > 0.0 {
                    (sell.price - fill.price) / sell.price
                } else {
                    0.0
                };

                trips.push(TradeTrip {
                    symbol: fill.symbol.clone(),
                    is_long: false,
                    entry_price: sell.price,
                    exit_price: fill.price,
                    quantity: matched,
                    pnl,
                    pnl_pct,
                    entry_time: sell.timestamp,
                    exit_time: fill.timestamp,
                    duration_seconds: (fill.timestamp - sell.timestamp).max(0),
                });

                remaining -= matched;
                sell.quantity -= matched;
                if sell.quantity <= 1e-6 {
                    sell_stack.pop();
                }
            }

            if remaining > 1e-6 {
                let mut unhedged = fill.clone();
                unhedged.quantity = remaining;
                buy_stack.push(unhedged);
            }
        } else {
            let mut remaining = fill.quantity;
            while remaining > 1e-6 && !buy_stack.is_empty() {
                let buy = buy_stack.last_mut().unwrap();
                let matched = remaining.min(buy.quantity);

                let pnl = (fill.price - buy.price) * matched;
                let pnl_pct = if buy.price > 0.0 {
                    (fill.price - buy.price) / buy.price
                } else {
                    0.0
                };

                trips.push(TradeTrip {
                    symbol: fill.symbol.clone(),
                    is_long: true,
                    entry_price: buy.price,
                    exit_price: fill.price,
                    quantity: matched,
                    pnl,
                    pnl_pct,
                    entry_time: buy.timestamp,
                    exit_time: fill.timestamp,
                    duration_seconds: (fill.timestamp - buy.timestamp).max(0),
                });

                remaining -= matched;
                buy.quantity -= matched;
                if buy.quantity <= 1e-6 {
                    buy_stack.pop();
                }
            }

            if remaining > 1e-6 {
                let mut unhedged = fill.clone();
                unhedged.quantity = remaining;
                sell_stack.push(unhedged);
            }
        }
    }

    trips
}
