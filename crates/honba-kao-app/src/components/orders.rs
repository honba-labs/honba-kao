//! Orders Table panel modeled after bfx-hf-ui-core AtomicOrdersTable.

use crate::ui::{ButtonVariant, HfButton, HfPanel, HfPanelHeader, HfRowMarker};
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct OrderItem {
    pub id: &'static str,
    pub symbol: &'static str,
    pub order_type: &'static str,
    pub is_buy: bool,
    pub qty: i64,
    pub filled_qty: i64,
    pub price: f64,
    pub status: &'static str,
}

#[component]
pub fn OrdersTablePanel() -> impl IntoView {
    let orders = RwSignal::new(vec![
        OrderItem {
            id: "ord-101",
            symbol: "TCS.NSE",
            order_type: "LIMIT",
            is_buy: true,
            qty: 100,
            filled_qty: 40,
            price: 4120.00,
            status: "PARTIAL",
        },
        OrderItem {
            id: "ord-102",
            symbol: "ICICIBANK.NSE",
            order_type: "TWAP (ALGO)",
            is_buy: true,
            qty: 400,
            filled_qty: 150,
            price: 1255.50,
            status: "RUNNING",
        },
        OrderItem {
            id: "ord-103",
            symbol: "TATAMOTORS.NSE",
            order_type: "STOP LOSS",
            is_buy: false,
            qty: 200,
            filled_qty: 0,
            price: 980.00,
            status: "PENDING",
        },
    ]);

    view! {
        <HfPanel>
            <HfPanelHeader title="Active & Algo Orders" counter=3>
                <HfButton variant=ButtonVariant::Ghost small=true>
                    "Cancel All"
                </HfButton>
            </HfPanelHeader>

            <div class="hfui-panel__content" style="padding: 0;">
                <table class="hfui-table">
                    <thead>
                        <tr>
                            <th>"Order ID"</th>
                            <th>"Symbol"</th>
                            <th>"Type"</th>
                            <th>"Side"</th>
                            <th>"Filled / Total"</th>
                            <th>"Limit Price"</th>
                            <th>"Status"</th>
                            <th>"Action"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || orders.get().into_iter().map(|ord| {
                            view! {
                                <tr>
                                    <td class="tnum" style="color: var(--shade-5);">{ord.id}</td>
                                    <td><strong style="color: var(--shade-9);">{ord.symbol}</strong></td>
                                    <td><span style="font-size: 11px; color: var(--shade-5);">{ord.order_type}</span></td>
                                    <td>
                                        <HfRowMarker is_buy=ord.is_buy />
                                        <span style=format!("color: {}; font-weight: 600;", if ord.is_buy { "var(--pnl-gain)" } else { "var(--pnl-loss)" })>
                                            {if ord.is_buy { "BUY" } else { "SELL" }}
                                        </span>
                                    </td>
                                    <td class="tnum">{format!("{} / {}", ord.filled_qty, ord.qty)}</td>
                                    <td class="tnum">{format!("₹{:.2}", ord.price)}</td>
                                    <td>
                                        <span style="font-weight: 600; font-size: 11px; color: var(--primary-8);">
                                            {ord.status}
                                        </span>
                                    </td>
                                    <td>
                                        <HfButton variant=ButtonVariant::Ghost small=true>
                                            "Cancel"
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
