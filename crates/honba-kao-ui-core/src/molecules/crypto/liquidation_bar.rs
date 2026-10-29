//! `HfLiquidationProgressBar` — tri-tier collateral safety meter with
//! liquidation buffer indicator. Ported from the restructure spec (Section
//! 7.3) and `secured-finance-app` molecule `LiquidationProgressBar`.
//!
//! `current_utilization`/`liquidation_threshold` are percentages used only
//! for the bar-width styling and a percent "buffer" readout, never fed into
//! money math, so both stay plain `f64`. `est_liquidation_price` is a real
//! money price: it crosses as a caller-pre-formatted `Prop<Option<String>>`
//! and is parsed to `Decimal` here only for display formatting.

use std::str::FromStr;

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;
use rust_decimal::Decimal;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfLiquidationProgressBar`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfLiquidationProgressBarProps {
    /// 0.0 to 100.0%.
    pub current_utilization: Prop<f64>,
    /// e.g. `85.0` for 85%.
    pub liquidation_threshold: Prop<f64>,
    /// Pre-formatted by the caller; parsed to `Decimal` for display. `None`
    /// renders "N/A".
    #[prop(default)]
    pub est_liquidation_price: Prop<Option<String>>,
}

/// Tri-tier collateral safety meter: a filled progress track against a
/// liquidation threshold tick, with the buffer and estimated liquidation
/// price beneath.
#[component]
pub fn HfLiquidationProgressBar(
    props: HfLiquidationProgressBarProps,
    cx: Scope<NoEvent>,
) -> impl IntoElement {
    let _ = cx;
    let utilization = props.current_utilization.get_untracked();
    let threshold = props.liquidation_threshold.get_untracked();
    let clamped = utilization.clamp(0.0, 100.0);
    let est_liq_price = props
        .est_liquidation_price
        .get_untracked()
        .map(|s| Decimal::from_str(&s).unwrap_or_default());

    let (header_value, fill_color) = if utilization < 60.0 {
        (
            text(format!("{utilization:.1}%")).text_positive(),
            AdaptiveColor::rgb(0x10b981, 0x34d399),
        )
    } else if utilization < 80.0 {
        (
            text(format!("{utilization:.1}%")).text_warning(),
            AdaptiveColor::rgb(0xf59e0b, 0xfbbf24),
        )
    } else {
        (
            text(format!("{utilization:.1}%")).text_negative(),
            AdaptiveColor::rgb(0xef4444, 0xf87171),
        )
    };

    let buffer = (threshold - utilization).max(0.0);
    let liq_price_text = match est_liq_price {
        Some(amount) => format!("Est. Liq Px: ${amount:.2}"),
        None => "Est. Liq Px: N/A".to_string(),
    };

    view()
        .flex_col()
        .gap(px(4.0))
        .p(px(8.0))
        .rounded(px(4.0))
        .surface_base()
        .border_soft()
        .app_class(Some(SharedString::from("hf-liquidation-progress-bar")))
        .child(
            view()
                .flex_row()
                .justify_between()
                .text_size(px(12.0))
                .child(text("Margin Utilization / Liquidation Risk").text_muted())
                .child(header_value),
        )
        .child(
            view()
                .relative()
                .w_full()
                .h(px(8.0))
                .rounded(px(4.0))
                .surface_muted()
                .overflow_hidden()
                .app_class(Some(SharedString::from("hf-progress-track")))
                .child(
                    view()
                        .h_full()
                        .w(relative((clamped / 100.0) as f32))
                        .bg(fill_color)
                        .app_class(Some(SharedString::from("hf-progress-fill"))),
                )
                .child(
                    view()
                        .absolute()
                        .top(px(0.0))
                        .h_full()
                        .left(relative((threshold.clamp(0.0, 100.0) / 100.0) as f32))
                        .w(px(2.0))
                        .bg(AdaptiveColor::rgb(0xf43f5e, 0xfb7185))
                        .app_class(Some(SharedString::from("hf-threshold-tick"))),
                ),
        )
        .child(
            view()
                .flex_row()
                .justify_between()
                .text_muted()
                .text_size(px(11.0))
                .child(text(format!("Buffer: +{buffer:.1}%")))
                .child(text(liq_price_text)),
        )
}
