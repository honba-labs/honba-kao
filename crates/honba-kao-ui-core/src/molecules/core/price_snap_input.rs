//! `HfPriceSnapInput` — price input with instant Best-Bid / Mid / Best-Ask snapping.
//! Ported from `secured-finance-app` molecule patterns and the restructure spec
//! (Section 7.2).

use leptos::prelude::*;
use crate::primitive::{HfButton, HfOrderInputBox, ButtonVariant, ButtonSize};

#[component]
pub fn HfPriceSnapInput(
    #[prop(into)] value: RwSignal<String>,
    #[prop(into)] currency_symbol: String,
    #[prop(into)] best_bid: Signal<Option<f64>>,
    #[prop(into)] best_ask: Signal<Option<f64>>,
    #[prop(default = false)] disabled: bool,
) -> impl IntoView {
    let on_mid_click = move |_| {
        if let (Some(bid), Some(ask)) = (best_bid.get(), best_ask.get()) {
            let mid = (bid + ask) / 2.0;
            value.set(format!("{:.2}", mid));
        }
    };
    let mid_disabled = Signal::derive(move || {
        disabled || best_bid.get().is_none() || best_ask.get().is_none()
    });

    view! {
        <div class="hf-price-snap-input flex items-center gap-1.5">
            <HfOrderInputBox
                value=value
                adornment=currency_symbol
                placeholder="0.00"
                disabled=disabled
            />
            <HfButton
                variant=ButtonVariant::Outline
                size=ButtonSize::Sm
                disabled=mid_disabled
                on_click=on_mid_click
            >
                "Mid"
            </HfButton>
        </div>
    }
}
