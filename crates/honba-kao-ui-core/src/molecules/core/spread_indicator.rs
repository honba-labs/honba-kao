//! `HfSpreadIndicator` — bid/ask spread row, absolute and in basis points.

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfSpreadIndicator`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfSpreadIndicatorProps {
    #[prop(default)]
    pub spread_abs: Prop<Option<f64>>,
    #[prop(default)]
    pub spread_bps: Prop<Option<f64>>,
}

/// Spread row: `Spread  abs (bps bps)`, `abs` alone, or `-` when unknown.
#[component]
pub fn HfSpreadIndicator(props: HfSpreadIndicatorProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let spread_abs = props.spread_abs;
    let spread_bps = props.spread_bps;

    view()
        .flex_row()
        .items_center()
        .justify_between()
        .px(px(8.0))
        .py(px(4.0))
        .surface_muted()
        .border_soft()
        .app_class(Some(SharedString::from(
            "hf-spread-indicator border-y font-mono",
        )))
        .child(text("Spread").text_muted().text_size(px(12.0)))
        .child(
            text(move || match (spread_abs.get(), spread_bps.get()) {
                (Some(abs), Some(bps)) => format!("{:.2} ({:.2} bps)", abs, bps),
                (Some(abs), None) => format!("{:.2}", abs),
                _ => "-".to_string(),
            })
            .text_muted()
            .text_size(px(12.0)),
        )
}
