//! Mosaic workspace template — NSH port per nsh-port.md §4 Phase 4.
//!
//! Multi-tile flexible workspace layout using `View` composition.

use nsh_core::prelude::*;

/// Props for [`HfMosaicWorkspace`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfMosaicWorkspaceProps {
    #[prop(default = false)]
    pub show_toolbar: Prop<bool>,
    pub children: Prop<Children>,
}

/// Multi-tile flexible workspace layout.
#[component]
pub fn HfMosaicWorkspace(props: HfMosaicWorkspaceProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    view().flex_col().gap(px(2.0)).w_full().h_full().child(
        view()
            .flex_row()
            .w_full()
            .child(view().flex_col().gap(px(2.0)).w_full().child(text(move || {
                if props.show_toolbar.get() {
                    "Toolbar".to_string()
                } else {
                    String::new()
                }
            })))
            .child(
                view()
                    .flex_1()
                    .grid_cols(12)
                    .gap(px(2.0))
                    .child(props.children),
            ),
    )
}
