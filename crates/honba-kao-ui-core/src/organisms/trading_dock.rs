//! Trading dock organism — NSH port per nsh-port.md §4 Phase 3.
//!
//! A dockable workspace shell. Uses `View` composition for now; `nsh_base`
//! `DockArea` integration follows once its controlled-layout API is stable.

use nsh_core::prelude::*;

/// Props for [`HfTradingDock`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfTradingDockProps {
    #[prop(lazy)]
    pub children: Prop<Children>,
}

/// A trading dock shell: panes laid out as dockable workspace.
/// The caller supplies panes as children and look via `.class()`/`.style()`.
#[component]
pub fn HfTradingDock(props: HfTradingDockProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    view()
        .flex_col()
        .gap(px(4.0))
        .px_4()
        .py_2()
        .bg(rgb(0x0f172a))
        .text_color(rgb(0xe2e8f0))
        .child(props.children)
}
