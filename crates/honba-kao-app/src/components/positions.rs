//! Positions Table panel modeled after bfx-hf-ui-core PositionsTable.

use crate::ui::{ButtonVariant, HfButton, HfPLNumber, HfPanel, HfPanelHeader, HfRowMarker};
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct PositionItem {
    pub symbol: &'static str,
    pub is_buy: bool,
    pub qty: i64,
    pub avg_price: f64,
    pub mark_price: f64,
    pub unrealized_pnl: f64,
}

#[component]
pub fn PositionsTablePanel() -> impl IntoView {
    let positions = RwSignal::new(vec![
        PositionItem {
            symbol: "RELIANCE.NSE",
            is_buy: true,
            qty: 250,
            avg_price: 2840.50,
            mark_price: 2884.20,
            unrealized_pnl: 10925.00,
        },
        PositionItem {
            symbol: "HDFCBANK.NSE",
            is_buy: true,
            qty: 500,
            avg_price: 1680.00,
            mark_price: 1692.40,
            unrealized_pnl: 6200.00,
        },
        PositionItem {
            symbol: "INFY.NSE",
            is_buy: false,
            qty: 300,
            avg_price: 1845.20,
            mark_price: 1852.90,
            unrealized_pnl: -2310.00,
        },
    ]);

    view! {
        <HfPanel>
            <HfPanelHeader title="Open Positions" counter=3>
                <HfButton variant=ButtonVariant::Red small=true>
                    "Square All"
                </HfButton>
            </HfPanelHeader>

            <div class="hfui-panel__content" style="padding: 0;">
                <table class="hfui-table">
                    <thead>
                        <tr>
                            <th>"Symbol"</th>
                            <th>"Side"</th>
                            <th>"Qty"</th>
                            <th>"Avg Price"</th>
                            <th>"LTP"</th>
                            <th>"Unrealized PnL"</th>
                            <th>"Action"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || positions.get().into_iter().map(|pos| {
                            view! {
                                <tr>
                                    <td>
                                        <strong style="color: var(--shade-9);">{pos.symbol}</strong>
                                    </td>
                                    <td>
                                        <HfRowMarker is_buy=pos.is_buy />
                                        <span style=format!("color: {}; font-weight: 600;", if pos.is_buy { "var(--pnl-gain)" } else { "var(--pnl-loss)" })>
                                            {if pos.is_buy { "LONG" } else { "SHORT" }}
                                        </span>
                                    </td>
                                    <td class="tnum">{pos.qty}</td>
                                    <td class="tnum">{format!("₹{:.2}", pos.avg_price)}</td>
                                    <td class="tnum">{format!("₹{:.2}", pos.mark_price)}</td>
                                    <td>
                                        <HfPLNumber value=pos.unrealized_pnl is_currency=true />
                                    </td>
                                    <td>
                                        <HfButton variant=ButtonVariant::Ghost small=true>
                                            "Close"
                                        </HfButton>
                                    </td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
        </HfPanel>
    }
}
