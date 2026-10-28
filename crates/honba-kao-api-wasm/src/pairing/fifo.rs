use super::trade_trip::{Fill, TradeTrip};
use rust_decimal::Decimal;
use std::collections::{HashMap, VecDeque};

pub fn pair_fills_fifo(fills: &[Fill]) -> Vec<TradeTrip> {
    // #57: partition fills by symbol so a RELIANCE buy is never matched
    // against an INFY sell (cross-symbol matching corruption).
    let mut by_symbol: HashMap<String, Vec<Fill>> = HashMap::new();
    for fill in fills {
        by_symbol.entry(fill.symbol.clone()).or_default().push(fill.clone());
    }

    let mut trips = Vec::new();
    for symbol_fills in by_symbol.values() {
        trips.extend(pair_fills_fifo_per_symbol(symbol_fills));
    }
    trips
}

fn pair_fills_fifo_per_symbol(fills: &[Fill]) -> Vec<TradeTrip> {
    let mut trips = Vec::new();
    let mut buy_queue: VecDeque<Fill> = VecDeque::new();
    let mut sell_queue: VecDeque<Fill> = VecDeque::new();
    let zero = Decimal::ZERO;

    for fill in fills {
        if fill.side == "BUY" {
            let mut remaining = fill.quantity;
            while !remaining.is_zero() && !sell_queue.is_empty() {
                let sell = sell_queue.front_mut().unwrap();
                let matched = remaining.min(sell.quantity);

                let pnl = (sell.price - fill.price) * matched;
                let pnl_pct = if sell.price > zero {
                    (sell.price - fill.price) / sell.price
                } else {
                    zero
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
                if sell.quantity.is_zero() {
                    sell_queue.pop_front();
                }
            }

            if !remaining.is_zero() {
                let mut unhedged = fill.clone();
                unhedged.quantity = remaining;
                buy_queue.push_back(unhedged);
            }
        } else {
            let mut remaining = fill.quantity;
            while !remaining.is_zero() && !buy_queue.is_empty() {
                let buy = buy_queue.front_mut().unwrap();
                let matched = remaining.min(buy.quantity);

                let pnl = (fill.price - buy.price) * matched;
                let pnl_pct = if buy.price > zero {
                    (fill.price - buy.price) / buy.price
                } else {
                    zero
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
                if buy.quantity.is_zero() {
                    buy_queue.pop_front();
                }
            }

            if !remaining.is_zero() {
                let mut unhedged = fill.clone();
                unhedged.quantity = remaining;
                sell_queue.push_back(unhedged);
            }
        }
    }

    trips
}
