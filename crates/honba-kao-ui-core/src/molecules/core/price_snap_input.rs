use crate::primitive::{HfButton, HfOrderInputBox};
use leptos::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[component]
pub fn HfPriceSnapInput(
    #[prop(into)] value: RwSignal<String>,
    #[prop(into)] currency_symbol: String,
    #[prop(into)] best_bid: Signal<Option<Decimal>>,
    #[prop(into)] best_ask: Signal<Option<Decimal>>,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    let on_mid_click = Callback::new(move |_| {
        if let (Some(bid), Some(ask)) = (best_bid.get(), best_ask.get()) {
            let mid = (bid + ask) / dec!(2);
            value.set(format!("{}", mid.round_dp(2)));
        }
    });

    let is_btn_disabled =
        Signal::derive(move || disabled || best_bid.get().is_none() || best_ask.get().is_none());

    view! {
        <div class="hf-price-snap-input flex items-center gap-1.5 w-full">
            <HfOrderInputBox
                value=value
                adornment=currency_symbol
                placeholder="0.00"
                disabled=disabled
            />
            <HfButton
                class="hf-snap-btn px-2 text-xs"
                variant=crate::primitive::ButtonVariant::Outline
                size=crate::primitive::ButtonSize::Sm
                disabled=is_btn_disabled
                on_click=on_mid_click
            >
                "Mid"
            </HfButton>
        </div>
    }
}
