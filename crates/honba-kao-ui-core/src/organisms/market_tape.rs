//! Market tape organism — NSH port per nsh-port.md §4 Phase 3.
//!
//! A scrolling ticker of market prices.

use nsh_core::prelude::*;

/// One tape entry: symbol + price + change direction.
#[derive(Debug, Clone, PartialEq, Props, JsObject)]
pub struct TapeEntry {
    pub symbol: String,
    pub price: String,
    pub change: String,
    #[prop(default)]
    pub up: bool,
}

/// Props for [`HfMarketTape`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfMarketTapeProps {
    pub entries: Prop<Vec<TapeEntry>>,
}

/// Scrolling market tape with live prices.
#[component]
pub fn HfMarketTape(props: HfMarketTapeProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let entries = props.entries;

    view()
        .flex_row()
        .gap(px(12.0))
        .px_4()
        .py_2()
        .bg(rgb(0x0f172a))
        .text_color(rgb(0xe2e8f0))
        .child(
            each(
                entries,
                move |e: &TapeEntry| e.symbol.clone(),
                move |entry_signal, _| {
                    let entry = entry_signal.get();
                    let sign = if entry.up { "+" } else { "-" };
                    view()
                        .flex_row()
                        .gap(px(6.0))
                        .px_3()
                        .py_1()
                        .child(text(entry.symbol))
                        .child(text(format!(" {}", entry.price)))
                        .child(text(format!(" {sign}")))
                        .child(text(entry.change))
                },
            )
            .into_element(),
        )
}
