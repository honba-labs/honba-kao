//! `HfDialog`, `HfModal`, `HfDrawer` — wrapping [`nsh_base::Sheet`].
//!
//! `nsh_base::AlertDialog` is a native, *imperative* alert/action-sheet:
//! `AlertDialogOptions` only has `title` / `description` / `actions`
//! (label + `AlertDialogRole::{Default,Cancel,Destructive}`), shown by
//! `AlertDialogOptions::show()` which resolves with the chosen action's id
//! — there is no `::element()`, no `Prop<Children>` slot and no
//! `Scope<Event>` to hang a wrapper off of. It cannot host the arbitrary
//! body content (forms, order summaries, confirmations) `HfDialog` /
//! `HfModal` / `HfDrawer` need, so it is the wrong primitive for this file
//! (the previous version of this file guessed fields like `role`,
//! `cancel_label` and `children` onto `AlertDialogOptions` that do not
//! exist on the real struct).
//!
//! `nsh_base::Sheet` is the real element that fits: a panel pinned to an
//! edge of the screen over a backdrop, with a `Prop<Children>` slot,
//! controlled `open`, and its own `SheetEvent`
//! (`OpenChange(bool, OpenChangeReason)` / `Open` / `Close` / `Cancel`) —
//! mirrored locally as [`HfSheetEvent`] and used as every wrapper's own
//! event, the same way `HfSwitch` mirrors `SwitchEvent` as its own
//! `HfSwitchEvent` (see `switch.rs`): `#[derive(Event)]`'s own hidden
//! macro of the enum's name (which `#[component]` expands) means
//! `nsh_base::SheetEvent` can never be reused directly as a `Scope<T>`
//! declared in this crate. Composing code hears open/close the same way
//! it would on a raw `Sheet`: `.on_event(move |event: &HfSheetEvent|
//! ..)`. `Sheet` only has edge placements
//! (`Placement::{Top,Bottom,Left,Right}`, no "center"), so `HfDialog` and
//! `HfModal` present from the top of the screen and `HfDrawer` from the
//! left or right (its `right` prop) — same as `nsh_base::Switch`'s
//! track/thumb, `Sheet`'s panel has no default styling (`popup_style` /
//! `backdrop_style` start empty), styled here through
//! [`HonbaStyle`](crate::theme::tokens::HonbaStyle).
//!
//! `open` is always app-owned (`Prop<bool>`, not `Sheet`'s own
//! controlled-or-uncontrolled `Option<bool>`), the same convention
//! `HfSwitch` uses for `checked`: the header close button (`HfDialog` /
//! `HfDrawer`) and `HfModal`'s Cancel/Confirm buttons are *outside*
//! `Sheet`'s own internal close machinery (escape / backdrop / swipe), so
//! they can only close a `Sheet` the app controls. `HfModal` has no
//! `on_confirm` / `on_cancel` callback props either (`Props` must be
//! `Clone + PartialEq`, see `switch.rs` — a boxed closure never is):
//! both buttons report through the same `HfSheetEvent::OpenChange` every
//! other close does, tagged by `OpenChangeReason` — `Cancel` for the
//! Cancel button and the header close ("✕") button (the same reason
//! Escape and a backdrop press already report: a dismissal), and
//! `TriggerPress` for Confirm (the one `OpenChangeReason` that means an
//! explicit action rather than a passive dismissal). Composing code tells
//! them apart by matching the reason.

use nsh_base::{OpenChangeReason, Sheet, SheetEvent, SheetProps};
use nsh_core::element::{Style, StyleRefinement};
use nsh_core::prelude::*;

use crate::primitive::base::button::{
    ButtonVariant, HfButton, HfButtonEvent, HfButtonProps, HfCloseButton, HfCloseButtonProps,
};
use crate::theme::tokens::HonbaStyle;

/// The events of `HfDialog`, `HfModal` and `HfDrawer`, translated 1:1
/// from [`nsh_base::SheetEvent`]: `#[derive(Event)]` makes a hidden macro
/// of the enum's name which the component's `#[component]` expands, so
/// the event enum and the component using it as `Scope<T>` must be of
/// one crate — `nsh_base::SheetEvent` can never be reused directly here
/// (the same reason `HfButton`/`HfSwitch` have their own event, see
/// `button.rs` / `switch.rs`).
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfSheetEvent {
    /// It closed (a press on the backdrop, Escape, back or a swipe),
    /// with why: set `open` to follow it.
    #[args(open, reason)]
    OpenChange(bool, OpenChangeReason),
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

/// A translucent backdrop behind the panel.
fn backdrop_style() -> StyleRefinement {
    StyleRefinement::default().bg(AdaptiveColor::rgb(0x000000, 0x000000).opacity(0.6))
}

/// The panel's own surface and corner radius.
fn panel_style() -> StyleRefinement {
    StyleRefinement::default().surface_base().rounded(px(12.0))
}

/// The header row shared by `HfDialog` and `HfDrawer`: a title and an
/// optional trailing element (the close button). `HfModal` builds its own
/// (title only, no close button — see its doc comment).
fn header_row(title: Option<String>, trailing: Option<AnyElement>) -> impl IntoElement {
    view()
        .flex_row()
        .items_center()
        .justify_between()
        .surface_elevated()
        .px(px(4.0))
        .py(px(3.0))
        .child(text(title.unwrap_or_default()).text_high())
        .child(trailing)
}

/// The body wrapper shared by all three: the arbitrary content the
/// composing side passed as this component's children.
fn body_wrap(children: Prop<Children>) -> impl IntoElement {
    view().flex_1().overflow_hidden().p(px(4.0)).child(children)
}

/// Props for [`HfDialog`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfDialogProps {
    /// Open. The app owns the value (same as `HfSwitch`'s `checked`):
    /// react to the emitted [`HfSheetEvent::OpenChange`] by setting it
    /// `false`.
    pub open: Prop<bool>,
    /// Above the body, with the close button.
    #[prop(default)]
    pub title: Prop<Option<String>>,
    /// Extra classes forwarded to the panel (`Sheet`'s `popup_class`),
    /// same escape hatch the original Leptos `HfDialog` had.
    #[prop(default)]
    pub class: Prop<Option<String>>,
    pub children: Prop<Children>,
}

/// Generic modal dialog: a title, a close button, and arbitrary body
/// content, presented over a backdrop from the top of the screen.
#[component]
pub fn HfDialog(props: HfDialogProps, cx: Scope<HfSheetEvent>) -> impl IntoElement {
    let is_open = props.open;
    let title = props.title.get_untracked();
    let class = props.class;

    let close = HfCloseButton::element(HfCloseButtonProps {
        aria_label: Prop::from(Some("Close".to_string())),
    })
    .on_event(move |event: &HfButtonEvent| {
        if matches!(event, HfButtonEvent::Press) {
            cx.emit(HfSheetEvent::Cancel);
            cx.emit(HfSheetEvent::OpenChange(false, OpenChangeReason::Cancel));
        }
    });

    Sheet::element(SheetProps {
        open: Prop::bind(move || Some(is_open.get())),
        side: Prop::from(Placement::Top),
        close_on_escape: Prop::from(true),
        close_on_backdrop_press: Prop::from(true),
        popup_class: Prop::bind(move || class.get().map(SharedString::from)),
        popup_style: Prop::from(Style::from(panel_style())),
        backdrop_style: Prop::from(Style::from(backdrop_style())),
        ..Default::default()
    })
    .child(header_row(title, Some(close.into_element())))
    .child(body_wrap(props.children))
    .on_event(move |event: &SheetEvent| cx.emit(HfSheetEvent::from(event.clone())))
}

/// Props for [`HfModal`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfModalProps {
    /// Open. The app owns the value, same as [`HfDialogProps::open`].
    pub open: Prop<bool>,
    /// Above the body. No close button (unlike `HfDialog`): dismiss via
    /// Cancel, Escape or the backdrop.
    #[prop(default)]
    pub title: Prop<Option<String>>,
    /// The Confirm button's label (default `"Confirm"`).
    #[prop(default)]
    pub confirm_label: Prop<Option<String>>,
    /// The Cancel button's label (default `"Cancel"`).
    #[prop(default)]
    pub cancel_label: Prop<Option<String>>,
    pub children: Prop<Children>,
}

/// Confirmation prompt: a title, arbitrary body content, and a
/// Cancel / Confirm footer, presented over a backdrop from the top of the
/// screen.
#[component]
pub fn HfModal(props: HfModalProps, cx: Scope<HfSheetEvent>) -> impl IntoElement {
    let is_open = props.open;
    let title = props.title.get_untracked();
    let confirm_label = props
        .confirm_label
        .get_untracked()
        .unwrap_or_else(|| "Confirm".to_string());
    let cancel_label = props
        .cancel_label
        .get_untracked()
        .unwrap_or_else(|| "Cancel".to_string());

    let cancel_button = HfButton::element(HfButtonProps {
        variant: Prop::from(Some(ButtonVariant::Ghost)),
        ..Default::default()
    })
    .child(text(cancel_label))
    .on_event(move |event: &HfButtonEvent| {
        if matches!(event, HfButtonEvent::Press) {
            cx.emit(HfSheetEvent::Cancel);
            cx.emit(HfSheetEvent::OpenChange(false, OpenChangeReason::Cancel));
        }
    });

    let confirm_button = HfButton::element(HfButtonProps {
        variant: Prop::from(Some(ButtonVariant::Primary)),
        ..Default::default()
    })
    .child(text(confirm_label))
    .on_event(move |event: &HfButtonEvent| {
        if matches!(event, HfButtonEvent::Press) {
            cx.emit(HfSheetEvent::OpenChange(
                false,
                OpenChangeReason::TriggerPress,
            ));
        }
    });

    let footer = view()
        .flex_row()
        .justify_end()
        .surface_elevated()
        .px(px(4.0))
        .py(px(3.0))
        .child(cancel_button)
        .child(confirm_button);

    Sheet::element(SheetProps {
        open: Prop::bind(move || Some(is_open.get())),
        side: Prop::from(Placement::Top),
        close_on_escape: Prop::from(true),
        close_on_backdrop_press: Prop::from(true),
        popup_style: Prop::from(Style::from(panel_style())),
        backdrop_style: Prop::from(Style::from(backdrop_style())),
        ..Default::default()
    })
    .child(header_row(title, None))
    .child(body_wrap(props.children))
    .child(footer)
    .on_event(move |event: &SheetEvent| cx.emit(HfSheetEvent::from(event.clone())))
}

/// Props for [`HfDrawer`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfDrawerProps {
    /// Open. The app owns the value, same as [`HfDialogProps::open`].
    pub open: Prop<bool>,
    /// Above the body, with the close button.
    #[prop(default)]
    pub title: Prop<Option<String>>,
    /// Slides in from the right edge instead of the left (default
    /// `false`, same default the original Leptos `HfDrawer` had).
    #[prop(default = false)]
    pub right: Prop<bool>,
    pub children: Prop<Children>,
}

/// Slide-over drawer panel (used by the Agent Console and wallets): a
/// title, a close button, and arbitrary body content, sliding in from the
/// left or right edge of the screen.
#[component]
pub fn HfDrawer(props: HfDrawerProps, cx: Scope<HfSheetEvent>) -> impl IntoElement {
    let is_open = props.open;
    let title = props.title.get_untracked();
    let side = if props.right.get_untracked() {
        Placement::Right
    } else {
        Placement::Left
    };

    let close = HfCloseButton::element(HfCloseButtonProps {
        aria_label: Prop::from(Some("Close".to_string())),
    })
    .on_event(move |event: &HfButtonEvent| {
        if matches!(event, HfButtonEvent::Press) {
            cx.emit(HfSheetEvent::Cancel);
            cx.emit(HfSheetEvent::OpenChange(false, OpenChangeReason::Cancel));
        }
    });

    Sheet::element(SheetProps {
        open: Prop::bind(move || Some(is_open.get())),
        side: Prop::from(side),
        close_on_escape: Prop::from(true),
        close_on_backdrop_press: Prop::from(true),
        popup_style: Prop::from(Style::from(panel_style().w(px(440.0)).h_full())),
        backdrop_style: Prop::from(Style::from(backdrop_style())),
        ..Default::default()
    })
    .child(header_row(title, Some(close.into_element())))
    .child(body_wrap(props.children))
    .on_event(move |event: &SheetEvent| cx.emit(HfSheetEvent::from(event.clone())))
}
