//! `HfKbdBadge` — styled [`Text`] for key combos.

use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfKbdBadge`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfKbdBadgeProps {
    /// The key combo text (e.g. `M1`, `F2`).
    pub key: Prop<String>,
}

/// A keyboard key badge (e.g. `[M1]`, `[F2]`).
#[component]
pub fn HfKbdBadge(props: HfKbdBadgeProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let k = props.key.get_untracked();

    view()
        .px(px(1.5))
        .py(px(0.5))
        .rounded(px(4.0))
        .surface_elevated()
        .border_soft()
        .text_size(px(10.0))
        .font_family("monospace")
        .text_high()
        .child(text(k))
}
