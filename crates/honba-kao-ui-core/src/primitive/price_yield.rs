//! `HfPriceYieldItem` — [`Text`] + formatting for price/yield pairs.

use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfPriceYieldItem`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfPriceYieldItemProps {
    pub label: Prop<String>,
    pub price: Prop<String>,
    #[prop(default)]
    pub yield_pct: Prop<Option<String>>,
}

/// A price/yield display item.
#[component]
pub fn HfPriceYieldItem(props: HfPriceYieldItemProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let lbl = props.label.get_untracked();
    let prc = props.price.get_untracked();
    let yld = props.yield_pct.get_untracked().unwrap_or_default();

    view()
        .flex()
        .justify_between()
        .px(px(2.0))
        .py(px(1.0))
        .child(text(lbl).text_high())
        .child(text(prc).text_muted().font_family("monospace"))
        .child(if yld.is_empty() {
            text("")
        } else {
            text(format!("({yld})")).text_positive().text_size(px(10.0))
        })
}
