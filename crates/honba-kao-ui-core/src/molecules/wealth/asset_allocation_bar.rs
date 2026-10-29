//! `HfAssetAllocationBar` — segmented bar showing Equity, Debt, Cash, Gold weights.

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

#[derive(Clone, Debug, Default, PartialEq, JsObject)]
pub struct Allocation {
    pub equity: f64,
    pub debt: f64,
    pub cash: f64,
    pub gold: f64,
}

/// Props for [`HfAssetAllocationBar`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfAssetAllocationBarProps {
    pub alloc: Prop<Allocation>,
}

/// A bar split into Equity / Debt / Cash / Gold segments, each sized by its
/// share of the total. Zero total draws an empty bar.
#[component]
pub fn HfAssetAllocationBar(
    props: HfAssetAllocationBarProps,
    cx: Scope<NoEvent>,
) -> impl IntoElement {
    let _ = cx;
    let a = props.alloc.get_untracked();
    let total = a.equity + a.debt + a.cash + a.gold;

    // Share of the total as a percentage; `flex_grow` splits the bar by these.
    let share = |v: f64| -> f32 {
        if total > 0.0 {
            (v / total * 100.0).clamp(0.0, 100.0) as f32
        } else {
            0.0
        }
    };

    view()
        .flex()
        .w_full()
        .h(px(8.0))
        .rounded_full()
        .overflow_hidden()
        .surface_muted()
        .border_soft()
        .child(view().flex_grow(share(a.equity)).h_full().bg(rgb(0x10b981)))
        .child(view().flex_grow(share(a.debt)).h_full().bg(rgb(0x3b82f6)))
        .child(view().flex_grow(share(a.cash)).h_full().bg(rgb(0xf59e0b)))
        .child(view().flex_grow(share(a.gold)).h_full().bg(rgb(0xf43f5e)))
}
