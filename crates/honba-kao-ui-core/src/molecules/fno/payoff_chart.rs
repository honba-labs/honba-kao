//! `HfOptionPayoffChart` — multi-leg payoff diagram at maturity.
//! Compute lives in `honba-kao-api-wasm`; this is the UI molecule only.
//! Draws a horizontal breakeven axis (PnL = 0) across the diagram when zero
//! lies inside the data's range.

use nsh_core::element::{NoEvent, RowAccessor, Rows};
use nsh_core::prelude::*;

use crate::primitive::base::plot::{
    Curve, HfGrid, HfGridProps, HfLine, HfLineProps, HfPlot, HfPlotProps,
};
use crate::theme::tokens::HonbaStyle;

/// One point of a payoff curve: the underlying's `spot` and the strategy's
/// `pnl` at that spot.
#[derive(Debug, Clone, PartialEq, Default, JsObject)]
pub struct PayoffPoint {
    pub spot: f64,
    pub pnl: f64,
}

/// Props for [`HfOptionPayoffChart`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfOptionPayoffChartProps {
    #[prop(default)]
    pub points: Prop<Vec<PayoffPoint>>,
    /// Extra Tailwind classes, applied after the chart's own.
    #[prop(default)]
    pub class: Prop<Option<String>>,
}

/// The rows of a payoff curve: the point's index (`idx`, the x axis) and its
/// `pnl` (the y axis).
fn payoff_rows(points: &[PayoffPoint]) -> Rows {
    let idx: Vec<f64> = (0..points.len()).map(|i| i as f64).collect();
    let pnl: Vec<f64> = points.iter().map(|p| p.pnl).collect();
    Rows::from_columns(
        points.len(),
        [
            (SharedString::from("idx"), idx),
            (SharedString::from("pnl"), pnl),
        ],
    )
}

/// The `(min, max)` of the points' `pnl`, widened to a unit when flat, or
/// `None` without points.
fn pnl_range(points: &[PayoffPoint]) -> Option<(f64, f64)> {
    if points.is_empty() {
        return None;
    }
    let mn = points.iter().map(|p| p.pnl).fold(f64::INFINITY, f64::min);
    let mx = points
        .iter()
        .map(|p| p.pnl)
        .fold(f64::NEG_INFINITY, f64::max);
    let mx = if (mx - mn).abs() < f64::EPSILON {
        mn + 1.0
    } else {
        mx
    };
    Some((mn, mx))
}

/// The breakeven grid: `[0.0]` (a line at PnL = 0) when zero lies inside the
/// range, else nothing.
fn breakeven_grid(points: &[PayoffPoint]) -> Vec<f64> {
    match pnl_range(points) {
        Some((mn, mx)) if mn <= 0.0 && 0.0 <= mx => vec![0.0],
        _ => Vec::new(),
    }
}

/// Multi-leg payoff chart: a linear curve of `pnl` against the point index,
/// and a dashed breakeven line at PnL = 0 when zero is in range.
///
/// The `points` prop is read reactively: the plot's rows and the breakeven
/// grid update when it changes.
///
/// The original Leptos version also printed a "Breakeven axis (PnL = 0)"
/// caption under the chart. It is dropped: the caption would have to show and
/// hide with the breakeven grid, and the shared `HfPlot` has no layout for a
/// caption beside its marks yet.
#[component]
pub fn HfOptionPayoffChart(
    props: HfOptionPayoffChartProps,
    cx: Scope<NoEvent>,
) -> impl IntoElement {
    let _ = cx;
    let class = props.class;
    let points = props.points;

    let rows = (move || payoff_rows(&points.get())).into_reactive();
    let breakeven = (move || breakeven_grid(&points.get())).into_reactive();

    view()
        .w_full()
        .h(px(192.0))
        .rounded(px(4.0))
        .border(px(1.0))
        .border_soft()
        .surface_base()
        .child(
            HfPlot::element(HfPlotProps {
                data: Prop::from(rows),
                x: Prop::from(Some(RowAccessor::from("idx"))),
                y: Prop::from(Some(RowAccessor::from("pnl"))),
                ..Default::default()
            })
            .child(HfLine::element(HfLineProps {
                curve: Prop::from(Curve::Linear),
                ..Default::default()
            }))
            .child(HfGrid::element(HfGridProps {
                y: Prop::from(breakeven),
                ..Default::default()
            })),
        )
        .app_class(move || class.get().map(SharedString::from))
}
