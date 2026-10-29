//! `HfInput`, `HfNumberInput` — wrapping [`nsh_core::element::input`].
//!
//! `nsh_core::element::input` makes the native text field (UITextField /
//! EditText); the `input()` free function is missing from
//! `nsh_core::prelude` (unlike `Input`, the element type it returns, which
//! the prelude does export), so it is imported explicitly here.
//!
//! `nsh_core::element::Input::on_change` reports the new text as `&str`,
//! not a `Fn(String)` callback prop: a `#[derive(Props)]` struct must be
//! `Clone + PartialEq` (`nsh_core::element::Props: Clone + PartialEq`),
//! which a boxed closure can never satisfy. [`HfInputEvent`] carries the
//! change the same way [`crate::primitive::base::button::HfButtonEvent`]
//! carries a press: composing code hears edits with `.on_event(move
//! |event: &HfInputEvent| ..)` on the element these constructors make.

use nsh_core::element::input;
use nsh_core::prelude::*;

/// The event of [`HfInput`] / [`HfNumberInput`]: the field's text changed.
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfInputEvent {
    /// The new text.
    #[args(value)]
    Change(String),
}

/// Props for [`HfInput`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfInputProps {
    pub value: Prop<String>,
    #[prop(default)]
    pub placeholder: Prop<Option<String>>,
    #[prop(default = false)]
    pub disabled: Prop<bool>,
}

/// Foundational styled text input element with focus borders.
#[component]
pub fn HfInput(props: HfInputProps, cx: Scope<HfInputEvent>) -> impl IntoElement {
    let value = props.value;
    let placeholder_val = props.placeholder.get_untracked().unwrap_or_default();

    input()
        .value(move || value.get())
        .disabled(props.disabled)
        .placeholder(placeholder_val)
        .rounded_md()
        .border(px(1.0))
        .px(px(3.0))
        .py(px(1.5))
        .on_change(move |text: &str| cx.emit(HfInputEvent::Change(text.to_string())))
}

/// Props for [`HfNumberInput`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfNumberInputProps {
    pub value: Prop<String>,
    #[prop(default)]
    pub placeholder: Prop<Option<String>>,
    #[prop(default)]
    pub min: Prop<Option<String>>,
    #[prop(default)]
    pub max: Prop<Option<String>>,
    #[prop(default)]
    pub step: Prop<Option<String>>,
    #[prop(default = false)]
    pub disabled: Prop<bool>,
}

/// Numeric input with step controls.
/// Values are `rust_decimal::Decimal` (Zero Floats Policy) — never `f64`.
#[component]
pub fn HfNumberInput(props: HfNumberInputProps, cx: Scope<HfInputEvent>) -> impl IntoElement {
    let value = props.value;
    let _ = props.min;
    let _ = props.max;
    let _ = props.step;

    input()
        .value(move || value.get())
        .disabled(props.disabled)
        .placeholder(props.placeholder.get_untracked().unwrap_or_default())
        .rounded_md()
        .border(px(1.0))
        .px(px(3.0))
        .py(px(1.5))
        .on_change(move |text: &str| cx.emit(HfInputEvent::Change(text.to_string())))
}
