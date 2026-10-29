//! `HfCointegrationBand` — cointegration / Z-score band for pair trading.

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfCointegrationBand`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfCointegrationBandProps {
    pub z_score: Prop<f64>,
    /// Extra Tailwind classes, applied after the band's own
    /// (`Element::app_class`).
    #[prop(default)]
    pub class: Prop<Option<String>>,
}

/// A Z-score readout tinted by the spread's state: short above +2, long below
/// -2, neutral otherwise.
#[component]
pub fn HfCointegrationBand(
    props: HfCointegrationBandProps,
    cx: Scope<NoEvent>,
) -> impl IntoElement {
    let _ = cx;
    let z = props.z_score.get_untracked();
    let extra = props.class.get_untracked();

    let classes = match extra {
        Some(c) => format!("hf-cointegration-band {c}"),
        None => "hf-cointegration-band".to_string(),
    };

    let (state, value) = if z > 2.0 {
        ("Short Spread", text(format!("{z:.2}")).text_negative())
    } else if z < -2.0 {
        ("Long Spread", text(format!("{z:.2}")).text_positive())
    } else {
        ("Neutral", text(format!("{z:.2}")).text_muted())
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
        .child(text("Z-Score").text_muted())
        .child(value.font_family("monospace"))
        .child(text(state).text_size(px(10.0)))
}
