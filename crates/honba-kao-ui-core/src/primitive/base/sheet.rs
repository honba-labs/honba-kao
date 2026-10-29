//! `HfSheet` — wrapping [`nsh_base::Sheet`].
//!
//! `nsh_base::Sheet` owns its presentation (open/close, escape, backdrop
//! press, swipe) and leaves the look to the app. `SheetEvent::OpenChange`
//! / `Open` / `Close` / `Cancel` is mirrored locally as [`HfSheetEvent`]
//! and used as `HfSheet`'s own event instead of an `on_close: Fn()`
//! callback prop, the same reason and pattern `HfDialog` (`dialog.rs`)
//! uses for its own `Sheet` wrap: `#[derive(Event)]`'s hidden macro of the
//! enum's name means `nsh_base::SheetEvent` can never be reused directly
//! as a `Scope<T>` declared in this crate. `open` is always app-owned
//! (`Prop<bool>`, not `Sheet`'s own controlled-or-uncontrolled
//! `Option<bool>`), the same convention `HfDialog`/`HfSwitch` use.

use nsh_base::{Sheet, SheetEvent, SheetProps};
use nsh_core::prelude::*;

/// The events of [`HfSheet`], translated 1:1 from [`nsh_base::SheetEvent`].
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfSheetEvent {
    /// It closed (a press on the backdrop, Escape, back or a swipe), with
    /// why: set `open` to follow it.
    #[args(open, reason)]
    OpenChange(bool, nsh_base::OpenChangeReason),
    /// It came in: it is all there.
    Open,
    /// It went: it is all gone (also when the app closed it).
    Close,
    /// The user tried to close it (Escape or back, a press on the
    /// backdrop, or an enabled swipe), whether or not `closeOnEscape` /
    /// `closeOnBackdropPress` let it close.
    Cancel,
}

impl From<SheetEvent> for HfSheetEvent {
    fn from(event: SheetEvent) -> Self {
        match event {
            SheetEvent::OpenChange(open, reason) => HfSheetEvent::OpenChange(open, reason),
            SheetEvent::Open => HfSheetEvent::Open,
            SheetEvent::Close => HfSheetEvent::Close,
            SheetEvent::Cancel => HfSheetEvent::Cancel,
        }
    }
}

/// Props for [`HfSheet`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfSheetProps {
    /// Open. The app owns the value (same as `HfDialog`'s `open`): react
    /// to the emitted [`HfSheetEvent::OpenChange`] by setting it `false`.
    #[prop(default = false)]
    pub open: Prop<bool>,
    pub children: Prop<Children>,
}

/// A bottom/side sheet overlay.
#[component]
pub fn HfSheet(props: HfSheetProps, cx: Scope<HfSheetEvent>) -> impl IntoElement {
    let is_open = props.open;

    Sheet::element(SheetProps {
        open: Prop::bind(move || Some(is_open.get())),
        ..Default::default()
    })
    .child(props.children)
    .on_event(move |event: &SheetEvent| cx.emit(HfSheetEvent::from(event.clone())))
}
