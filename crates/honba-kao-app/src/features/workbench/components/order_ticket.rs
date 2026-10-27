//! Right-hand order ticket, ported from the labs `order-ticket.tsx`: side,
//! product, order type, quantity (+ steppers), price, trigger, a margin
//! summary and the submit action.

use leptos::prelude::*;

use super::market_depth::MarketDepth;
use super::number_input::WbNumberInput;
use crate::features::workbench::stores::book_store::TradingState;
use crate::features::workbench::stores::order_draft_store::{OrderSide, OrderType, Product, TicketField};
use crate::features::workbench::stores::workbench_store::WorkbenchStore;
use crate::shared::utils::currency;

/// Order types as they appear in the ticket, in pill order.
const ORDER_TYPES: [OrderType; 4] = [
    OrderType::Market,
    OrderType::Limit,
    OrderType::StopMarket,
    OrderType::StopLimit,
];

/// Quantity quick-increments.
const QTY_STEPS: [i64; 4] = [10, 25, 50, 100];

#[component]
pub fn OrderTicketPanel() -> impl IntoView {
    let store = WorkbenchStore::take();
    let show_depth = RwSignal::new(false);

    view! {
        <aside class="wb-ticket">
            // -- Header: symbol at LTP, depth + close ------------------------
            <header class="wb-ticket__head">
                <div class="wb-ticket__instrument">
                    {move || store.draft.ticket.get().symbol.clone()}
                    <span class="wb-ticket__exchange">"NSE"</span>
                    <span class="wb-ticket__ltp tnum">
                        {move || format!("₹{:.2}", store.draft.ticket.get().price)}
                    </span>
                </div>
                <div class="wb-ticket__head-actions">
                    <button
                        class=move || {
                            if show_depth.get() {
                                "wb-chip wb-chip--engaged"
                            } else {
                                "wb-chip"
                            }
                        }
                        title="Market Depth"
                        on:click=move |_| show_depth.update(|open| *open = !*open)
                    >
                        "⇅ Depth"
                    </button>
                    <button
                        class="wb-chip wb-chip--ghost"
                        title="Close Order Ticket"
                        on:click=move |_| store.draft.close_order_ticket()
                    >
                        "✕"
                    </button>
                </div>
            </header>

            <Show when=move || show_depth.get()>
                <TicketDepth />
            </Show>

            // -- Buy / Sell -------------------------------------------------
            <div class="wb-toggle-grid">
                {[
                    OrderSide::Buy,
                    OrderSide::Sell,
                ]
                .into_iter()
                .map(|side| {
                    view! {
                        <button
                            class=move || {
                                if store.draft.ticket.get().side == side {
                                    "wb-toggle-grid__item wb-toggle-grid__item--on"
                                } else {
                                    "wb-toggle-grid__item"
                                }
                            }
                            on:click=move |_| {
                                store.draft.set_ticket_field(TicketField::Side(side))
                            }
                        >
                            {side.label()}
                        </button>
                    }
                })
                .collect_view()
            }
            </div>

            // -- Product ----------------------------------------------------
            <div class="wb-field">
                <span class="wb-field__label">"Product"</span>
                <div class="wb-toggle-grid">
                    <ProductButton store=store product=Product::Intraday />
                    <ProductButton store=store product=Product::Delivery />
                </div>
            </div>

            // -- Order type -------------------------------------------------
            <div class="wb-field">
                <span class="wb-field__label">"Order Type"</span>
                <div class="wb-type-grid">
                    {ORDER_TYPES
                        .iter()
                        .map(|order_type| view! { <TypePill store=store order_type=*order_type /> })
                        .collect_view()}
                </div>
            </div>

            // -- Quantity ---------------------------------------------------
            <div class="wb-field">
                <div class="wb-field__row">
                    <span class="wb-field__label">"Quantity"</span>
                    <span class="wb-field__hint">"Lot: 1"</span>
                </div>
                <WbNumberInput
                    class="wb-input--mono"
                    placeholder=Signal::stored("1".to_string())
                    disabled=Signal::stored(false)
                    min=1.0
                    step="1"
                    value=Signal::derive(move || store.draft.ticket.get().quantity as f64)
                    on_set=Callback::new(move |quantity: f64| {
                        store.draft.set_ticket_field(TicketField::Quantity(quantity as i64));
                    })
                />
                <div class="wb-steppers">
                    {QTY_STEPS
                        .iter()
                        .map(|step| {
                            let step = *step;
                            view! {
                                <button
                                    class="wb-step"
                                    on:click=move |_| store.draft.shift_quantity(step)
                                >
                                    {format!("+{step}")}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
            </div>

            // -- Price / Trigger ------------------------------------------------
            <div class="wb-field">
                <span class="wb-field__label">"Price (₹)"</span>
                <WbNumberInput
                        class="wb-input--mono"
                        step="0.05"
                        placeholder=Signal::derive(move || {
                            if store.draft.ticket.get().order_type == OrderType::Market {
                                format!("Market (LTP ₹{:.2})", store.draft.ticket.get().price)
                            } else {
                                "Enter limit price".to_string()
                            }
                        })
                        disabled=Signal::derive(move || store.draft.ticket.get().order_type == OrderType::Market)
                        value=Signal::derive(move || store.draft.ticket.get().price)
                        on_set=Callback::new(move |price: f64| {
                            store.draft.set_ticket_field(TicketField::Price(price));
                        })
                    />
            </div>

            <Show when=move || {
                matches!(
                    store.draft.ticket.get().order_type,
                    OrderType::StopMarket | OrderType::StopLimit
                )
            }>
                <div class="wb-field">
                    <span class="wb-field__label">"Trigger Price (₹)"</span>
                    <WbNumberInput
                        class="wb-input--mono"
                        step="0.05"
                        placeholder=Signal::stored("Trigger (₹)".to_string())
                        disabled=Signal::stored(false)
                        value=Signal::derive(move || store.draft.ticket.get().trigger_price)
                        on_set=Callback::new(move |trigger_price: f64| {
                            store.draft.set_ticket_field(TicketField::TriggerPrice(trigger_price));
                        })
                    />
                </div>
            </Show>

            // -- Summary ------------------------------------------------------
            {move || {
                let ticket = store.draft.ticket.get();
                let order_value = (ticket.price * ticket.quantity as f64 * 100.0).round() / 100.0;
                let margin = (order_value * ticket.product.margin_factor() * 100.0).round() / 100.0;
                view! {
                    <div class="wb-summary">
                        <div class="wb-summary__row">
                            <span>"Order Value:"</span>
                            <span>{currency(order_value)}</span>
                        </div>
                        <div class="wb-summary__row wb-summary__row--strong">
                            <span>"Margin Req:"</span>
                            <span>{currency(margin)}</span>
                        </div>
                    </div>
                }
            }}

            // -- Submit --------------------------------------------------------
            {move || {
                let ticket = store.draft.ticket.get();
                let halted = store.book.trading_state.get() == TradingState::Halted;
                let side = if ticket.side.is_buy() { "buy" } else { "sell" };
                let label = if halted {
                    "Engine halted".to_string()
                } else {
                    format!("Place {} Order", ticket.side.label())
                };
                view! {
                    <button
                        class=format!("wb-submit wb-submit--{side}{}", if halted { " wb-submit--halted" } else { "" })
                        disabled=halted
                        on:click=move |_| store.place_order()
                    >
                        {label}
                    </button>
                }
            }}
        </aside>
    }
}

/// Depth for the ticket's current symbol (re-reads it on every open).
#[component]
fn TicketDepth() -> impl IntoView {
    let store = WorkbenchStore::take();
    view! {
        {move || {
            let symbol = store.draft.ticket.get().symbol;
            view! { <MarketDepth symbol=symbol /> }
        }}
    }
}

#[component]
fn ProductButton(store: WorkbenchStore, product: Product) -> impl IntoView {
    let selected = move || store.draft.ticket.get().product == product;
    view! {
        <button
            class=move || {
                if selected() { "wb-toggle wb-toggle--on" } else { "wb-toggle" }
            }
            on:click=move |_| store.draft.set_ticket_field(TicketField::Product(product))
        >
            {product.label()}
        </button>
    }
}

#[component]
fn TypePill(store: WorkbenchStore, order_type: OrderType) -> impl IntoView {
    let selected = move || store.draft.ticket.get().order_type == order_type;
    view! {
        <button
            class=move || {
                if selected() { "wb-pill wb-pill--on" } else { "wb-pill" }
            }
            on:click=move |_| store.draft.set_ticket_field(TicketField::OrderType(order_type))
        >
            {order_type.short()}
        </button>
    }
}
