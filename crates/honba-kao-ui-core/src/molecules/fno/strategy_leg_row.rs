//! `HfStrategyLegRow` — leg builder item with buy/sell side, strike, expiry,
//! and lot multiplier.

use std::str::FromStr;

use nsh_core::prelude::*;
use rust_decimal::Decimal;

use crate::primitive::base::button::{HfButtonEvent, HfCloseButton, HfCloseButtonProps};
use crate::theme::tokens::HonbaStyle;

/// One leg of a strategy. `strike` and `lots` are pre-formatted decimals.
#[derive(Clone, Debug, Default, PartialEq, JsObject)]
pub struct StrategyLeg {
    pub side: String, // "BUY" / "SELL"
    pub strike: String,
    pub expiry: String,
    pub lots: String,
}

/// The events of [`HfStrategyLegRow`].
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfStrategyLegRowEvent {
    /// The remove button was pressed.
    Remove,
}

/// Props for [`HfStrategyLegRow`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfStrategyLegRowProps {
    pub leg: Prop<StrategyLeg>,
}

/// Parse a pre-formatted decimal and round it to `dp` places for display.
fn round_display(raw: &str, dp: u32) -> String {
    Decimal::from_str(raw)
        .unwrap_or_default()
        .round_dp(dp)
        .to_string()
}

/// One strategy leg: side × lots, strike, expiry, and a remove button.
#[component]
pub fn HfStrategyLegRow(
    props: HfStrategyLegRowProps,
    cx: Scope<HfStrategyLegRowEvent>,
) -> impl IntoElement {
    let leg = props.leg;

    let side = view()
        .flex_row()
        .items_center()
        .gap(px(4.0))
        .child(
            text(move || leg.get().side)
                .font_weight(FontWeight::BOLD)
                .app_class(move || {
                    let color = if leg.get().side == "BUY" {
                        "text-emerald-400"
                    } else {
                        "text-rose-400"
                    };
                    Some(SharedString::from(color))
                }),
        )
        .child(text(" × ").text_muted())
        .child(text(move || round_display(&leg.get().lots, 2)).font_family("monospace"));

    let strike = text(move || round_display(&leg.get().strike, 2))
        .font_family("monospace")
        .text_high();

    let expiry = text(move || leg.get().expiry).text_muted();

    let remove = HfCloseButton::element(HfCloseButtonProps {
        aria_label: Prop::from(Some("Remove leg".to_string())),
        ..Default::default()
    })
    .on_event(move |event: &HfButtonEvent| {
        if matches!(event, HfButtonEvent::Press) {
            cx.emit(HfStrategyLegRowEvent::Remove);
        }
    });

    view()
        .flex_row()
        .justify_between()
        .items_center()
        .px(px(12.0))
        .py(px(6.0))
        .rounded(px(4.0))
        .border_soft()
        .surface_base()
        .text_size(px(12.0))
        .app_class(Some(SharedString::from("hf-strategy-leg-row")))
        .child(side)
        .child(strike)
        .child(expiry)
        .child(remove)
}
