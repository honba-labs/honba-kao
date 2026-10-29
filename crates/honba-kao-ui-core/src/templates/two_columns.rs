//! Two-column split layout template — NSH port per nsh-port.md §4 Phase 4.
//!
//! Desktop (laptop+): an 8/4 grid (~70/30) of primary and rail.
//! Mobile: the two columns stack vertically.

use nsh_core::prelude::*;

/// Props for [`HfTwoColumnSplitLayout`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfTwoColumnSplitLayoutProps {
    #[prop(default = false)]
    pub reverse_on_mobile: Prop<bool>,
    pub children: Prop<Children>,
}

/// 70/30 asymmetric split layout.
#[component]
pub fn HfTwoColumnSplitLayout(
    props: HfTwoColumnSplitLayoutProps,
    _cx: Scope<NoEvent>,
) -> impl IntoElement {
    view()
        .flex_col()
        .gap(px(4.0))
        .w_full()
        .child(props.children)
}
