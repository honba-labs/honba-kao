//! `HfEstimationFields` — order value, estimated fee and margin requirement
//! rows for an order ticket.

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfEstimationFields`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfEstimationFieldsProps {
    pub notional: Prop<f64>,
    pub est_fee: Prop<f64>,
    pub margin_req: Prop<f64>,
    /// Currency prefix for each value; `None` means `₹`.
    #[prop(default)]
    pub currency_prefix: Prop<Option<String>>,
}

/// One label/value row: the label muted on the left, the value mono on the right.
fn row(label: &'static str, value: String) -> impl IntoElement {
    view()
        .flex_row()
        .justify_between()
        .text_muted()
        .child(text(label))
        .child(
            text(value)
                .text_high()
                .app_class(Some(SharedString::from("font-mono"))),
        )
}

/// Three read-only rows: order value, estimated fee (1 bp), margin required.
/// Values are read once, not reactively.
#[component]
pub fn HfEstimationFields(props: HfEstimationFieldsProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let curr = props
        .currency_prefix
        .get_untracked()
        .unwrap_or_else(|| "₹".to_string());
    let notional = props.notional.get_untracked();
    let est_fee = props.est_fee.get_untracked();
    let margin_req = props.margin_req.get_untracked();

    view()
        .flex_col()
        .gap(px(4.0))
        .p(px(8.0))
        .rounded_md()
        .border(px(1.0))
        .border_soft()
        .surface_muted()
        .text_size(px(12.0))
        .app_class(Some(SharedString::from("hf-estimation-fields")))
        .child(row("Order Value", format!("{curr}{notional:.2}")))
        .child(row("Est. Fee (1 bp)", format!("{curr}{est_fee:.2}")))
        .child(row("Margin Required", format!("{curr}{margin_req:.2}")))
}
