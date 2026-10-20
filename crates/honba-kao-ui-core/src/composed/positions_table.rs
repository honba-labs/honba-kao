//! HfPositionsTable — Symbol, Side, Qty, Avg, LTP, UPL, RPL, actions.
//! Mirrors bfx-hf-ui-core PositionsTablePanel.

use leptos::prelude::*;
use crate::trading::HfPLNumber;

/// Position data for the table.
#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    pub symbol: String,
    pub side: crate::trading::HfSide,
    pub quantity: i64,
    pub avg_price: f64,
    pub ltp: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
}

#[component]
pub fn HfPositionsTable(
    positions: Vec<Position>,
    #[prop(optional, into)]
    class: String,
) -> impl IntoView {
    let full_class = format!("hf-positions-table {class}").trim().to_string();

    view! {
        <div class=full_class>
            <table class="hf-table">
                <thead>
                    <tr>
                        <th>"Symbol"</th>
                        <th>"Side"</th>
                        <th class="hf-table__cell--numeric">"Qty"</th>
                        <th class="hf-table__cell--numeric">"Avg"</th>
                        <th class="hf-table__cell--numeric">"LTP"</th>
                        <th class="hf-table__cell--numeric">"UPL"</th>
                        <th class="hf-table__cell--numeric">"RPL"</th>
                        <th>"Actions"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || positions.iter().map(|pos| {
                        let side_class = match pos.side {
                            crate::trading::HfSide::Buy => "hf-side-marker--buy",
                            crate::trading::HfSide::Sell => "hf-side-marker--sell",
                        };

                        view! {
                            <tr class="hf-table__row">
                                <td class="hf-table__cell--symbol">{pos.symbol.clone()}</td>
                                <td>
                                    <div class=format!("hf-side-marker {} {}", side_class, "hf-side-marker--small").trim().to_string() />
                                </td>
                                <td class="hf-table__cell--numeric">{pos.quantity.to_string()}</td>
                                <td class="hf-table__cell--numeric">{format!("{:.2}", pos.avg_price)}</td>
                                <td class="hf-table__cell--numeric">{format!("{:.2}", pos.ltp)}</td>
                                <td class="hf-table__cell--numeric">
                                    <HfPLNumber value=pos.unrealized_pnl format=crate::trading::HfPLNumberFormat::Currency />
                                </td>
                                <td class="hf-table__cell--numeric">
                                    <HfPLNumber value=pos.realized_pnl format=crate::trading::HfPLNumberFormat::Currency />
                                </td>
                                <td class="hf-table__cell--actions">
                                    <button class="hf-table__action-btn hf-table__action-btn--square-off">"Square"</button>
                                    <button class="hf-table__action-btn hf-table__action-btn--edit">"Edit"</button>
                                </td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        </div>
    }
}