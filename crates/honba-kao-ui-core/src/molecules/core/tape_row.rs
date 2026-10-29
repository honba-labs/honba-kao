//! `HfTapeRow` — one trade on the time-and-sales tape: price, size and time.

use std::str::FromStr;

use nsh_core::prelude::*;
use rust_decimal::Decimal;

use crate::theme::tokens::HonbaStyle;

/// The events of [`HfTapeRow`].
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfTapeRowEvent {
    /// The row was pressed.
    Click,
}

/// Props for [`HfTapeRow`]. `price` and `size` are pre-formatted decimals.
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfTapeRowProps {
    pub price: Prop<String>,
    pub size: Prop<String>,
    pub time: Prop<String>,
    pub is_buyer_maker: Prop<bool>,
}

/// Parse a pre-formatted decimal and round it to `dp` places for display.
fn round_display(raw: &str, dp: u32) -> String {
    Decimal::from_str(raw)
        .unwrap_or_default()
        .round_dp(dp)
        .to_string()
}

/// A tape row, pressable as a whole. The price takes the trade's side colour.
#[component]
pub fn HfTapeRow(props: HfTapeRowProps, cx: Scope<HfTapeRowEvent>) -> impl IntoElement {
    let price = props.price;
    let size = props.size;
    let time = props.time;
    let is_buyer_maker = props.is_buyer_maker.get_untracked();

    let price_text = text(move || round_display(&price.get(), 2)).text_size(px(12.0));
    let price_text = if is_buyer_maker {
        price_text.text_negative()
    } else {
        price_text.text_positive()
    };

    view()
        .flex_row()
        .items_center()
        .justify_between()
        .px(px(8.0))
        .py(px(2.0))
        .text_size(px(12.0))
        .app_class(Some(SharedString::from(
            "hf-tape-row hover:bg-surface-elevated font-mono cursor-pointer",
        )))
        .child(price_text)
        .child(
            text(move || round_display(&size.get(), 4))
                .text_muted()
                .text_size(px(12.0)),
        )
        .child(text(move || time.get()).text_muted().text_size(px(10.0)))
        .on_press(move |_: &()| cx.emit(HfTapeRowEvent::Click))
}
