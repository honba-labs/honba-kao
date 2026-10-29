//! `HfLink` — styled, pressable [`View`] + [`Text`] acting as a link.
//!
//! NSH has no dedicated link primitive in `nsh_base`, so a link is a
//! pressable styled view. It reports presses as [`HfLinkEvent::Press`]
//! instead of an `on_click: Fn()` callback prop: a `#[derive(Props)]`
//! struct must be `Clone + PartialEq` (`nsh_core::element::Props: Clone +
//! PartialEq`), which a boxed closure can never satisfy — the same reason
//! `HfButton` has its own event (see `button.rs`). Composing code handles
//! navigation itself: `.on_event(move |event: &HfLinkEvent| ..)`.

use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// The event of [`HfLink`].
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfLinkEvent {
    /// The link was pressed.
    Press,
}

/// Props for [`HfLink`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfLinkProps {
    /// Honored by callers: this component never navigates on its own
    /// (there is no `<a>` primitive to point at a URL), so a caller
    /// wanting real navigation handles [`HfLinkEvent::Press`] and routes
    /// from there.
    pub href: Prop<String>,
    #[prop(default)]
    pub aria_label: Prop<Option<String>>,
    pub children: Prop<Children>,
}

/// A pressable text link.
#[component]
pub fn HfLink(props: HfLinkProps, cx: Scope<HfLinkEvent>) -> impl IntoElement {
    let _ = props.href;
    let label = props.aria_label.get_untracked().map(SharedString::from);

    view()
        .surface_base()
        .text_positive()
        .rounded(px(4.0))
        .px(px(2.0))
        .py(px(1.0))
        .optional_label(label)
        .child(props.children)
        .on_press(move |_: &()| cx.emit(HfLinkEvent::Press))
}
