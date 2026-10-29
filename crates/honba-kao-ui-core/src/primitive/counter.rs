//! `HfCounter`, `HfAuctionTimer` — [`Text`] + signal/effect counters.

use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfCounter`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfCounterProps {
    /// The number this counter displays. Reactive, as the original Leptos
    /// `Signal<usize>` was: the displayed text re-renders as it changes.
    pub value: Prop<usize>,
    #[prop(default)]
    pub label: Prop<Option<String>>,
    /// Extra Tailwind classes, applied after the counter's own
    /// (`Element::app_class`).
    #[prop(default)]
    pub class: Prop<Option<String>>,
}

/// A numeric counter that animates between values.
#[component]
pub fn HfCounter(props: HfCounterProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let value = props.value;
    let class = props.class;
    let lbl = props.label.get_untracked().unwrap_or_default();

    view()
        .flex()
        .flex_col()
        .items_start()
        .app_class(move || class.get().map(SharedString::from))
        .child(
            text(move || value.get().to_string())
                .text_high()
                .font_weight(FontWeight::BOLD),
        )
        .child(if lbl.is_empty() {
            text("")
        } else {
            text(lbl).text_muted().text_size(px(10.0))
        })
}

/// Props for [`HfAuctionTimer`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfAuctionTimerProps {
    /// Seconds remaining in the auction.
    pub seconds: Prop<u64>,
    #[prop(default)]
    pub label: Prop<Option<String>>,
}

/// An auction / countdown timer.
#[component]
pub fn HfAuctionTimer(props: HfAuctionTimerProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let secs = props.seconds.get_untracked();
    let lbl = props.label.get_untracked().unwrap_or_default();

    let (m, s) = (secs / 60, secs % 60);
    let time_str = format!("{m:02}:{s:02}");

    view()
        .flex()
        .flex_col()
        .items_start()
        .child(text(time_str).text_high().font_weight(FontWeight::BOLD))
        .child(if lbl.is_empty() {
            text("")
        } else {
            text(lbl).text_muted().text_size(px(10.0))
        })
}
