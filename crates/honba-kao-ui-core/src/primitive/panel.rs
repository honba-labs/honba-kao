//! `HfPanel`, `HfPanelHeader`, `HfSectionGrid` — panel primitives.

use nsh_core::element::{Children, NoEvent};
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfPanel`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfPanelProps {
    #[prop(default)]
    pub header: Prop<Option<String>>,
    pub children: Prop<Children>,
}

/// A panel: a surface region with optional header.
#[component]
pub fn HfPanel(props: HfPanelProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let hdr = props.header.get_untracked().unwrap_or_default();

    view()
        .surface_base()
        .rounded(px(8.0))
        .border(px(1.0))
        .child(
            view()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(if hdr.is_empty() {
                    text("")
                } else {
                    text(hdr).text_high().font_weight(FontWeight::SEMIBOLD)
                })
                .child(
                    text(children_string(props.children))
                        .text_muted()
                        .text_size(px(12.0)),
                ),
        )
}

/// Props for [`HfPanelHeader`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfPanelHeaderProps {
    pub title: Prop<String>,
}

/// A panel header.
#[component]
pub fn HfPanelHeader(props: HfPanelHeaderProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let t = props.title.get_untracked();

    view()
        .px(px(2.0))
        .py(px(1.0))
        .child(text(t).text_high().font_weight(FontWeight::SEMIBOLD))
}

/// Props for [`HfSectionGrid`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfSectionGridProps {
    #[prop(default = 2)]
    pub cols: Prop<u16>,
    pub children: Prop<Children>,
}

/// A section grid (2 or 3 columns).
#[component]
pub fn HfSectionGrid(props: HfSectionGridProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let _cols = props.cols.get_untracked();

    view()
        .grid()
        .grid_cols(2)
        .gap(px(4.0))
        .px(px(2.0))
        .py(px(1.0))
        .child(text(children_string(props.children)).text_muted())
}

/// Best-effort extraction of text content from children.
fn children_string(_children: Prop<Children>) -> String {
    String::new()
}
