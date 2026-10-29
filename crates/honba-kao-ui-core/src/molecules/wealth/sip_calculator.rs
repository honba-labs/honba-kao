//! `HfSipCalculatorCard` — compounding projection tool. Projection math is
//! computed in `honba-kao-api-wasm::analytics`.
//! All money values use `rust_decimal::Decimal` (Zero Floats Policy); the
//! inputs cross as pre-formatted strings and are parsed back here.

use std::str::FromStr;

use honba_api_wasm::analytics::calculate_sip_projection;
use nsh_core::prelude::*;
use rust_decimal::Decimal;

use crate::primitive::base::input::{HfInputEvent, HfNumberInput, HfNumberInputProps};
use crate::theme::tokens::HonbaStyle;

/// The events of [`HfSipCalculatorCard`]: one per input, each a proposed new
/// value for the caller to accept or reject.
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfSipCalculatorEvent {
    MonthlyChange(String),
    YearsChange(String),
    AnnualReturnChange(String),
}

/// Props for [`HfSipCalculatorCard`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfSipCalculatorCardProps {
    /// Monthly installment, pre-formatted by the caller.
    pub monthly: Prop<String>,
    /// Term in years, pre-formatted by the caller.
    pub years: Prop<String>,
    /// Annual return as a percent, pre-formatted by the caller.
    pub annual_return: Prop<String>,
}

/// Parse a pre-formatted decimal, `0` when unparsable.
fn parse(s: String) -> Decimal {
    Decimal::from_str(&s).unwrap_or_default()
}

/// One labelled number field: a muted caption above the input.
fn labelled(label: &'static str, field: impl IntoElement) -> impl IntoElement {
    view()
        .flex_col()
        .gap(px(2.0))
        .child(text(label).text_muted().text_size(px(10.0)))
        .child(field)
}

/// SIP projection card: three inputs, the projected value, and the invested
/// and gains split beneath.
#[component]
pub fn HfSipCalculatorCard(
    props: HfSipCalculatorCardProps,
    cx: Scope<HfSipCalculatorEvent>,
) -> impl IntoElement {
    let monthly = parse(props.monthly.get_untracked());
    let years = parse(props.years.get_untracked());
    let annual_return = parse(props.annual_return.get_untracked());
    let sip = calculate_sip_projection(monthly, years, annual_return);

    let monthly_input = HfNumberInput::element(HfNumberInputProps {
        value: props.monthly,
        placeholder: Prop::from(Some("0".to_string())),
        ..Default::default()
    })
    .on_event(move |event: &HfInputEvent| match event {
        HfInputEvent::Change(s) => cx.emit(HfSipCalculatorEvent::MonthlyChange(s.clone())),
    });

    let years_input = HfNumberInput::element(HfNumberInputProps {
        value: props.years,
        placeholder: Prop::from(Some("0".to_string())),
        min: Prop::from(Some("0".to_string())),
        step: Prop::from(Some("1".to_string())),
        ..Default::default()
    })
    .on_event(move |event: &HfInputEvent| match event {
        HfInputEvent::Change(s) => cx.emit(HfSipCalculatorEvent::YearsChange(s.clone())),
    });

    let return_input = HfNumberInput::element(HfNumberInputProps {
        value: props.annual_return,
        placeholder: Prop::from(Some("0".to_string())),
        min: Prop::from(Some("0".to_string())),
        step: Prop::from(Some("0.1".to_string())),
        ..Default::default()
    })
    .on_event(move |event: &HfInputEvent| match event {
        HfInputEvent::Change(s) => cx.emit(HfSipCalculatorEvent::AnnualReturnChange(s.clone())),
    });

    view()
        .flex_col()
        .gap(px(12.0))
        .p(px(12.0))
        .rounded_lg()
        .border(px(1.0))
        .border_soft()
        .surface_base()
        .app_class(Some(SharedString::from("hf-sip-calculator")))
        .child(
            text("SIP Calculator")
                .text_high()
                .text_size(px(12.0))
                .uppercase(),
        )
        .child(
            view()
                .flex_row()
                .gap(px(12.0))
                .child(
                    view()
                        .flex_1()
                        .child(labelled("Monthly (₹)", monthly_input)),
                )
                .child(view().flex_1().child(labelled("Years", years_input)))
                .child(
                    view()
                        .flex_1()
                        .child(labelled("Return % p.a.", return_input)),
                ),
        )
        .child(
            view()
                .flex_col()
                .gap(px(2.0))
                .child(text("Projected Value (₹)").text_muted().text_size(px(10.0)))
                .child(
                    text(format!("{:.2}", sip.projected))
                        .text_high()
                        .text_size(px(14.0))
                        .app_class(Some(SharedString::from("font-mono"))),
                ),
        )
        .child(
            view()
                .flex_row()
                .justify_between()
                .text_muted()
                .text_size(px(10.0))
                .child(text(format!("Invested: {:.2}", sip.invested)))
                .child(
                    text(format!("Gains: {:.2}", sip.gains))
                        .app_class(Some(SharedString::from("text-emerald-400"))),
                ),
        )
}
