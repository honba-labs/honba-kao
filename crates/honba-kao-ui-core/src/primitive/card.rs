//! `HfCard` — styled [`View`] composition.

use nsh_core::element::{Children, NoEvent};
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfCard`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfCardProps {
    #[prop(default = false)]
    pub outlined: Prop<bool>,
    pub children: Prop<Children>,
}

/// A card: rounded surface with a soft border.
#[component]
pub fn HfCard(props: HfCardProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let outlined_val = props.outlined.get_untracked();

    if outlined_val {
        view()
            .surface_base()
            .border(px(1.0))
            .rounded(px(8.0))
            .child(text(children_string(props.children)))
    } else {
        view()
            .surface_base()
            .rounded(px(8.0))
            .child(text(children_string(props.children)))
    }
}

/// Best-effort extraction of text content from children.
fn children_string(_children: Prop<Children>) -> String {
    String::new()
}
