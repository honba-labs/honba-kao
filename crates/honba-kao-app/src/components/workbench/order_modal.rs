//! Modal order ticket, ported from the labs `order-modal.tsx`: a dialog
//! version of the ticket — halted banner, depth accordion, side toggle,
//! product cards, order-type tabs, quantity stepper with quick chips,
//! price/trigger grid, and a validity + order-value footer.

use core::time::Duration;

use leptos::prelude::*;

use super::market_depth::MarketDepth;
use super::number_input::WbNumberInput;
use crate::stores::workbench::{
    OrderSide, OrderType, Product, TicketField, TimeInForce, TradingState, WorkbenchStore,
};
use crate::utils::currency;

/// Quantity quick chips, as in the labs modal.
const QUICK_INCREMENTS: [i64; 6] = [1, 10, 25, 50, 100, 250];

const ORDER_TYPE_TABS: [(OrderType, &'static str); 4] = [
    (OrderType::Market, "Market"),
    (OrderType::Limit, "Limit"),
    (OrderType::StopLimit, "SL"),
    (OrderType::StopMarket, "SL-M"),
];

#[component]
pub fn WorkbenchOrderModal() -> impl IntoView {
    let store = WorkbenchStore::take();

    let show_depth = RwSignal::new(false);
    let submitting = RwSignal::new(false);
    let success = RwSignal::new(false);

    let halted = move || store.trading_state.get() == TradingState::Halted;

    let close = move |_| store.close_order_modal();

    let submit = move |_| {
        if halted() || submitting.get_untracked() {
            return;
        }
        submitting.set(true);
        store.place_order();
        submitting.set(false);
        success.set(true);
        set_timeout(
            move || {
                success.set(false);
                store.close_order_modal();
            },
            Duration::from_millis(500),
        );
    };

    view! {
        <Show when=move || store.modal_open.get()>
            <div class="hfui-modal__backdrop" on:click=close>
                <div
                    class="hfui-modal__container wb-order-modal"
                    on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                >
                    // -- Header ------------------------------------------------=
                    <div class="hfui-modal__header">
                        <div class="wb-order-modal__title">
                            {move || {
                                let side = store.ticket.get().side;
                                let class = if side.is_buy() {
                                    "wb-order-modal__side wb-order-modal__side--buy"
                                } else {
                                    "wb-order-modal__side wb-order-modal__side--sell"
                                };
                                view! { <span class=class>{side.label()}</span> }
                            }}
                            {move || store.ticket.get().symbol.clone()}
                            <span class="wb-order-modal__venue">"NSE EQ"</span>
                        </div>
                        <button class="hfui-panel__icon-button" title="Close" on:click=close>
                            "✕"
                        </button>
                    </div>

                    <div class="hfui-modal__body">
                        // -- LTP + depth toggle line -----------------------------
                        <div class="wb-order-modal__ltp-row">
                            <span class="tnum">
                                "LTP: "
                                <strong>{move || format!("₹{:.2}", store.ticket.get().price)}</strong>
                            </span>
                            <button
                                class="wb-order-modal__depth-toggle"
                                on:click=move |_| show_depth.update(|open| *open = !*open)
                            >
                                "⇅ "
                                {move || {
                                    if show_depth.get() { "Hide Market Depth" } else { "Show Market Depth" }
                                }}
                            </button>
                        </div>

                        // -- Halted banner --------------------------------------
                        <Show when=halted>
                            <div class="wb-halted-banner">"⚠ Trading engine is currently HALTED. Orders cannot be submitted."</div>
                        </Show>

                        // -- Depth accordion --------------------------------------
                        <Show when=move || show_depth.get()>
                            <div class="wb-order-modal__depth">
                                {move || {
                                    let symbol = store.ticket.get().symbol;
                                    view! { <MarketDepth symbol=symbol /> }
                                }}
                            </div>
                        </Show>

                        // -- Side toggle ------------------------------------------
                        <div class="wb-toggle-grid wb-toggle-grid--modal">
                            <ModalSideButton store=store side=OrderSide::Buy />
                            <ModalSideButton store=store side=OrderSide::Sell />
                        </div>

                        // -- Product cards ------------------------------------------
                        <div class="wb-field">
                            <div class="wb-field__row">
                                <span class="wb-field__label wb-field__label--secondary">"Product Type"</span>
                                <span class="wb-field__hint">
                                    {move || {
                                        if store.ticket.get().product == Product::Intraday {
                                            "5x Leverage Intraday"
                                        } else {
                                            "Zero Leverage Delivery"
                                        }
                                    }}
                                </span>
                            </div>
                            <div class="wb-toggle-grid">
                                <ModalProductCard store=store product=Product::Intraday />
                                <ModalProductCard store=store product=Product::Delivery />
                            </div>
                        </div>

                        // -- Order type tabs -----------------------------------------
                        <div class="wb-field">
                            <span class="wb-field__label wb-field__label--secondary">"Order Type"</span>
                            <div class="wb-type-grid">
                                {ORDER_TYPE_TABS
                                    .iter()
                                    .map(|(order_type, label)| {
                                        let order_type = *order_type;
                                        let label = *label;
                                        let selected = move || store.ticket.get().order_type == order_type;
                                        view! {
                                            <button
                                                class=move || {
                                                    if selected() { "wb-pill wb-pill--on" } else { "wb-pill" }
                                                }
                                                on:click=move |_| {
                                                    store.set_ticket_field(TicketField::OrderType(order_type));
                                                }
                                            >
                                                {label}
                                            </button>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        </div>

                        // -- Quantity + quick chips -----------------------------------
                        <div class="wb-field">
                            <div class="wb-field__row">
                                <span class="wb-field__label wb-field__label--secondary">"Quantity (Shares)"</span>
                                <span class="wb-field__hint">"Lot size: 1"</span>
                            </div>

                            <div class="wb-stepper-row">
                                <button
                                    class="wb-step wb-step--square"
                                    on:click=move |_| store.shift_quantity(-1)
                                >
                                    "−"
                                </button>
                                <WbNumberInput
                                    class="wb-input--mono wb-input--center"
                                    placeholder=Signal::stored("1".to_string())
                                    disabled=Signal::stored(false)
                                    min=1.0
                                    step="1"
                                    value=Signal::derive(move || store.ticket.get().quantity as f64)
                                    on_set=Callback::new(move |quantity: f64| {
                                        store.set_ticket_field(TicketField::Quantity(quantity as i64));
                                    })
                                />
                                <button
                                    class="wb-step wb-step--square"
                                    on:click=move |_| store.shift_quantity(1)
                                >
                                    "+"
                                </button>
                            </div>

                            <div class="wb-chip-row">
                                {QUICK_INCREMENTS
                                    .iter()
                                    .map(|chip| {
                                        let chip = *chip;
                                        view! {
                                            <button
                                                class="wb-step wb-step--chip"
                                                on:click=move |_| store.shift_quantity(chip)
                                            >
                                                {format!("+{chip}")}
                                            </button>
                                        }
                                    })
                                    .collect_view()}
                                <button
                                    class="wb-step wb-step--chip wb-step--reset"
                                    on:click=move |_| {
                                        store.set_ticket_field(TicketField::Quantity(1));
                                    }
                                >
                                    "Reset (1)"
                                </button>
                            </div>
                        </div>

                        // -- Price + trigger grid ---------------------------------------
                        <div class="wb-two-col">
                            <div class="wb-field">
                                <span class="wb-field__label wb-field__label--secondary">"Price (₹)"</span>
                                <WbNumberInput
                                    class="wb-input--mono"
                                    step="0.05"
                                    placeholder=Signal::derive(move || {
                                        if store.ticket.get().order_type == OrderType::Market {
                                            "MKT".to_string()
                                        } else {
                                            "0.00".to_string()
                                        }
                                    })
                                    disabled=Signal::derive(move || {
                                        store.ticket.get().order_type == OrderType::Market
                                    })
                                    value=Signal::derive(move || store.ticket.get().price)
                                    on_set=Callback::new(move |price: f64| {
                                        store.set_ticket_field(TicketField::Price(price));
                                    })
                                />
                            </div>
                            <div class="wb-field">
                                <span class="wb-field__label wb-field__label--secondary">"Trigger Price (₹)"</span>
                                <WbNumberInput
                                    class="wb-input--mono"
                                    step="0.05"
                                    placeholder=Signal::derive(move || {
                                        if matches!(
                                            store.ticket.get().order_type,
                                            OrderType::StopMarket | OrderType::StopLimit
                                        ) {
                                            "Trigger".to_string()
                                        } else {
                                            "N/A".to_string()
                                        }
                                    })
                                    disabled=Signal::derive(move || {
                                        !matches!(
                                            store.ticket.get().order_type,
                                            OrderType::StopMarket | OrderType::StopLimit
                                        )
                                    })
                                    value=Signal::derive(move || store.ticket.get().trigger_price)
                                    on_set=Callback::new(move |trigger_price: f64| {
                                        store.set_ticket_field(TicketField::TriggerPrice(trigger_price));
                                    })
                                />
                            </div>
                        </div>

                        // -- Validity + order value ---------------------------------------
                        {move || {
                            let ticket = store.ticket.get();
                            let order_value = (ticket.price * ticket.quantity as f64 * 100.0).round() / 100.0;
                            view! {
                                <div class="wb-validity">
                                    <div class="wb-validity__tifs">
                                        <span class="wb-field__hint">"Validity:"</span>
                                        {[
                                            (TimeInForce::Day, "DAY"),
                                            (TimeInForce::Ioc, "IOC"),
                                        ]
                                            .map(|(tif, label)| {
                                                let selected = move || store.ticket.get().tif == tif;
                                                view! {
                                                    <button
                                                        class=move || {
                                                            if selected() {
                                                                "wb-pill wb-pill--on"
                                                            } else {
                                                                "wb-pill"
                                                            }
                                                        }
                                                        on:click=move |_| {
                                                            store.set_ticket_field(TicketField::TimeInForce(tif));
                                                        }
                                                    >
                                                        {label}
                                                    </button>
                                                }
                                            })
                                            .collect_view()}
                                    </div>
                                    <div class="wb-validity__value">
                                        "Value: "
                                        <strong class="tnum">{currency(order_value)}</strong>
                                    </div>
                                </div>
                            }
                        }}
                    </div>

                    // -- Footer ------------------------------------------------------
                    <div class="hfui-modal__footer">
                        {move || {
                            let ticket = store.ticket.get();
                            let margin = (ticket.price * ticket.quantity as f64
                                * ticket.product.margin_factor()
                                * 100.0)
                                .round()
                                / 100.0;
                            view! {
                                <div class="wb-order-modal__margin">
                                    <span class="wb-field__hint">"Margin Required"</span>
                                    <span class="wb-order-modal__margin-value tnum">{currency(margin)}</span>
                                </div>
                                <div class="wb-order-modal__actions">
                                    <button class="wb-btn wb-btn--chrome" on:click=close>
                                        "Cancel"
                                    </button>
                                    {move || {
                                        let ticket = store.ticket.get();
                                        let halted = store.trading_state.get() == TradingState::Halted;
                                        let side = if ticket.side.is_buy() { "buy" } else { "sell" };
                                        let label = if success.get() {
                                            "✓ Placed".to_string()
                                        } else if submitting.get() {
                                            "Placing…".to_string()
                                        } else {
                                            format!("{} {}", ticket.side.label(), ticket.symbol)
                                        };
                                        view! {
                                            <button
                                                class=format!("wb-submit wb-submit--{side}{}", if halted { " wb-submit--halted" } else { "" })
                                                disabled=halted
                                                on:click=submit
                                            >
                                                {label}
                                            </button>
                                        }
                                    }}
                                </div>
                            }
                        }}
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn ModalSideButton(store: WorkbenchStore, side: OrderSide) -> impl IntoView {
    let selected = move || store.ticket.get().side == side;
    view! {
        <button
            class=move || {
                let base = if side.is_buy() { "wb-side wb-side--buy" } else { "wb-side wb-side--sell" };
                if selected() { format!("{base} wb-side--on") } else { base.to_string() }
            }
            on:click=move |_| store.set_ticket_field(TicketField::Side(side))
        >
            "⚡ "
            {side.label()}
        </button>
    }
}

#[component]
fn ModalProductCard(store: WorkbenchStore, product: Product) -> impl IntoView {
    let selected = move || store.ticket.get().product == product;
    view! {
        <button
            class=move || {
                let base = "wb-toggle wb-toggle--card";
                if selected() { format!("{base} wb-toggle--on") } else { base.to_string() }
            }
            on:click=move |_| store.set_ticket_field(TicketField::Product(product))
        >
            {move || {
                if product == Product::Intraday {
                    "Intraday (MIS)"
                } else {
                    "Longterm (CNC)"
                }
            }}
        </button>
    }
}
