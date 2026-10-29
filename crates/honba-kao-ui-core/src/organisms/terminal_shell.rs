//! Terminal shell organism — NSH port per nsh-port.md §4 Phase 3.
//!
//! A persistent workspace shell. Uses `View` composition for now; `nsh_base`
//! `DockArea` integration follows once its controlled-layout API is stable.

use nsh_core::element::{Children, NoEvent, Scope};
use nsh_core::prelude::*;

/// Props for [`HfTerminalShell`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfTerminalShellProps {
    pub open: Prop<Option<bool>>,
    #[prop(default)]
    pub default_open: Prop<bool>,
    #[prop(lazy)]
    pub children: Prop<Children>,
}

/// A terminal shell: a persistent workspace shell.
#[component]
pub fn HfTerminalShell(props: HfTerminalShellProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    view()
        .flex_col()
        .gap(px(4.0))
        .px_4()
        .py_2()
        .bg(rgb(0x0f172a))
        .text_color(rgb(0xe2e8f0))
        .child(props.children)
}
