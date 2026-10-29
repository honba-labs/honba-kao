//! `HfOrderDisplayBox`, `OrderDisplayItem` — display box primitive.

use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// One item in an order display box.
#[derive(Clone, Debug, Default, PartialEq, JsObject)]
pub struct OrderDisplayItem {
    pub label: String,
    pub value: String,
    pub tone: Option<String>,
}

/// Props for [`HfOrderDisplayBox`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfOrderDisplayBoxProps {
    pub items: Prop<Vec<OrderDisplayItem>>,
}

/// A compact key/value display row.
#[component]
pub fn HfOrderDisplayBox(props: HfOrderDisplayBoxProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let items = props.items.get_untracked();

    view()
        .flex()
        .flex_col()
        .gap(px(2.0))
        .px(px(2.0))
        .py(px(1.0))
        .surface_base()
        .rounded(px(6.0))
        .children(items.into_iter().map(|item| {
            let value = text(item.value).text_size(px(10.0));
            let value = match item.tone.as_deref() {
                Some("positive") => value.text_positive(),
                Some("negative") => value.text_negative(),
                Some("warning") => value.text_warning(),
                _ => value.text_high(),
            };

            view()
                .flex()
                .justify_between()
                .child(text(item.label).text_muted().text_size(px(10.0)))
                .child(value)
        }))
}
