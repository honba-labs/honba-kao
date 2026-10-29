//! `HfTab`, `HfSubTab` — tab primitives (bespoke View/Text wraps).

use nsh_core::element::{Children, NoEvent};
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfTab`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfTabProps {
    pub label: Prop<String>,
    #[prop(default = false)]
    pub selected: Prop<bool>,
    pub children: Prop<Children>,
}

/// A single tab.
#[component]
pub fn HfTab(props: HfTabProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let lbl = props.label.get_untracked();
    let sel = props.selected.get_untracked();

    if sel {
        view()
            .px(px(2.0))
            .py(px(1.0))
            .border(px(2.0))
            .border_color(rgb(0x10b981))
            .child(text(lbl).text_positive().font_weight(FontWeight::SEMIBOLD))
    } else {
        view()
            .px(px(2.0))
            .py(px(1.0))
            .child(text(lbl).text_muted().font_weight(FontWeight::MEDIUM))
    }
}

/// Props for [`HfSubTab`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfSubTabProps {
    pub label: Prop<String>,
    #[prop(default = false)]
    pub selected: Prop<bool>,
    pub children: Prop<Children>,
}

/// A sub-tab within a tab group.
#[component]
pub fn HfSubTab(props: HfSubTabProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let lbl = props.label.get_untracked();
    let sel = props.selected.get_untracked();

    if sel {
        view()
            .px(px(1.5))
            .py(px(0.5))
            .text_color(rgb(0x10b981))
            .font_weight(FontWeight::SEMIBOLD)
            .child(text(lbl))
    } else {
        view()
            .px(px(1.5))
            .py(px(0.5))
            .text_color(rgb(0x94a3b8))
            .font_weight(FontWeight::MEDIUM)
            .child(text(lbl))
    }
}
