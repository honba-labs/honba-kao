//! Working orders table.

use leptos::prelude::*;

use crate::components::ui::{ButtonVariant, HfButton, HfPanel, HfPanelHeader, HfRowMarker};
use crate::types::Order;
use crate::utils::currency;

#[component]
pub fn OrdersTable(#[prop(optional)] orders: Option<Vec<Order>>) -> impl IntoView {
    let rows = move || {
        orders
            .clone()
            .unwrap_or_else(crate::data::demo_data::demo_orders)
    };

    view! {
        <HfPanel>
            <HfPanelHeader title="Active & algo orders" counter=rows().len()>
                <HfButton variant=ButtonVariant::Ghost small=true>"Cancel all"</HfButton>
            </HfPanelHeader>

            <div class="hfui-panel__content flush">
                <table class="hfui-table">
                    <thead>
                        <tr>
                            <th>"Order"</th>
                            <th>"Symbol"</th>
                            <th>"Type"</th>
                            <th>"Side"</th>
                            <th>"Filled / total"</th>
                            <th>"Limit"</th>
                            <th>"Status"</th>
                            <th>"Action"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || rows()
                            .into_iter()
                            .map(|ord| {
                                view! {
                                    <tr>
                                        <td class="tnum muted">{ord.order_id}</td>
                                        <td><strong class="sym">{ord.symbol}</strong></td>
                                        <td><span class="muted small">{ord.order_type}</span></td>
                                        <td>
                                            <HfRowMarker is_buy=ord.is_buy />
                                            <span class=if ord.is_buy { "pnl-up" } else { "pnl-down" }>
                                                {if ord.is_buy { "BUY" } else { "SELL" }}
                                            </span>
                                        </td>
                                        <td class="tnum">
                                            {format!("{} / {}", ord.filled_qty, ord.qty)}
                                        </td>
                                        <td class="tnum">{currency(ord.price)}</td>
                                        <td><span class="status-chip">{ord.status}</span></td>
                                        <td>
                                            <HfButton variant=ButtonVariant::Ghost small=true>"Cancel"</HfButton>
                                        </td>
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
