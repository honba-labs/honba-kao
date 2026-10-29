//! `HfUnderwaterGauge` — drawdown underwater curve gauge.

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfUnderwaterGauge`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfUnderwaterGaugeProps {
    pub drawdown_pct: Prop<f64>,
    /// Extra Tailwind classes, applied after the gauge's own
    /// (`Element::app_class`).
    #[prop(default)]
    pub class: Prop<Option<String>>,
}

/// A drawdown readout: positive while the drawdown is under 20%, negative from
/// 20% on.
#[component]
pub fn HfUnderwaterGauge(props: HfUnderwaterGaugeProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let drawdown = props.drawdown_pct.get_untracked();
    let extra = props.class.get_untracked();

    let classes = match extra {
        Some(c) => format!("hf-underwater-gauge {c}"),
        None => "hf-underwater-gauge".to_string(),
    };

    let value = if drawdown < 20.0 {
        text(format!("{drawdown:.1}%")).text_positive()
    } else {
        text(format!("{drawdown:.1}%")).text_negative()
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
        .child(text("Drawdown").text_muted())
        .child(value.font_family("monospace"))
}
