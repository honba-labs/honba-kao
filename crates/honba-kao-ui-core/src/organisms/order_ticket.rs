//! Order ticket organism — NSH port per nsh-port.md §4 Phase 3.
//!
//! An order entry form inside a `Sheet`, with input fields. Built from
//! `nsh_base::Sheet`; honba styling via `.class()`/`.style()`.

use nsh_base::Sheet as BaseSheet;
use nsh_core::element::{Children, NoEvent, Scope};
use nsh_core::prelude::*;

/// Props for [`HfOrderTicket`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfOrderTicketProps {
    pub open: Prop<Option<bool>>,
    #[prop(default)]
    pub default_open: Prop<bool>,
    pub symbol: Prop<String>,
    pub side: Prop<String>, // "BUY" | "SELL"
    pub quantity: Prop<String>,
    #[prop(lazy)]
    pub children: Prop<Children>,
}

/// Order entry ticket presented in a bottom sheet.
#[component]
pub fn HfOrderTicket(props: HfOrderTicketProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    BaseSheet::element(nsh_base::SheetProps {
        open: props.open,
        default_open: props.default_open,
        side: Prop::from(Placement::Bottom),
        close_on_escape: Prop::from(true),
        close_on_backdrop_press: Prop::from(true),
        close_on_swipe: Prop::from(false),
        popup_class: Prop::from(Option::<SharedString>::None),
        popup_style: Default::default(),
        backdrop_class: Prop::from(Option::<SharedString>::None),
        backdrop_style: Default::default(),
        children: props.children,
    })
}
