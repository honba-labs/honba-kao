//! Synthetic 5-level market depth, ported from the labs `market-depth.tsx`.
//!
//! No backend call: the book is derived from the instrument's demo LTP —
//! levels sit one "tick" (0.05% of LTP) apart with a fixed order/qty mix.

use leptos::prelude::*;

use crate::features::workbench::stores::order_draft_store::instrument_price;
use crate::shared::utils::quantity;

#[derive(Clone, Debug, PartialEq)]
struct DepthLevel {
    orders: i64,
    qty: i64,
    price: f64,
}

#[component]
pub fn MarketDepth(symbol: String) -> impl IntoView {
    let title = format!("Market Depth ({symbol})");

    let book = Memo::new(move |_| {
        let ltp = instrument_price(&symbol);
        let tick = round2(ltp * 0.0005).max(0.05);

        // (steps, orders, qty) — the labs book's fixed composition.
        let bids: Vec<DepthLevel> = [
            (-1, 14, 1250),
            (-2, 28, 3400),
            (-3, 42, 5800),
            (-4, 19, 2100),
            (-5, 35, 4500),
        ]
        .into_iter()
        .map(|(steps, orders, qty)| DepthLevel {
            orders,
            qty,
            price: round2(ltp + tick * steps as f64),
        })
        .collect();

        let asks: Vec<DepthLevel> = [
            (1, 18, 1600),
            (2, 31, 2900),
            (3, 50, 6200),
            (4, 22, 2400),
            (5, 29, 3800),
        ]
        .into_iter()
        .map(|(steps, orders, qty)| DepthLevel {
            orders,
            qty,
            price: round2(ltp + tick * steps as f64),
        })
        .collect();

        let total_bid: i64 = bids.iter().map(|level| level.qty).sum();
        let total_ask: i64 = asks.iter().map(|level| level.qty).sum();
        let spread = round2(asks[0].price - bids[0].price);
        let spread_pct = ((spread / ltp) * 100.0 * 1000.0).round() / 1000.0;
        let max_qty = bids
            .iter()
            .chain(asks.iter())
            .map(|level| level.qty)
            .max()
            .unwrap_or(1);

        (
            bids, asks, total_bid, total_ask, spread, spread_pct, max_qty,
        )
    });

    view! {
        <div class="wb-depth">
            <div class="wb-depth__head">
                <span>{title}</span>
                {move || {
                    let (_, _, _, _, spread, spread_pct, _) = book.get();
                    format!("Spread: ₹{spread:.2} ({spread_pct:.3}%)")
                }}
            </div>

            <div class="wb-depth__grid">
                <div class="wb-depth__side">
                    <div class="wb-depth__cols">
                        <span>"Orders"</span>
                        <span>"Qty"</span>
                        <span class="wb-depth__bid">"Bid"</span>
                    </div>
                    {move || {
                        let (bids, _, _, _, _, _, max_qty) = book.get();
                        bids.into_iter()
                            .map(|level| view! { <LevelRow level=level max_qty=max_qty is_bid=true /> })
                            .collect_view()
                    }}
                    {move || {
                        let (_, _, total_bid, _, _, _, _) = book.get();
                        view! {
                            <div class="wb-depth__total wb-depth__total--bid">
                                <span>"Total"</span>
                                <span>{quantity(total_bid)}</span>
                            </div>
                        }
                    }}
                </div>

                <div class="wb-depth__side">
                    <div class="wb-depth__cols">
                        <span class="wb-depth__ask">"Ask"</span>
                        <span>"Qty"</span>
                        <span>"Orders"</span>
                    </div>
                    {move || {
                        let (_, asks, _, _, _, _, max_qty) = book.get();
                        asks.into_iter()
                            .map(|level| view! { <LevelRow level=level max_qty=max_qty is_bid=false /> })
                            .collect_view()
                    }}
                    {move || {
                        let (_, _, _, total_ask, _, _, _) = book.get();
                        view! {
                            <div class="wb-depth__total wb-depth__total--ask">
                                <span>"Total"</span>
                                <span>{quantity(total_ask)}</span>
                            </div>
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

/// One depth row with a qty bar behind the figures.
///
/// Bids read orders → qty → price; asks mirror them, like the labs grid.
#[component]
fn LevelRow(level: DepthLevel, max_qty: i64, is_bid: bool) -> impl IntoView {
    let width = format!("width: {:.1}%;", level.qty as f64 / max_qty as f64 * 100.0);
    let bar_class = if is_bid {
        "wb-depth__bar--bid"
    } else {
        "wb-depth__bar--ask"
    };

    let qty = quantity(level.qty);
    let price = format!("{:.2}", level.price);
    let orders = level.orders.to_string();

    // Cell order mirrors the labs grid: bids (orders, qty, price), asks
    // (price, qty, orders).
    let (left, middle, right) = if is_bid {
        (
            ("wb-depth__orders".to_string(), orders.clone()),
            qty.clone(),
            ("wb-depth__price wb-depth__bid".to_string(), price.clone()),
        )
    } else {
        (
            ("wb-depth__price wb-depth__ask".to_string(), price.clone()),
            qty.clone(),
            ("wb-depth__orders".to_string(), orders.clone()),
        )
    };
    let (left_class, left_value) = left;
    let (right_class, right_value) = right;

    view! {
        <div class="wb-depth__row">
            <span class=format!("wb-depth__bar {}", bar_class) style=width />
            <span class=left_class>{left_value}</span>
            <span>{middle}</span>
            <span class=right_class>{right_value}</span>
        </div>
    }
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
