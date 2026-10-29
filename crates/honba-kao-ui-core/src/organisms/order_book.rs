//! Order book organism — NSH port per nsh-port.md §4 Phase 3.
//!
//! L2 depth ladder with rows rendered via `each`. Prices/sizes are strings
//! (Decimal formatted by the caller) since `Decimal` is not `JsValue`-safe.

use nsh_core::prelude::*;

/// One book level. Amounts as strings for JsValue compatibility.
#[derive(Debug, Clone, PartialEq, Props, JsObject)]
pub struct BookLevel {
    pub price: String,
    pub size: String,
    pub order_count: String,
}

/// Spread info as pre-formatted strings.
#[derive(Debug, Clone, PartialEq, Props, JsObject)]
pub struct SpreadInfo {
    pub absolute: String,
    pub percentage: String,
}

/// Props for [`HfOrderBook`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfOrderBookProps {
    pub bids: Prop<Vec<BookLevel>>,
    pub asks: Prop<Vec<BookLevel>>,
    pub spread: Prop<Option<SpreadInfo>>,
}

/// Depth ladder: bids on top, asks below, with spread in the middle.
#[component]
pub fn HfOrderBook(props: HfOrderBookProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let bids = props.bids;
    let asks = props.asks;

    view()
        .flex_col()
        .gap(px(4.0))
        .px_4()
        .py_2()
        .bg(rgb(0x0f172a))
        .text_color(rgb(0xe2e8f0))
        .child(
            view()
                .flex_row()
                .justify_between()
                .child(text("BIDS"))
                .child(text("ASKS")),
        )
        .child(
            view()
                .flex_row()
                .gap(px(8.0))
                .child(
                    view().flex_1().child(
                        each(
                            bids,
                            move |b: &BookLevel| b.price.clone(),
                            move |b_signal, _| {
                                let b = b_signal.get();
                                view()
                                    .flex_row()
                                    .justify_between()
                                    .px_2()
                                    .py_1()
                                    .child(text(b.price))
                                    .child(text(b.size))
                            },
                        )
                        .into_element(),
                    ),
                )
                .child(
                    view().flex_1().child(
                        each(
                            asks,
                            move |a: &BookLevel| a.price.clone(),
                            move |a_signal, _| {
                                let a = a_signal.get();
                                view()
                                    .flex_row()
                                    .justify_between()
                                    .px_2()
                                    .py_1()
                                    .child(text(a.price))
                                    .child(text(a.size))
                            },
                        )
                        .into_element(),
                    ),
                ),
        )
        .child(
            view()
                .flex_row()
                .justify_between()
                .child(text("Spread"))
                .child(text(move || {
                    props.spread.get_untracked().map_or_else(
                        || "—".to_string(),
                        |s| format!("{} ({}%)", s.absolute, s.percentage),
                    )
                })),
        )
}
