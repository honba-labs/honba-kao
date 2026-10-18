//! Open positions table.

use leptos::prelude::*;

use crate::components::ui::{
    ButtonVariant, HfButton, HfPLNumber, HfPanel, HfPanelHeader, HfRowMarker,
};
use crate::types::Position;
use crate::utils::currency;

#[component]
pub fn PositionsTable(#[prop(optional)] positions: Option<Vec<Position>>) -> impl IntoView {
    let rows = move || {
        positions
            .clone()
            .unwrap_or_else(crate::data::demo_data::demo_positions)
    };

    view! {
        <HfPanel>
            <HfPanelHeader title="Open positions" counter=rows().len()>
                <HfButton variant=ButtonVariant::Red small=true>"Square all"</HfButton>
            </HfPanelHeader>

            <div class="hfui-panel__content flush">
                <table class="hfui-table">
                    <thead>
                        <tr>
                            <th>"Symbol"</th>
                            <th>"Side"</th>
                            <th>"Qty"</th>
                            <th>"Avg"</th>
                            <th>"Mark"</th>
                            <th>"Unrealised"</th>
                            <th>"Action"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || rows()
                            .into_iter()
                            .map(|pos| {
                                view! {
                                    <tr>
                                        <td><strong class="sym">{pos.symbol}</strong></td>
                                        <td>
                                            <HfRowMarker is_buy=pos.is_buy />
                                            <span class=if pos.is_buy { "pnl-up" } else { "pnl-down" }>
                                                {if pos.is_buy { "LONG" } else { "SHORT" }}
                                            </span>
                                        </td>
                                        <td class="tnum">{pos.qty}</td>
                                        <td class="tnum">{currency(pos.avg_price)}</td>
                                        <td class="tnum">{currency(pos.mark_price)}</td>
                                        <td><HfPLNumber value=pos.unrealized_pnl is_currency=true /></td>
                                        <td>
                                            <HfButton variant=ButtonVariant::Ghost small=true>"Close"</HfButton>
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
