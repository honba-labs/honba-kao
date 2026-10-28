use crate::molecules::core::{HfPriceSnapInput, HfQuantityChipInput};
use crate::primitive::{ButtonSize, ButtonVariant, HfButton};
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct OrderIntent {
    pub symbol: String,
    pub side: String,       // "BUY" or "SELL"
    pub order_type: String, // "LIMIT" or "MARKET"
    pub price: f64,
    pub quantity: f64,
}

#[component]
pub fn HfOrderTicket(
    #[prop(into)] symbol: Signal<String>,
    #[prop(into)] mark_price: Signal<f64>,
    #[prop(into)] best_bid: Signal<Option<f64>>,
    #[prop(into)] best_ask: Signal<Option<f64>>,
    #[prop(into)] available_capital: Signal<f64>,
    #[prop(into)] margin_requirement_ratio: Signal<f64>,
    #[prop(into)] on_submit_order: Callback<OrderIntent>,
) -> impl IntoView {
    let side = RwSignal::new("BUY".to_string());
    let price_str = RwSignal::new("".to_string());
    let qty_str = RwSignal::new("".to_string());

    let est_notional = Signal::derive(move || {
        let p = price_str
            .get()
            .parse::<f64>()
            .unwrap_or_else(|_| mark_price.get());
        let q = qty_str.get().parse::<f64>().unwrap_or(0.0);
        p * q
    });

    let est_margin = Signal::derive(move || est_notional.get() * margin_requirement_ratio.get());
    let est_fee = Signal::derive(move || est_notional.get() * 0.0001);

    view! {
        <div class="hf-order-ticket flex flex-col gap-3 p-3 bg-surface-base border border-border-soft rounded">
            // Side Toggle Tabs
            <div class="hf-side-tabs grid grid-cols-2 gap-1 bg-surface-subtle p-0.5 rounded">
                <HfButton
                    class=format!("py-1.5 text-xs font-bold rounded transition-colors {}", if side.get() == "BUY" { "bg-emerald-600 text-white" } else { "text-text-muted" })
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Sm
                    on_click=Callback::new(move |_| side.set("BUY".to_string()))
                >
                    "BUY"
                </HfButton>
                <HfButton
                    class=format!("py-1.5 text-xs font-bold rounded transition-colors {}", if side.get() == "SELL" { "bg-rose-600 text-white" } else { "text-text-muted" })
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Sm
                    on_click=Callback::new(move |_| side.set("SELL".to_string()))
                >
                    "SELL"
                </HfButton>
            </div>

            // Price Snap Input
            <div class="flex flex-col gap-1">
                <label class="text-[11px] text-text-muted">"Limit Price"</label>
                <HfPriceSnapInput
                    value=price_str
                    currency_symbol="₹".to_string()
                    best_bid=best_bid
                    best_ask=best_ask
                />
            </div>

            // Quantity Chip Input
            <div class="flex flex-col gap-1">
                <label class="text-[11px] text-text-muted">"Quantity"</label>
                <HfQuantityChipInput
                    value=qty_str
                    max_quantity=Signal::derive(move || {
                        let p = price_str.get().parse::<f64>().unwrap_or_else(|_| mark_price.get());
                        if p > 0.0 { available_capital.get() / p } else { 0.0 }
                    })
                />
            </div>

            // Live Impact Estimation
            <div class="hf-ticket-impact flex flex-col gap-1 p-2 rounded bg-surface-subtle text-xs border border-border-soft">
                <div class="flex justify-between text-text-muted">
                    <span>"Order Value"</span>
                    <span class="font-mono text-text-high">{move || format!("₹{:.2}", est_notional.get())}</span>
                </div>
                <div class="flex justify-between text-text-muted">
                    <span>"Est. Fee (1 bp)"</span>
                    <span class="font-mono text-text-high">{move || format!("₹{:.2}", est_fee.get())}</span>
                </div>
                <div class="flex justify-between text-text-muted">
                    <span>"Margin Required"</span>
                    <span class="font-mono text-text-high">{move || format!("₹{:.2}", est_margin.get())}</span>
                </div>
            </div>

            // Order Action Button
            <HfButton
                class=format!("w-full py-2 font-bold text-white transition-opacity {}", if side.get() == "BUY" { "bg-emerald-600 hover:bg-emerald-500" } else { "bg-rose-600 hover:bg-rose-500" })
                variant=ButtonVariant::Primary
                size=ButtonSize::Md
                on_click=Callback::new(move |_| {
                    let p = price_str.get().parse::<f64>().unwrap_or(0.0);
                    let q = qty_str.get().parse::<f64>().unwrap_or(0.0);
                    if p > 0.0 && q > 0.0 {
                        on_submit_order.run(OrderIntent {
                            symbol: symbol.get(),
                            side: side.get(),
                            order_type: "LIMIT".to_string(),
                            price: p,
                            quantity: q,
                        });
                    }
                })
            >
                {move || format!("PLACE {} ORDER", side.get())}
            </HfButton>
        </div>
    }
}
