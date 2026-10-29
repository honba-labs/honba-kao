//! Mutual fund dossier organism — NSH port per nsh-port.md §4 Phase 3.
//!
//! A drill-down fund detail view using `nav_stack`/`nav_page` from
//! `nsh_base` navigation. Built from `nsh_core` primitives.

use nsh_base::{nav_page, nav_stack};
use nsh_core::element::{Children, NoEvent, Scope};
use nsh_core::prelude::*;

/// Props for [`HfMutualFundDossier`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfMutualFundDossierProps {
    pub fund_name: Prop<String>,
    pub nav: Prop<String>,
    #[prop(lazy)]
    pub children: Prop<Children>,
}

/// A fund dossier in a navigation stack.
#[component]
pub fn HfMutualFundDossier(
    props: HfMutualFundDossierProps,
    _cx: Scope<NoEvent>,
) -> impl IntoElement {
    view().flex_col().child(
        nav_stack().child(
            nav_page()
                .child(view().child(text(format!("{}", props.fund_name.get_untracked()))))
                .child(view().child(text(format!("NAV: {}", props.nav.get_untracked()))))
                .child(props.children),
        ),
    )
}
