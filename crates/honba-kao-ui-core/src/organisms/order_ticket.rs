//! Order ticket organism — ported from secured-finance `AdvancedLendingOrderCard`.
//!
//! All price/qty/money values are `Decimal` (zero-floats invariant, issue #33).

use crate::molecules::core::{HfPriceSnapInput, HfQuantityChipInput};
use crate::primitive::{ButtonSize, ButtonVariant, HfButton};
use leptos::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Clone, Debug, PartialEq)]
pub struct OrderIntent {
    pub symbol: String,
    pub side: String,       // "BUY" or "SELL"
    pub order_type: String, // "LIMIT" or "MARKET"
    pub price: Decimal,
    pub quantity: Decimal,
}

#[component]
pub fn HfOrderTicket(
    #[prop(into)] symbol: Signal<String>,
    #[prop(into)] mark_price: Signal<Decimal>,
    #[prop(into)] best_bid: Signal<Option<Decimal>>,
    #[prop(into)] best_ask: Signal<Option<Decimal>>,
    #[prop(into)] available_capital: Signal<Decimal>,
    #[prop(into)] margin_requirement_ratio: Signal<Decimal>,
    #[prop(into)] on_submit_order: Callback<OrderIntent>,
) -> impl IntoView {
    let side = RwSignal::new("BUY".to_string());
    let order_type = RwSignal::new("LIMIT".to_string()); // "LIMIT" or "MARKET"
    let price_str = RwSignal::new("".to_string());
    let qty_str = RwSignal::new("".to_string());
    let confirmed = RwSignal::new(false); // two-step place: confirm, then submit

    let est_notional = Signal::derive(move || {
        let p: Decimal = price_str
            .get()
            .parse()
            .unwrap_or_else(|_| mark_price.get());
        let q: Decimal = qty_str.get().parse().unwrap_or(Decimal::ZERO);
        p * q
    });

    let est_margin = Signal::derive(move || est_notional.get() * margin_requirement_ratio.get());
    let est_fee = Signal::derive(move || est_notional.get() * dec!(0.0001));

    view! {
        <div class="hf-order-ticket flex flex-col gap-3 p-3 bg-surface-base border border-border-soft rounded">
            // Side Toggle Tabs
            <div class="hf-side-tabs grid grid-cols-2 gap-1 bg-surface-subtle p-0.5 rounded">
                <HfButton
                    class=Signal::derive(move || format!("py-1.5 text-xs font-bold rounded transition-colors {}", if side.get() == "BUY" { "bg-emerald-600 text-white" } else { "text-text-muted" }))
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Sm
                    on_click=Callback::new(move |_| side.set("BUY".to_string()))
                >
                    "BUY"
                </HfButton>
                <HfButton
                    class=Signal::derive(move || format!("py-1.5 text-xs font-bold rounded transition-colors {}", if side.get() == "SELL" { "bg-rose-600 text-white" } else { "text-text-muted" }))
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Sm
                    on_click=Callback::new(move |_| side.set("SELL".to_string()))
                >
                    "SELL"
                </HfButton>
            </div>

            // Order type toggle (LIMIT / MARKET)
            <div class="hf-side-tabs grid grid-cols-2 gap-1 bg-surface-subtle p-0.5 rounded mt-1">
                <HfButton
                    class=Signal::derive(move || format!("py-1.5 text-xs font-bold rounded transition-colors {}", if order_type.get() == "LIMIT" { "bg-surface-base text-text-high font-semibold shadow-sm" } else { "text-text-muted hover:text-text-high" }))
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Sm
                    on_click=Callback::new(move |_| order_type.set("LIMIT".to_string()))
                >
                    "Limit"
                </HfButton>
                <HfButton
                    class=Signal::derive(move || format!("py-1.5 text-xs font-bold rounded transition-colors {}", if order_type.get() == "MARKET" { "bg-surface-base text-text-high font-semibold shadow-sm" } else { "text-text-muted hover:text-text-high" }))
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Sm
                    on_click=Callback::new(move |_| order_type.set("MARKET".to_string()))
                >
                    "Market"
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
                        let p: Decimal = price_str.get().parse().unwrap_or(mark_price.get());
                        let m = margin_requirement_ratio.get();
                        if p > Decimal::ZERO && m > Decimal::ZERO {
                            // margin-aware: notional x margin ratio must fit in capital
                            available_capital.get() / (p * m)
                        } else {
                            Decimal::ZERO
                        }
                    })
                />
            </div>

            // Live Impact Estimation
            <div class="hf-ticket-impact flex flex-col gap-1 p-2 rounded bg-surface-subtle text-xs border border-border-soft">
                <div class="flex justify-between text-text-muted">
                    <span>"Order Value"</span>
                    <span class="font-mono text-text-high">{move || format!("₹{}", est_notional.get().round_dp(2))}</span>
                </div>
                <div class="flex justify-between text-text-muted">
                    <span>"Est. Fee (1 bp)"</span>
                    <span class="font-mono text-text-high">{move || format!("₹{}", est_fee.get().round_dp(2))}</span>
                </div>
                <div class="flex justify-between text-text-muted">
                    <span>"Margin Required"</span>
                    <span class="font-mono text-text-high">{move || format!("₹{}", est_margin.get().round_dp(2))}</span>
                </div>
            </div>

            // Order Action Button (two-step confirmation: confirm, then submit)
            <HfButton
                class=Signal::derive(move || format!("w-full py-2 font-bold text-white transition-opacity {}", if side.get() == "BUY" { "bg-emerald-600 hover:bg-emerald-500" } else { "bg-rose-600 hover:bg-rose-500" }))
                variant=ButtonVariant::Primary
                size=ButtonSize::Md
                on_click=Callback::new(move |_| {
                    if !confirmed.get() {
                        confirmed.set(true);
                    } else {
                        let p: Decimal = price_str.get().parse().unwrap_or(Decimal::ZERO);
                        let q: Decimal = qty_str.get().parse().unwrap_or(Decimal::ZERO);
                        if p > Decimal::ZERO && q > Decimal::ZERO {
                            on_submit_order.run(OrderIntent {
                                symbol: symbol.get(),
                                side: side.get(),
                                order_type: order_type.get(),
                                price: p,
                                quantity: q,
                            });
                            confirmed.set(false);
                        }
                    }
                })
            >
                {move || {
                    if confirmed.get() {
                        "CONFIRM ORDER".to_string()
                    } else {
                        format!("PLACE {} ORDER ({})", side.get(), order_type.get())
                    }
                }}
            </HfButton>
        </div>
    }
}
