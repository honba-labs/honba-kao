//! Execution log: fill-by-fill record of what the engine did.

use leptos::prelude::*;

use crate::components::ui::{HfPanel, HfPanelHeader};
use crate::utils::{currency, quantity};

struct Fill {
    time: &'static str,
    symbol: &'static str,
    side: &'static str,
    qty: i64,
    price: f64,
    venue: &'static str,
}

const FILLS: &[Fill] = &[
    Fill {
        time: "15:24:02",
        symbol: "RELIANCE",
        side: "BUY",
        qty: 50,
        price: 2884.20,
        venue: "NSE",
    },
    Fill {
        time: "15:22:41",
        symbol: "HDFCBANK",
        side: "BUY",
        qty: 100,
        price: 1690.00,
        venue: "NSE",
    },
    Fill {
        time: "15:19:58",
        symbol: "INFY",
        side: "SELL",
        qty: 75,
        price: 1852.90,
        venue: "NSE",
    },
    Fill {
        time: "15:14:12",
        symbol: "TCS",
        side: "BUY",
        qty: 40,
        price: 4120.00,
        venue: "BSE",
    },
    Fill {
        time: "15:09:33",
        symbol: "SBIN",
        side: "SELL",
        qty: 200,
        price: 812.50,
        venue: "NSE",
    },
];

#[component]
pub fn ExecutionLog() -> impl IntoView {
    view! {
        <HfPanel>
            <HfPanelHeader title="Execution log" counter=FILLS.len() />

            <div class="hfui-panel__content flush">
                <table class="hfui-table">
                    <thead>
                        <tr>
                            <th>"Time"</th>
                            <th>"Symbol"</th>
                            <th>"Side"</th>
                            <th>"Qty"</th>
                            <th>"Price"</th>
                            <th>"Venue"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {FILLS
                            .iter()
                            .map(|f| {
                                let up = f.side == "BUY";
                                view! {
                                    <tr>
                                        <td class="tnum muted">{f.time}</td>
                                        <td><strong class="sym">{f.symbol}</strong></td>
                                        <td>
                                            <span class=if up { "pnl-up" } else { "pnl-down" }>{f.side}</span>
                                        </td>
                                        <td class="tnum">{quantity(f.qty)}</td>
                                        <td class="tnum">{currency(f.price)}</td>
                                        <td class="muted small">{f.venue}</td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>
        </HfPanel>
    }
}
