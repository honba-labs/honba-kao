//! `HfQuantityChipInput` — quantity input with percentage chips
//! (`25%`, `50%`, `75%`, `100%`) and a `Max` button.
//! Ported from `secured-finance-app` molecule patterns and the restructure spec
//! (Section 7.2).

use std::str::FromStr;

use nsh_core::prelude::*;
use rust_decimal::Decimal;

use crate::primitive::base::button::{
    ButtonSize, ButtonVariant, HfButton, HfButtonEvent, HfButtonProps, HfChip, HfChipProps,
};
use crate::primitive::base::input::{HfInput, HfInputEvent, HfInputProps};

/// The event of [`HfQuantityChipInput`]: a proposed new value. The caller decides
/// whether to accept it and passes the value back down through `value`.
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfQuantityChipInputEvent {
    /// The proposed new value (typed, or a share of `max_quantity`).
    Change(String),
}

/// Props for [`HfQuantityChipInput`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfQuantityChipInputProps {
    pub value: Prop<String>,
    /// Maximum quantity, pre-formatted by the caller.
    pub max_quantity: Prop<String>,
    #[prop(default = false)]
    pub disabled: Prop<bool>,
}

/// Emit `max_quantity * pct / 100`, rounded to 4dp. Emits nothing when
/// `max_quantity` does not parse or is not positive.
fn emit_share(max_quantity: Prop<String>, pct: i64, cx: Scope<HfQuantityChipInputEvent>) {
    let max_qty = match Decimal::from_str(&max_quantity.get_untracked()) {
        Ok(v) if v > Decimal::ZERO => v,
        _ => return,
    };
    // `Decimal::new(pct, 2)` is `pct / 100` exactly, no float involved.
    let target = max_qty * Decimal::new(pct, 2);
    cx.emit(HfQuantityChipInputEvent::Change(format!(
        "{}",
        target.round_dp(4)
    )));
}

/// Quantity input, a `Max` button, and a row of percentage chips.
#[component]
pub fn HfQuantityChipInput(
    props: HfQuantityChipInputProps,
    cx: Scope<HfQuantityChipInputEvent>,
) -> impl IntoElement {
    let input = HfInput::element(HfInputProps {
        value: props.value,
        placeholder: Prop::from(Some("0.00".to_string())),
        disabled: props.disabled,
        ..Default::default()
    })
    .flex_1()
    .on_event(move |event: &HfInputEvent| match event {
        HfInputEvent::Change(s) => cx.emit(HfQuantityChipInputEvent::Change(s.clone())),
    });

    let max_qty = props.max_quantity;
    let max_button = HfButton::element(HfButtonProps {
        variant: Prop::from(Some(ButtonVariant::Ghost)),
        size: Prop::from(Some(ButtonSize::Sm)),
        ..Default::default()
    })
    .child(text("Max"))
    .on_event(move |event: &HfButtonEvent| {
        if matches!(event, HfButtonEvent::Press) {
            emit_share(max_qty, 100, cx);
        }
    });

    let chip = |label: &'static str, pct: i64| {
        HfChip::element(HfChipProps::default())
            .child(text(label))
            .on_event(move |event: &HfButtonEvent| {
                if matches!(event, HfButtonEvent::Press) {
                    emit_share(max_qty, pct, cx);
                }
            })
    };

    view()
        .flex_col()
        .gap(px(6.0))
        .w_full()
        .app_class(Some(SharedString::from("hf-quantity-chip-input")))
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(6.0))
                .app_class(Some(SharedString::from("hf-qty-input-row")))
                .child(input)
                .child(max_button),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(4.0))
                .app_class(Some(SharedString::from("hf-qty-chips-row")))
                .child(chip("25%", 25))
                .child(chip("50%", 50))
                .child(chip("75%", 75))
                .child(chip("100%", 100)),
        )
}
