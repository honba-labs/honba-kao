use crate::primitive::{HfButton, HfOrderInputBox};
use leptos::prelude::*;

/// Mid price = (best bid + best ask) / 2. Pure so it can be unit-tested.
pub(crate) fn mid_price(best_bid: f64, best_ask: f64) -> f64 {
    (best_bid + best_ask) / 2.0
}

#[component]
pub fn HfPriceSnapInput(
    #[prop(into)] value: RwSignal<String>,
    #[prop(into)] currency_symbol: String,
    #[prop(into)] best_bid: Signal<Option<f64>>,
    #[prop(into)] best_ask: Signal<Option<f64>>,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    let on_mid_click = Callback::new(move |_| {
        if let (Some(bid), Some(ask)) = (best_bid.get(), best_ask.get()) {
            value.set(format!("{:.2}", mid_price(bid, ask)));
        }
    });

    let is_btn_disabled =
        Signal::derive(move || disabled || best_bid.get().is_none() || best_ask.get().is_none());

    let spread = Signal::derive(move || match (best_bid.get(), best_ask.get()) {
        (Some(bid), Some(ask)) if ask > bid => {
            let abs = ask - bid;
            let bps = (abs / bid) * 10_000.0;
            format!("{:.2} / {:.2} bps", abs, bps)
        }
        _ => "-".to_string(),
    });

    view! {
        <div class="hf-price-snap-input flex flex-col gap-1 w-full">
            <div class="hf-book-tick flex justify-between text-[10px] text-text-muted mb-0.5">
                <span>{move || match best_bid.get() {
                    Some(b) => format!("Bid {:.2}", b),
                    None => "Bid –".to_string(),
                }}</span>
                <span>{move || spread.get()}</span>
                <span>{move || match best_ask.get() {
                    Some(a) => format!("Ask {:.2}", a),
                    None => "Ask –".to_string(),
                }}</span>
            </div>
            <div class="hf-price-snap-row flex items-center gap-1.5 w-full">
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
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mid_price_is_average_of_bid_and_ask() {
        let tol = 1e-6;
        assert!((mid_price(100.0, 102.0) - 101.0).abs() < tol);
        assert!((mid_price(0.0, 0.0) - 0.0).abs() < tol);
        assert!((mid_price(-5.0, 5.0) - 0.0).abs() < tol);
        assert!(mid_price(100.0, 110.0) > mid_price(100.0, 100.0));
    }
}
