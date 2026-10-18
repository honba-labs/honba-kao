//! Closed-trade history. Mirrors `web/src/components/trader/PositionHistory.tsx`.

use leptos::prelude::*;

use crate::components::ui::{HfPLNumber, HfPanel, HfPanelHeader};
use crate::utils::currency;

struct ClosedTrade {
    closed_at: &'static str,
    symbol: &'static str,
    side: &'static str,
    qty: i64,
    entry: f64,
    exit: f64,
    realized: f64,
}

const TRADES: &[ClosedTrade] = &[
    ClosedTrade {
        closed_at: "2026-10-17",
        symbol: "RELIANCE",
        side: "LONG",
        qty: 150,
        entry: 2812.00,
        exit: 2884.20,
        realized: 10_830.0,
    },
    ClosedTrade {
        closed_at: "2026-10-16",
        symbol: "INFY",
        side: "SHORT",
        qty: 200,
        entry: 1860.50,
        exit: 1852.90,
        realized: 1_520.0,
    },
    ClosedTrade {
        closed_at: "2026-10-16",
        symbol: "SBIN",
        side: "LONG",
        qty: 300,
        entry: 820.00,
        exit: 812.50,
        realized: -2_250.0,
    },
    ClosedTrade {
        closed_at: "2026-10-15",
        symbol: "TCS",
        side: "LONG",
        qty: 60,
        entry: 4088.00,
        exit: 4120.00,
        realized: 1_920.0,
    },
];

#[component]
pub fn PositionHistory() -> impl IntoView {
    view! {
        <HfPanel>
            <HfPanelHeader title="Closed positions" counter=TRADES.len() />

            <div class="hfui-panel__content flush">
                <table class="hfui-table">
                    <thead>
                        <tr>
                            <th>"Closed"</th>
                            <th>"Symbol"</th>
                            <th>"Side"</th>
                            <th>"Qty"</th>
                            <th>"Entry"</th>
                            <th>"Exit"</th>
                            <th>"Realised"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {TRADES
                            .iter()
                            .map(|t| {
                                let long = t.side == "LONG";
                                view! {
                                    <tr>
                                        <td class="tnum muted">{t.closed_at}</td>
                                        <td><strong class="sym">{t.symbol}</strong></td>
                                        <td>
                                            <span class=if long { "pnl-up" } else { "pnl-down" }>{t.side}</span>
                                        </td>
                                        <td class="tnum">{t.qty}</td>
                                        <td class="tnum">{currency(t.entry)}</td>
                                        <td class="tnum">{currency(t.exit)}</td>
                                        <td><HfPLNumber value=t.realized is_currency=true /></td>
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
