//! `HfSwitch` — wrapping [`nsh_base::Switch`].
//!
//! `nsh_base::Switch` owns press/keyboard/disabled/checked behavior and
//! leaves the look to the app (track + thumb, styled through
//! `trackStyle` / `thumbStyle`). `SwitchEvent::Change(bool)` is mirrored
//! locally as [`HfSwitchEvent`] and used as `HfSwitch`'s own event
//! instead of an `on_change: Fn(bool)` prop: a `#[derive(Props)]` struct
//! must be `Clone + PartialEq` (`nsh_core::element::Props: Clone +
//! PartialEq`), which a boxed closure can never satisfy — and
//! `#[derive(Event)]`'s own hidden macro of the enum's name (which
//! `#[component]` expands) means `nsh_base::SwitchEvent` can never be
//! reused directly as a `Scope<T>` declared in this crate either, so
//! [`HfSwitchEvent`] translates it 1:1. Composing code hears a toggle the
//! same way it would on a raw `nsh_base::Switch`: `.on_change(move
//! |checked: &bool| ..)`.

use nsh_base::{Switch, SwitchEvent, SwitchProps};
use nsh_core::element::{Style, StyleRefinement};
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// The events of [`HfSwitch`], translated 1:1 from
/// [`nsh_base::SwitchEvent`]: `#[derive(Event)]` makes a hidden macro of
/// the enum's name which the component's `#[component]` expands, so the
/// event enum and the component using it as `Scope<T>` must be of one
/// crate — `nsh_base::SwitchEvent` can never be reused directly here.
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfSwitchEvent {
    /// The switch was pressed: the value it asks for, the opposite of
    /// `checked`.
    #[args(checked)]
    Change(bool),
}

/// Props for [`HfSwitch`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfSwitchProps {
    /// On or off. The app owns the value, same as `nsh_base::Switch`.
    pub checked: Prop<bool>,
    #[prop(default = false)]
    pub disabled: Prop<bool>,
}

/// Binary toggle switch (`bfx-hf-ui-core`'s `hf-switch`, ported: a pill
/// track with a sliding thumb, tinted emerald while checked).
#[component]
pub fn HfSwitch(props: HfSwitchProps, cx: Scope<HfSwitchEvent>) -> impl IntoElement {
    let track_style = StyleRefinement::default()
        .w(px(44.0))
        .h(px(24.0))
        .rounded_full()
        .border(px(1.0))
        .surface_base();
    let thumb_style = StyleRefinement::default()
        .w(px(20.0))
        .h(px(20.0))
        .rounded_full()
        .bg(AdaptiveColor::rgb(0xffffff, 0xffffff));
    let checked_style = StyleRefinement::default().bg(AdaptiveColor::rgb(0x10b981, 0x10b981));

    Switch::element(SwitchProps {
        checked: props.checked,
        disabled: props.disabled,
        checked_style: Prop::from(Style::from(checked_style)),
        track_style: Prop::from(Style::from(track_style)),
        thumb_style: Prop::from(Style::from(thumb_style)),
        ..Default::default()
    })
    .disabled_style(|s| s.opacity(0.5))
    .on_event(move |event: &SwitchEvent| match event {
        SwitchEvent::Change(checked) => cx.emit(HfSwitchEvent::Change(*checked)),
    })
}
