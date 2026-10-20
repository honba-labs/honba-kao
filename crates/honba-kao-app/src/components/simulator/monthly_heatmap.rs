//! The monthly heatmap — the Leptos port of honba-fe
//! `features/simulator/components/monthly-heatmap.tsx` (port phase P5).
//!
//! Month × year realised-PnL matrix from the journal. Intensity scales with
//! the month's return on the running realised equity; the color-mix
//! formula (`12% + intensity × 68%`) replaces fe's rgba alpha. The matrix
//! reconciles with the realised total by construction (each cell holds an
//! `Option<f64>` return that is `Some` only once equity was positive); the
//! wire `total_return` is shown alongside, labelled, rather than asserted
//! to equal a *realised* figure it need not match (risk R5).

use leptos::prelude::*;

use crate::sim::analytics::MonthlyHeatmap;

/// fe's `MONTHS` header row.
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// The empty-slot background (fe: `transparent`) and fe's `cellColor` —
/// alpha `12% + |pct| / maxAbs × 68%` over the gain or loss token; a flat
/// surface token for a present cell with no computable return.
fn cell_background(return_pct: Option<f64>, max_abs: f64) -> String {
    let Some(pct) = return_pct else {
        return "var(--surface-3)".to_string();
    };
    if max_abs <= 0.0 {
        return "var(--surface-3)".to_string();
    }
    let intensity = (pct.abs() / max_abs).min(1.0);
    // 12% .. 80% as an integer percent for color-mix.
    let alpha = (0.12 + intensity * 0.68) * 100.0;
    let token = if pct >= 0.0 { "var(--pnl-gain)" } else { "var(--pnl-loss)" };
    format!("color-mix(in srgb, {token} {alpha:.0}%, transparent)")
}

/// One heatmap cell's display text: the signed return, or `·` when trades
/// happened but returns were not computable, or empty when no trades closed.
fn cell_text(return_pct: Option<f64>, has_cell: bool) -> String {
    if !has_cell {
        return String::new();
    }
    return_pct.map_or_else(
        || "·".to_string(),
        |value| format!("{}{:.1}%", if value > 0.0 { "+" } else { "" }, value * 100.0),
    )
}

/// The month × year matrix plus its footer sentences (`MonthlyHeatmap`).
#[component]
pub fn MonthlyHeatmap(
    /// The IST month × year matrix, derived per `store.analytics_bundle`.
    #[prop(into)]
    heatmap: Signal<MonthlyHeatmap>,
    /// The payload's `total_return`, for the labelled comparison (`None` when
    /// the run reported no metrics).
    #[prop(into)]
    wire_total_return: Signal<Option<f64>>,
) -> impl IntoView {
    view! {
        <div class="sim-heatmap" data-testid="monthly-heatmap">
            {move || {
                let heatmap = heatmap.get();
                if heatmap.years.is_empty() {
                    view! {
                        <div class="sim-heatmap__empty">
                            "No closed round trips, so there is no monthly breakdown."
                        </div>
                    }
                        .into_any()
                } else {
                    // Highest |return_pct| sets the intensity scale.
                    let max_abs = heatmap
                        .cells
                        .iter()
                        .map(|cell| cell.return_pct.unwrap_or(0.0).abs())
                        .fold(0.0f64, f64::max);

                    let header = MONTHS
                        .iter()
                        .map(|month| {
                            view! { <span class="sim-heatmap__month">{*month}</span> }
                        })
                        .collect_view();

                    let rows = heatmap
                        .years
                        .iter()
                        .map(|year| {
                            let year = *year;
                            // One owned slice per year: no borrows escape the
                            // render closure.
                            let year_cells: Vec<(usize, crate::sim::analytics::HeatmapCell)> =
                                heatmap
                                    .cells
                                    .iter()
                                    .filter(|cell| cell.year == year)
                                    .map(|cell| (cell.month as usize, *cell))
                                    .collect();

                            let months = (0..12)
                                .map(|month| {
                                    let cell = year_cells
                                        .iter()
                                        .find(|(cell_month, _)| *cell_month == month)
                                        .map(|(_, cell)| *cell);
                                    let (style, text, is_empty, title) = match cell {
                                        Some(cell) => {
                                            let background =
                                                cell_background(cell.return_pct, max_abs);
                                            let text = cell_text(cell.return_pct, true);
                                            let title = format!(
                                                "{} {year}: {}{:.2} over {} trip(s)",
                                                MONTHS[month],
                                                if cell.pnl >= 0.0 { "+" } else { "" },
                                                cell.pnl,
                                                cell.trades,
                                            );
                                            (background, text, false, title)
                                        }
                                        None => (
                                            "transparent".to_string(),
                                            String::new(),
                                            true,
                                            format!("{} {year}: no round trips", MONTHS[month]),
                                        ),
                                    };
                                    view! {
                                        <div
                                            class="sim-heatmap__cell"
                                            class=("sim-heatmap__cell--empty", is_empty)
                                            style:background-color=style
                                            title=title
                                        >
                                            {text}
                                        </div>
                                    }
                                })
                                .collect_view();

                            view! {
                                <>
                                    <span class="sim-heatmap__year">{year}</span>
                                    {months}
                                </>
                            }
                        })
                        .collect_view();

                    let total = heatmap.total_pnl;
                    let total_class = if total >= 0.0 {
                        "sim-heatmap__total--gain"
                    } else {
                        "sim-heatmap__total--loss"
                    };

                    view! {
                        <div class="sim-heatmap__grid">
                            <span />
                            {header}
                            {rows}
                        </div>
                        <div class="sim-heatmap__foot">
                            <span>
                                "Realised total: "
                                <strong class=total_class>
                                    {format!("{}{:.2}", if total >= 0.0 { "+" } else { "" }, total)}
                                </strong>
                            </span>
                            <span>
                                "Realised return: "
                                <strong>
                                    {heatmap
                                        .total_return_pct
                                        .map_or_else(|| "—".to_string(), |value| format!("{:.2}%", value * 100.0))}
                                </strong>
                            </span>
                            <span class="sim-heatmap__foot-index">
                                "Payload total return: "
                                {move || {
                                    wire_total_return
                                        .get()
                                        .map_or_else(|| "—".to_string(), |value| format!("{:.2}%", value * 100.0))
                                }}
                                " (reported; a realised matrix need not match a mark-to-market figure)"
                            </span>
                        </div>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_background_scales_intensity_over_the_gain_and_loss_tokens() {
        // A present cell with no computable return reads the flat surface.
        assert_eq!(cell_background(None, 0.5), "var(--surface-3)");
        // Zero spread across the matrix: flat surface, no color-mix crash.
        assert_eq!(cell_background(Some(0.25), 0.0), "var(--surface-3)");
        // Full-intensity gains hit the 80% alpha ceiling.
        let max = cell_background(Some(0.04), 0.04);
        assert_eq!(max, "color-mix(in srgb, var(--pnl-gain) 80%, transparent)");
        // Half-intensity losses land mid-scale.
        let mid = cell_background(Some(-0.01), 0.04);
        assert_eq!(
            mid,
            "color-mix(in srgb, var(--pnl-loss) 29%, transparent)"
        );
    }

    #[test]
    fn cell_text_reports_signed_pcts_and_the_absent_marker() {
        assert_eq!(cell_text(Some(0.012_3), true), "+1.2%");
        assert_eq!(cell_text(Some(-0.034), true), "-3.4%");
        assert_eq!(cell_text(None, true), "·"); // trades closed, return not computable
        assert_eq!(cell_text(None, false), ""); // no round trips at all
    }
}
