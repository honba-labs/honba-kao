//! `HfTabs` — wrapping [`nsh_base::Tabs`] container.
//!
//! `nsh_base::Tabs` has only a `children: Prop<Children>` field: it lays
//! out a tab list and gives it the `tablist` role for screen readers, but
//! holds no selected value and no selection concept at all. Selection
//! already lives correctly in this crate's `primitive/tab.rs`'s `HfTab`
//! (each tab knows whether it is the selected one and reports its own
//! press) — composing code owns which tab is selected the same way it
//! already owns `HfTab::selected`, by rendering the right `HfTab`/
//! `HfSubTab` children here.

use nsh_base::{Tabs, TabsProps};
use nsh_core::prelude::*;

/// Props for [`HfTabs`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfTabsProps {
    pub children: Prop<Children>,
}

/// A tabs container (`nsh_base::Tabs` shim): lays out `HfTab`/`HfSubTab`
/// children as one tab list for screen readers.
#[component]
pub fn HfTabs(props: HfTabsProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;

    Tabs::element(TabsProps {
        children: props.children,
    })
}
