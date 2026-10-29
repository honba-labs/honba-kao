//! `HfProgress`, `HfSpinner` — wrapping [`nsh_base::Progress`].
//!
//! `nsh_base::Progress` owns the progress-bar accessibility role, range
//! and value, and leaves the look (track + indicator) to the app. Neither
//! component here emits events (purely declarative, like the original
//! Leptos `HfProgress`), so both stay on `Scope<NoEvent>`.

use nsh_base::{Progress, ProgressProps};
use nsh_core::Color;
use nsh_core::element::{Style, StyleRefinement};
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfProgress`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfProgressProps {
    #[prop(default = 0.0)]
    pub value: Prop<f32>,
    #[prop(default = false)]
    pub indeterminate: Prop<bool>,
    #[prop(default)]
    pub color: Prop<Option<Color>>,
    pub children: Prop<Children>,
}

/// Determinate or indeterminate progress bar.
#[component]
pub fn HfProgress(props: HfProgressProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let indicator_color = props
        .color
        .get_untracked()
        .map(AdaptiveColor::from)
        .unwrap_or(AdaptiveColor::rgb(0x10b981, 0x10b981));
    let track_style = StyleRefinement::default()
        .h(px(6.0))
        .rounded_full()
        .surface_elevated();
    let indicator_style = StyleRefinement::default()
        .rounded_full()
        .bg(indicator_color);

    Progress::element(ProgressProps {
        value: props.value,
        indeterminate: props.indeterminate,
        track_style: Prop::from(Style::from(track_style)),
        indicator_style: Prop::from(Style::from(indicator_style)),
        children: props.children,
        ..Default::default()
    })
    .child(props.children)
}

/// Props for [`HfSpinner`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfSpinnerProps {
    #[prop(default)]
    pub color: Prop<Option<Color>>,
    pub children: Prop<Children>,
}

/// Indeterminate spinner (a progress bar with no value), muted by default
/// (ported from `secured-finance-app`'s `text-text-muted` spinner).
#[component]
pub fn HfSpinner(props: HfSpinnerProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let indicator_color = props
        .color
        .get_untracked()
        .map(AdaptiveColor::from)
        .unwrap_or(AdaptiveColor::rgb(0x94a3b8, 0x94a3b8));
    let indicator_style = StyleRefinement::default()
        .rounded_full()
        .bg(indicator_color);

    Progress::element(ProgressProps {
        value: Prop::from(0.0),
        indeterminate: Prop::from(true),
        indicator_style: Prop::from(Style::from(indicator_style)),
        children: props.children,
        ..Default::default()
    })
    .child(props.children)
}
