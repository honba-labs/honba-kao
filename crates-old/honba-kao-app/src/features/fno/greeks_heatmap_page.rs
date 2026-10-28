//! Greeks heatmap (Phase 3.4a) at `/fno/greeks`.
//!
//! Call-side Delta, Gamma, Theta and Vega for ATM ± 5 strikes. Values come
//! from simple display formulas of distance from spot, not Black-Scholes.
//! Each cell is shaded by its magnitude relative to the column maximum.

use super::fmt::group_int;
use honba_kao_ui_core::{
    HfPageHeader, HfPanel, HfPanelHeader, HfTable, HfTableBody, HfTableCell, HfTableHeader,
    HfTableHeaderCell, HfTableRow,
};
use leptos::prelude::*;

/// Underlying spot in whole rupees.
const SPOT: i64 = 24_850;
/// Strike spacing in whole rupees.
const STRIKE_STEP: i64 = 50;
/// Eleven strikes, ATM - 5 through ATM + 5.
const HALF_SPAN: i64 = 5;
const ROWS: usize = (2 * HALF_SPAN + 1) as usize;

const GREEKS: [&str; 4] = ["Delta", "Gamma", "Theta", "Vega"];

/// Display-only call Greeks for `offset` strikes from ATM.
fn call_greeks(offset: i64) -> [f64; 4] {
    let a = offset.abs() as f64;
    let delta = 1.0 / (1.0 + (0.45 * offset as f64).exp());
    let gamma = 0.0016 * (1.0 - a / 6.0);
    let theta = -(0.4 + 0.15 * (HALF_SPAN as f64 - a));
    let vega = 12.0 - 0.8 * a;
    [delta, gamma, theta, vega]
}

/// Shade bucket 1 (faint) ..= 5 (strong), by `value` relative to `max_abs`.
fn shade(value: f64, max_abs: f64) -> u8 {
    if max_abs <= 0.0 {
        return 1;
    }
    let ratio = value.abs() / max_abs * 100.0;
    match ratio {
        r if r < 20.0 => 1,
        r if r < 40.0 => 2,
        r if r < 60.0 => 3,
        r if r < 80.0 => 4,
        _ => 5,
    }
}

/// Value text for a Greek column, with the precision the column needs.
fn fmt_greek(idx: usize, value: f64) -> String {
    match idx {
        1 => format!("{value:.4}"),
        _ => format!("{value:.2}"),
    }
}

#[component]
pub fn GreeksHeatmapPage() -> impl IntoView {
    let rows: Vec<(i64, [f64; 4])> = (0..ROWS)
        .map(|i| {
            let offset = i as i64 - HALF_SPAN;
            (SPOT + offset * STRIKE_STEP, call_greeks(offset))
        })
        .collect();

    // Largest absolute value per Greek column, for per-column shading.
    let max_abs: [f64; 4] = std::array::from_fn(|col| {
        rows.iter()
            .map(|(_, g)| g[col].abs())
            .fold(0.0_f64, f64::max)
    });

    view! {
        <div class="console fno">
            <HfPageHeader title="Greeks Heatmap" subtitle="NIFTY · Derivatives · mock data" />

            <HfPanel class="fno__greeks">
                <HfPanelHeader title="Call Greeks by Strike" />
                <div class="fno__toolbar">
                    <span class="placeholder__note">
                        "Shaded by magnitude within each Greek · mock data"
                    </span>
                    <div class="fno__legend">
                        <span class="fno__legend-label">"Low"</span>
                        {(1..=5u8)
                            .map(|s| view! { <span class=format!("fno__heat fno__heat--s{s} fno__swatch")></span> })
                            .collect_view()}
                        <span class="fno__legend-label">"High"</span>
                    </div>
                </div>

                <div class="fno__scroll">
                    <HfTable class="fno__table">
                        <HfTableHeader>
                            <HfTableRow>
                                <HfTableHeaderCell class="tnum">"Strike"</HfTableHeaderCell>
                                {GREEKS
                                    .iter()
                                    .map(|g| view! { <HfTableHeaderCell class="tnum">{*g}</HfTableHeaderCell> })
                                    .collect_view()}
                            </HfTableRow>
                        </HfTableHeader>
                        <HfTableBody>
                            {rows
                                .into_iter()
                                .enumerate()
                                .map(|(i, (strike, greeks))| {
                                    let class = if i as i64 == HALF_SPAN { "fno__row--atm" } else { "" };
                                    view! {
                                        <HfTableRow class=class>
                                            <HfTableCell class="tnum fno__strike">{group_int(strike)}</HfTableCell>
                                            {greeks
                                                .iter()
                                                .enumerate()
                                                .map(|(col, &value)| {
                                                    let s = shade(value, max_abs[col]);
                                                    view! {
                                                        <HfTableCell class=format!("tnum fno__heat fno__heat--s{s}")>
                                                            {fmt_greek(col, value)}
                                                        </HfTableCell>
                                                    }
                                                })
                                                .collect_view()}
                                        </HfTableRow>
                                    }
                                })
                                .collect_view()}
                        </HfTableBody>
                    </HfTable>
                </div>
            </HfPanel>
        </div>
    }
}
