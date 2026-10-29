//! `HfPcrMeter` — put/call ratio meter.

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfPcrMeter`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfPcrMeterProps {
    pub pcr: Prop<f64>,
    /// Extra Tailwind classes, applied after the meter's own.
    #[prop(default)]
    pub class: Prop<Option<String>>,
}

/// Sentiment label and its Tailwind text color for a put/call ratio.
fn level(pcr: f64) -> (&'static str, &'static str) {
    if pcr > 1.2 {
        ("Bearish", "text-rose-400")
    } else if pcr < 0.8 {
        ("Bullish", "text-emerald-400")
    } else {
        ("Neutral", "text-slate-400")
    }
}

/// A put/call ratio readout with a Bearish / Neutral / Bullish label.
#[component]
pub fn HfPcrMeter(props: HfPcrMeterProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let pcr = props.pcr;
    let extra = props.class.get_untracked();

    let classes = match extra {
        Some(c) => format!("hf-pcr-meter {c}"),
        None => "hf-pcr-meter".to_string(),
    };

    view()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.0))
        .p(px(8.0))
        .rounded(px(4.0))
        .border_soft()
        .surface_base()
        .text_size(px(12.0))
        .app_class(Some(SharedString::from(classes)))
        .child(text("PCR").text_muted())
        .child(
            text(move || format!("{:.2}", pcr.get()))
                .font_family("monospace")
                .text_high(),
        )
        .child(
            text(move || level(pcr.get()).0)
                .text_size(px(10.0))
                .app_class(move || Some(SharedString::from(level(pcr.get()).1))),
        )
}
