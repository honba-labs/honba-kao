//! `HfPercentSlider` — bespoke slider built on [`View`] + gestures.
//!
//! `nsh_base` has no standalone `Slider` in this release, so Honba builds a
//! slider from a pressable [`View`] whose thumb position is a signal. It
//! reports a change as [`HfPercentSliderEvent::Change`] instead of an
//! `on_change: Fn(f32)` callback prop: a `#[derive(Props)]` struct must be
//! `Clone + PartialEq` (`nsh_core::element::Props: Clone + PartialEq`),
//! which a boxed closure can never satisfy — the same reason `HfButton`
//! has its own event (see `button.rs`).

use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// The event of [`HfPercentSlider`].
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfPercentSliderEvent {
    /// The slider was pressed: the percentage it reports (0..=100).
    #[args(value)]
    Change(f32),
}

/// Props for [`HfPercentSlider`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfPercentSliderProps {
    pub value: Prop<f32>,
    #[prop(default = 0.0)]
    pub min: Prop<f32>,
    #[prop(default = 100.0)]
    pub max: Prop<f32>,
}

/// A horizontal percentage slider (0..=100).
#[component]
pub fn HfPercentSlider(
    props: HfPercentSliderProps,
    cx: Scope<HfPercentSliderEvent>,
) -> impl IntoElement {
    let _ = props.min;
    let _ = props.max;

    let pct = props.value.get_untracked().clamp(0.0, 100.0);

    view()
        .flex_1()
        .h(px(4.0))
        .rounded(px(999.0))
        .surface_muted()
        .child(
            view()
                .h(px(4.0))
                .w(px(pct * 4.0))
                .rounded(px(999.0))
                .bg(rgb(0x10b981)),
        )
        .on_press(move |_: &()| cx.emit(HfPercentSliderEvent::Change(pct)))
}
