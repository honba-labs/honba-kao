//! `HfFooterBar` — layout [`View`] footer.

use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// One labeled value shown in a [`HfFooterBar`].
#[derive(Clone, Debug, Default, PartialEq, JsObject)]
pub struct HfFooterItem {
    pub label: String,
    pub value: String,
}

/// Props for [`HfFooterBar`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfFooterBarProps {
    /// Shown left-to-right.
    pub items: Prop<Vec<HfFooterItem>>,
}

/// A terminal footer bar.
#[component]
pub fn HfFooterBar(props: HfFooterBarProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let items = props.items.get_untracked();

    view()
        .surface_base()
        .border_strong()
        .px(px(2.0))
        .py(px(1.0))
        .flex()
        .justify_between()
        .items_center()
        .children(items.into_iter().map(|item| {
            view()
                .flex()
                .flex_col()
                .gap(px(0.5))
                .child(text(item.label).text_muted().text_size(px(10.0)))
                .child(text(item.value).text_high().text_size(px(10.0)))
        }))
}
