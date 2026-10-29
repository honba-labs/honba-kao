//! `HfPriceSnapInput` — price input with a currency prefix and a `Mid`
//! button that snaps the value to the midpoint of the best bid and ask.

use std::str::FromStr;

use nsh_core::prelude::*;
use rust_decimal::Decimal;

use crate::primitive::base::button::{
    ButtonSize, ButtonVariant, HfButton, HfButtonEvent, HfButtonProps,
};
use crate::primitive::base::input::{HfInput, HfInputEvent, HfInputProps};
use crate::theme::tokens::HonbaStyle;

/// The event of [`HfPriceSnapInput`]: a proposed new value. The caller decides
/// whether to accept it and passes the value back down through `value`.
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfPriceSnapInputEvent {
    /// The proposed new value (typed, or the midpoint from `Mid`).
    Change(String),
}

/// Props for [`HfPriceSnapInput`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfPriceSnapInputProps {
    pub value: Prop<String>,
    pub currency_symbol: Prop<String>,
    /// Best bid, pre-formatted by the caller.
    #[prop(default)]
    pub best_bid: Prop<Option<String>>,
    /// Best ask, pre-formatted by the caller.
    #[prop(default)]
    pub best_ask: Prop<Option<String>>,
    #[prop(default = false)]
    pub disabled: Prop<bool>,
}

/// Parse an optional pre-formatted decimal, `None` when absent or unparsable.
fn parse_opt(s: Option<String>) -> Option<Decimal> {
    s.and_then(|s| Decimal::from_str(&s).ok())
}

/// Currency-prefixed price input with a `Mid` snap button.
#[component]
pub fn HfPriceSnapInput(
    props: HfPriceSnapInputProps,
    cx: Scope<HfPriceSnapInputEvent>,
) -> impl IntoElement {
    let disabled = props.disabled.get_untracked();
    let bid = parse_opt(props.best_bid.get_untracked());
    let ask = parse_opt(props.best_ask.get_untracked());
    let mid_available = matches!((bid, ask), (Some(_), Some(_)));
    let mid_disabled = disabled || !mid_available;

    let prefix = text(props.currency_symbol.get_untracked())
        .text_muted()
        .text_size(px(12.0));

    let input = HfInput::element(HfInputProps {
        value: props.value,
        placeholder: Prop::from(Some("0.00".to_string())),
        disabled: Prop::from(disabled),
        ..Default::default()
    })
    .on_event(move |event: &HfInputEvent| match event {
        HfInputEvent::Change(s) => cx.emit(HfPriceSnapInputEvent::Change(s.clone())),
    });

    let mid_button = HfButton::element(HfButtonProps {
        variant: Prop::from(Some(ButtonVariant::Outline)),
        size: Prop::from(Some(ButtonSize::Sm)),
        disabled: Prop::from(mid_disabled),
        aria_label: Prop::from(Some("Snap to midpoint".to_string())),
        ..Default::default()
    })
    .child(text("Mid"))
    .on_event(move |event: &HfButtonEvent| {
        if matches!(event, HfButtonEvent::Press) {
            if let (Some(bid), Some(ask)) = (bid, ask) {
                let mid = (bid + ask) / Decimal::from(2);
                cx.emit(HfPriceSnapInputEvent::Change(format!(
                    "{}",
                    mid.round_dp(2)
                )));
            }
        }
    });

    view()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .w_full()
        .app_class(Some(SharedString::from("hf-price-snap-input")))
        .child(prefix)
        .child(input.flex_1())
        .child(mid_button)
}
