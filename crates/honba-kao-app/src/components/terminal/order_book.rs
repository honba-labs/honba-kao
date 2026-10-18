//! Depth ladder / order book.

use leptos::prelude::*;

use crate::components::ui::{HfPanel, HfPanelHeader};
use crate::utils::currency;

struct Level {
    price: f64,
    qty: i64,
    depth_pct: f64,
}

fn bids() -> Vec<Level> {
    vec![
        Level {
            price: 2884.20,
            qty: 180,
            depth_pct: 62.0,
        },
        Level {
            price: 2883.85,
            qty: 240,
            depth_pct: 82.0,
        },
        Level {
            price: 2883.40,
            qty: 120,
            depth_pct: 44.0,
        },
        Level {
            price: 2882.95,
            qty: 300,
            depth_pct: 100.0,
        },
    ]
}

fn asks() -> Vec<Level> {
    vec![
        Level {
            price: 2884.55,
            qty: 140,
            depth_pct: 48.0,
        },
        Level {
            price: 2884.90,
            qty: 220,
            depth_pct: 74.0,
        },
        Level {
            price: 2885.35,
            qty: 160,
            depth_pct: 55.0,
        },
        Level {
            price: 2885.80,
            qty: 260,
            depth_pct: 88.0,
        },
    ]
}

#[component]
pub fn OrderBook() -> impl IntoView {
    let spread = 2884.55 - 2884.20;

    view! {
        <HfPanel>
            <HfPanelHeader title="Order book">
                <span class="muted small">"spread "{currency(spread)}</span>
            </HfPanelHeader>

            <div class="hfui-panel__content depth">
                <div class="depth__side">
                    {asks()
                        .into_iter()
                        .rev()
                        .map(|level| view! {
                            <div class="depth__row depth__row--ask">
                                <span class="depth__bar" style=format!("width: {:.0}%;", level.depth_pct) />
                                <span class="tnum depth__price">{currency(level.price)}</span>
                                <span class="tnum depth__qty">{level.qty}</span>
                            </div>
                        })
                        .collect_view()}
                </div>

                <div class="depth__mid">{currency(2884.20)}</div>

                <div class="depth__side">
                    {bids()
                        .into_iter()
                        .map(|level| view! {
                            <div class="depth__row depth__row--bid">
                                <span class="depth__bar" style=format!("width: {:.0}%;", level.depth_pct) />
                                <span class="tnum depth__price">{currency(level.price)}</span>
                                <span class="tnum depth__qty">{level.qty}</span>
                            </div>
                        })
                        .collect_view()}
                </div>
            </div>
        </HfPanel>
    }
}
