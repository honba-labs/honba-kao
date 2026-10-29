//! `HfBacktestMetricCard` — single backtest performance metric tile.

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfBacktestMetricCard`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfBacktestMetricCardProps {
    pub label: Prop<String>,
    pub value: Prop<String>,
    #[prop(default = true)]
    pub positive: Prop<bool>,
}

/// A backtest metric tile: a small uppercase label over a monospace value.
#[component]
pub fn HfBacktestMetricCard(
    props: HfBacktestMetricCardProps,
    cx: Scope<NoEvent>,
) -> impl IntoElement {
    let _ = cx;
    let label = props.label.get_untracked();
    let value = props.value.get_untracked();
    let positive = props.positive.get_untracked();

    // The Leptos version tinted an empty span, so the tint never showed. The
    // tinted empty line is kept so the tile keeps its height.
    let tint = if positive {
        text("").text_positive()
    } else {
        text("").text_negative()
    };

    view()
        .surface_base()
        .border_soft()
        .rounded(px(4.0))
        .p(px(12.0))
        .app_class(Some(SharedString::from("hf-backtest-card")))
        .child(
            text(label)
                .text_muted()
                .text_size(px(10.0))
                .uppercase()
                .letter_spacing(px(0.8)),
        )
        .child(
            text(value)
                .text_high()
                .font_family("monospace")
                .text_size(px(14.0)),
        )
        .child(tint.text_size(px(10.0)))
}
