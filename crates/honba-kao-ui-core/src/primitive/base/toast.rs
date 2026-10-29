//! `HfAlertBanner`, `HfAlertBannerStack`, `HfToastStack` — wrapping
//! [`nsh_base::Toast`] / [`nsh_base::ToastStack`].
//!
//! Neither has a title/message field of its own: `Toast` is an unstyled
//! semantic root (lifecycle/motion props, a `Prop<Children>` slot, and
//! `ToastEvent::DismissRequest` for a swipe that asked to dismiss it), so
//! `HfAlertBanner` puts its message through `.child(text(...))` instead
//! of a `title`/`message` field, and mirrors `DismissRequest` as
//! [`HfToastEvent`] — the same 1:1 translation `HfButton`/`HfSwitch` do
//! for their own `nsh_base` events (`#[derive(Event)]`'s hidden macro of
//! the enum's name means `nsh_base::ToastEvent`/`ToastStackEvent` can
//! never be reused directly as a `Scope<T>` declared in this crate).
//! `ToastStack` only ever lays out its toast children, mirrored the same
//! way as [`HfToastStackEvent`].

use nsh_base::{Toast, ToastEvent, ToastProps, ToastStack, ToastStackEvent, ToastStackProps};
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Alert severity tones.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PropEnum)]
pub enum AlertSeverity {
    #[default]
    Info,
    Success,
    Warning,
    Danger,
}

/// The event of [`HfAlertBanner`], translated 1:1 from
/// [`nsh_base::ToastEvent`].
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfToastEvent {
    /// A swipe asked to dismiss it.
    DismissRequest,
}

impl From<ToastEvent> for HfToastEvent {
    fn from(event: ToastEvent) -> Self {
        match event {
            ToastEvent::DismissRequest => HfToastEvent::DismissRequest,
        }
    }
}

/// The event of [`HfAlertBannerStack`] / [`HfToastStack`], translated 1:1
/// from [`nsh_base::ToastStackEvent`].
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfToastStackEvent {
    /// Interaction expanded or collapsed the stack: the pointer on it, or
    /// a finger down on it.
    #[args(expanded)]
    ExpandedChange(bool),
    /// Escape or the Android back button while it holds toasts: dismiss
    /// the newest.
    Cancel,
}

impl From<ToastStackEvent> for HfToastStackEvent {
    fn from(event: ToastStackEvent) -> Self {
        match event {
            ToastStackEvent::ExpandedChange(expanded) => {
                HfToastStackEvent::ExpandedChange(expanded)
            }
            ToastStackEvent::Cancel => HfToastStackEvent::Cancel,
        }
    }
}

/// Props for [`HfAlertBanner`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfAlertBannerProps {
    pub message: Prop<String>,
    #[prop(default)]
    pub severity: Prop<AlertSeverity>,
}

/// A single alert banner / toast: the message as its content, styled by
/// [`AlertSeverity`].
#[component]
pub fn HfAlertBanner(props: HfAlertBannerProps, cx: Scope<HfToastEvent>) -> impl IntoElement {
    let severity = props.severity.get_untracked();
    let message = props.message;

    // `HonbaStyle` has no per-severity border color (only `text_positive` /
    // `text_negative` / `text_warning`), so `Info` and the border both fall
    // back to the same adaptive color the text uses for its tone.
    let border = match severity {
        AlertSeverity::Info => AdaptiveColor::rgb(0x3b82f6, 0x60a5fa),
        AlertSeverity::Success => AdaptiveColor::rgb(0x10b981, 0x34d399),
        AlertSeverity::Warning => AdaptiveColor::rgb(0xf59e0b, 0xfbbf24),
        AlertSeverity::Danger => AdaptiveColor::rgb(0xef4444, 0xf87171),
    };

    Toast::element(ToastProps::default())
        .border(px(1.0))
        .border_color(border)
        .child(dynamic(move || {
            let label = text(message.get());
            match severity {
                AlertSeverity::Info => label.text_color(border),
                AlertSeverity::Success => label.text_positive(),
                AlertSeverity::Warning => label.text_warning(),
                AlertSeverity::Danger => label.text_negative(),
            }
        }))
        .on_event(move |event: &ToastEvent| cx.emit(HfToastEvent::from(event.clone())))
}

/// Props for [`HfAlertBannerStack`] / [`HfToastStack`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfToastStackProps {
    /// The toasts, oldest first.
    pub children: Prop<Children>,
}

/// A stack of alert banners / toasts.
#[component]
pub fn HfAlertBannerStack(
    props: HfToastStackProps,
    cx: Scope<HfToastStackEvent>,
) -> impl IntoElement {
    ToastStack::element(ToastStackProps {
        children: props.children,
        ..Default::default()
    })
    .on_event(move |event: &ToastStackEvent| cx.emit(HfToastStackEvent::from(event.clone())))
}

/// A live-updating toast stack (alias of [`HfAlertBannerStack`]).
#[component]
pub fn HfToastStack(props: HfToastStackProps, cx: Scope<HfToastStackEvent>) -> impl IntoElement {
    ToastStack::element(ToastStackProps {
        children: props.children,
        ..Default::default()
    })
    .on_event(move |event: &ToastStackEvent| cx.emit(HfToastStackEvent::from(event.clone())))
}
