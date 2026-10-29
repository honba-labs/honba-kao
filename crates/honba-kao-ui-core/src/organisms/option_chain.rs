//! Option chain organism — NSH port per nsh-port.md §4 Phase 3.
//!
//! A table-like view with bid/ask/IV/delta columns. Built from `nsh_core`
//! primitives.

use nsh_core::prelude::*;

/// One option chain row. Amounts as strings for JsValue compatibility.
#[derive(Debug, Clone, PartialEq, Props, JsObject)]
pub struct OptionChainRow {
    pub strike: String,
    pub bid: String,
    pub ask: String,
    pub iv: String,
    pub delta: String,
}

/// Props for [`HfOptionChain`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfOptionChainProps {
    pub underlying: Prop<String>,
    pub rows: Prop<Vec<OptionChainRow>>,
}

/// Option chain: strikes with bid/ask/IV/delta columns.
#[component]
pub fn HfOptionChain(props: HfOptionChainProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let rows = props.rows;

    view()
        .flex_col()
        .gap(px(4.0))
        .px_4()
        .py_2()
        .bg(rgb(0x0f172a))
        .text_color(rgb(0xe2e8f0))
        .child(view().child(text(format!(
            "Option Chain — {}",
            props.underlying.get_untracked()
        ))))
        .child(
            view()
                .flex_col()
                .gap(px(2.0))
                .child(
                    view()
                        .flex_row()
                        .justify_between()
                        .px_2()
                        .py_1()
                        .child(text("Strike"))
                        .child(text("Bid"))
                        .child(text("Ask"))
                        .child(text("IV"))
                        .child(text("Delta")),
                )
                .child(
                    each(
                        rows,
                        move |r: &OptionChainRow| r.strike.clone(),
                        move |row_signal, _| {
                            let r = row_signal.get();
                            view()
                                .flex_row()
                                .justify_between()
                                .px_2()
                                .py_1()
                                .child(text(r.strike))
                                .child(text(r.bid))
                                .child(text(r.ask))
                                .child(text(r.iv))
                                .child(text(r.delta))
                        },
                    )
                    .into_element(),
                ),
        )
}
