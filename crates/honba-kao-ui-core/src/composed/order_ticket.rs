//! HfOrderTicket — symbol, side, type, qty, price, SL/TP, submit.
//! Core trading order entry component.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use crate::{HfButton, HfInput, HfSelect, HfSelectItem, HfButtonVariant, HfSideMarker, HfSide};

#[component]
pub fn HfOrderTicket(
    /// Default symbol.
    #[prop(optional, into)]
    symbol: Option<String>,
    /// Default side.
    #[prop(optional)]
    side: Option<HfSide>,
    /// Default order type.
    #[prop(optional, into)]
    order_type: Option<String>,
    /// Default quantity.
    #[prop(optional)]
    quantity: Option<i64>,
    /// Default price.
    #[prop(optional)]
    price: Option<f64>,
    /// Called when the order is submitted.
    #[prop(optional)]
    on_submit: Option<Callback<OrderTicketData>>,
    #[prop(optional, into)]
    class: String,
) -> impl IntoView {
    let side = side.unwrap_or(HfSide::Buy);
    let order_type = order_type.unwrap_or_else(|| "LIMIT".to_string());
    let quantity = quantity.unwrap_or(25);
    let price = price.unwrap_or(0.0);

    let symbol_value = leptos::prelude::RwSignal::new(symbol.unwrap_or_default());
    let qty_value = leptos::prelude::RwSignal::new(quantity.to_string());
    let price_value = leptos::prelude::RwSignal::new(price.to_string());

    let handle_submit = {
        let on_submit = on_submit;
        let symbol_value = symbol_value;
        let side = side;
        let order_type_for_cb = order_type.clone();
        let qty_value = qty_value;
        let price_value = price_value;
        Callback::new(move |_| {
            if let Some(cb) = on_submit {
                cb.run(OrderTicketData {
                    symbol: symbol_value.get(),
                    side,
                    order_type: order_type_for_cb.clone(),
                    quantity: qty_value.get().parse().unwrap_or(0),
                    price: price_value.get().parse().unwrap_or(0.0),
                });
            }
        })
    };

    let on_symbol_input = {
        let symbol_value = symbol_value;
        Callback::new(move |ev: web_sys::Event| {
            let target = ev.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
            symbol_value.set(target.value());
        })
    };

    let on_qty_input = {
        let qty_value = qty_value;
        Callback::new(move |ev: web_sys::Event| {
            let target = ev.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
            qty_value.set(target.value());
        })
    };

    let on_price_input = {
        let price_value = price_value;
        Callback::new(move |ev: web_sys::Event| {
            let target = ev.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
            price_value.set(target.value());
        })
    };

    let full_class = format!("hf-order-ticket {class}").trim().to_string();

    view! {
        <div class=full_class>
            <div class="hf-order-ticket__header">
                <HfInput
                    placeholder="Symbol"
                    value=symbol_value.get()
                    on_input=on_symbol_input
                />
                <HfSideMarker side=side />
            </div>

            <div class="hf-order-ticket__row">
                <HfSelect value=order_type.clone()>
                    <HfSelectItem value="MARKET">"MKT"</HfSelectItem>
                    <HfSelectItem value="LIMIT">"LMT"</HfSelectItem>
                    <HfSelectItem value="STOP_MARKET">"SL-M"</HfSelectItem>
                    <HfSelectItem value="STOP_LIMIT">"SL"</HfSelectItem>
                </HfSelect>
            </div>

            <div class="hf-order-ticket__row">
                <HfInput
                    r#type="number"
                    placeholder="Qty"
                    value=qty_value.get()
                    on_input=on_qty_input
                />
                <HfInput
                    r#type="number"
                    placeholder="Price"
                    value=price_value.get()
                    on_input=on_price_input
                />
            </div>

            <div class="hf-order-ticket__actions">
                <HfButton variant=HfButtonVariant::Green on_click=handle_submit>
                    "Place Order"
                </HfButton>
            </div>
        </div>
    }
}

/// Order ticket submission data.
#[derive(Clone, Debug, PartialEq)]
pub struct OrderTicketData {
    pub symbol: String,
    pub side: HfSide,
    pub order_type: String,
    pub quantity: i64,
    pub price: f64,
}