//! `HfCollateralSimulator` — before / after impact simulation for collateral
//! deposits and withdrawals. Ported from `secured-finance-app` molecule
//! `CollateralSimulationSection`.
//!
//! The Leptos original owned `proposed_delta` as an `RwSignal<f64>` that its
//! two buttons mutated directly. nsh components don't own mutable state
//! that way — a component only has `Prop<T>` (caller-owned) and emits
//! `Event`s, the caller decides how state changes — same division of
//! responsibility as `HfQuantityChipInput`'s chips (`quantity_chip_input.rs`)
//! and `HfFilterPill`'s `Toggle` (`filter_pill.rs`). The two buttons here
//! just report which was pressed; the caller owns the ±10,000 step size,
//! updates `proposed_delta` itself, and re-renders with the new value.
//!
//! `current_collateral`/`proposed_delta` are money (Zero Floats Policy):
//! they cross as caller-pre-formatted `Prop<String>` and are parsed to
//! `Decimal` here only for the `simulated`/`impact` display calculations —
//! no `f64` touches them.

use std::str::FromStr;

use nsh_core::prelude::*;
use rust_decimal::Decimal;

use crate::primitive::base::button::{
    ButtonSize, ButtonVariant, HfButton, HfButtonEvent, HfButtonProps,
};
use crate::theme::tokens::HonbaStyle;

/// The events of [`HfCollateralSimulator`]: which button was pressed. The
/// caller owns the ±10,000 step size and `proposed_delta`'s value; the
/// component never mutates its own state.
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfCollateralSimulatorEvent {
    /// "Deposit ₹10k" was pressed.
    Deposit,
    /// "Withdraw ₹10k" was pressed.
    Withdraw,
}

/// Props for [`HfCollateralSimulator`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfCollateralSimulatorProps {
    /// Pre-formatted by the caller; parsed to `Decimal` for display.
    pub current_collateral: Prop<String>,
    /// Positive = deposit, negative = withdraw. Pre-formatted by the
    /// caller; parsed to `Decimal` for display.
    pub proposed_delta: Prop<String>,
}

/// Before / after impact simulation for collateral deposits / withdrawals:
/// two step buttons, the resulting delta, the simulated collateral value,
/// and the signed impact.
#[component]
pub fn HfCollateralSimulator(
    props: HfCollateralSimulatorProps,
    cx: Scope<HfCollateralSimulatorEvent>,
) -> impl IntoElement {
    let current = Decimal::from_str(&props.current_collateral.get_untracked()).unwrap_or_default();
    let delta = Decimal::from_str(&props.proposed_delta.get_untracked()).unwrap_or_default();
    let simulated = current + delta;
    let impact = delta;

    // `nsh_base::Button`'s variants (`ButtonVariant`, `primitive/base/
    // button.rs`) have no green/red-tinted variant matching the original's
    // emerald/rose buttons — layered via `HfButtonProps::class` (the "extra
    // Tailwind classes, applied after the button's own" slot that file
    // documents), same mechanism `funding_rate.rs`'s `classes` combination
    // uses, rather than inventing a `ButtonVariant` that doesn't exist.
    let deposit_button = HfButton::element(HfButtonProps {
        variant: Prop::from(Some(ButtonVariant::Ghost)),
        size: Prop::from(Some(ButtonSize::Sm)),
        class: Prop::from(Some(
            "bg-emerald-600/10 text-emerald-400 hover:bg-emerald-600/20 border \
             border-emerald-500/30"
                .to_string(),
        )),
        ..Default::default()
    })
    .child(text("Deposit ₹10k"))
    .on_event(move |event: &HfButtonEvent| {
        if matches!(event, HfButtonEvent::Press) {
            cx.emit(HfCollateralSimulatorEvent::Deposit);
        }
    });

    let withdraw_button = HfButton::element(HfButtonProps {
        variant: Prop::from(Some(ButtonVariant::Ghost)),
        size: Prop::from(Some(ButtonSize::Sm)),
        class: Prop::from(Some(
            "bg-rose-600/10 text-rose-400 hover:bg-rose-600/20 border border-rose-500/30"
                .to_string(),
        )),
        ..Default::default()
    })
    .child(text("Withdraw ₹10k"))
    .on_event(move |event: &HfButtonEvent| {
        if matches!(event, HfButtonEvent::Press) {
            cx.emit(HfCollateralSimulatorEvent::Withdraw);
        }
    });

    let impact_text = if impact >= Decimal::ZERO {
        text(format!("{impact:+.2}")).text_positive()
    } else {
        text(format!("{impact:+.2}")).text_negative()
    };

    view()
        .flex_col()
        .gap(px(8.0))
        .rounded(px(8.0))
        .border_soft()
        .surface_base()
        .p(px(12.0))
        .text_size(px(12.0))
        .app_class(Some(SharedString::from("hf-collateral-simulator")))
        .child(
            text("Collateral Simulator")
                .text_muted()
                .font_weight(FontWeight::SEMIBOLD)
                .uppercase()
                .letter_spacing(px(0.5))
                .text_size(px(10.0)),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(8.0))
                .app_class(Some(SharedString::from("hf-sim-controls")))
                .child(deposit_button)
                .child(withdraw_button)
                .child(
                    view().app_class(Some(SharedString::from("ml-auto"))).child(
                        text(format!("Δ: ₹{delta:+.2}"))
                            .text_muted()
                            .font_family("monospace"),
                    ),
                ),
        )
        .child(
            view()
                .flex_row()
                .justify_between()
                .items_center()
                .p(px(8.0))
                .rounded(px(4.0))
                .surface_muted()
                .border_soft()
                .app_class(Some(SharedString::from("hf-sim-result")))
                .child(text("Simulated Collateral").text_muted())
                .child(
                    text(format!("₹{simulated:.2}"))
                        .text_high()
                        .font_family("monospace"),
                )
                .child(impact_text.font_family("monospace")),
        )
}
