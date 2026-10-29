//! `HfFundingRateTile` — crypto funding rate telemetry tile.

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfFundingRateTile`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfFundingRateTileProps {
    /// Annualized funding rate as a percent (e.g. `12.5` for 12.5%/yr). Only
    /// ever displayed and color-branched here, never fed into money math, so
    /// it stays a plain `f64`.
    pub rate: Prop<f64>,
    /// Extra Tailwind classes, applied after the tile's own.
    #[prop(default)]
    pub class: Prop<Option<String>>,
}

/// Funding-rate telemetry tile: the annualized rate, tinted emerald when
/// non-negative and rose when negative.
#[component]
pub fn HfFundingRateTile(props: HfFundingRateTileProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let rate = props.rate.get_untracked();
    let extra = props.class.get_untracked();

    let base = "hf-funding-rate";
    let classes = match extra {
        Some(c) => format!("{base} {c}"),
        None => base.to_string(),
    };

    let value = if rate >= 0.0 {
        text(format!("{rate:.2}%")).text_positive()
    } else {
        text(format!("{rate:.2}%")).text_negative()
    };

    view()
        .flex_col()
        .gap(px(4.0))
        .p(px(8.0))
        .rounded(px(4.0))
        .border_soft()
        .surface_base()
        .text_size(px(12.0))
        .app_class(Some(SharedString::from(classes)))
        .child(text("Funding Rate (annualized)").text_muted())
        .child(
            view()
                .flex_row()
                .items_baseline()
                .gap(px(4.0))
                .child(value)
                .child(text("/yr").text_muted()),
        )
}
