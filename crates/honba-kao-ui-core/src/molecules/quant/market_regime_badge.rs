//! `HfMarketRegimeBadge` — market regime indicator (bull / bear / range).

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;

/// Market regime. `Range` is the default (the original had no default).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PropEnum)]
pub enum Regime {
    Bull,
    Bear,
    #[default]
    Range,
}

/// Props for [`HfMarketRegimeBadge`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfMarketRegimeBadgeProps {
    #[prop(default)]
    pub regime: Prop<Regime>,
}

/// Pure display: a small bordered pill with the regime's label and colour.
#[component]
pub fn HfMarketRegimeBadge(
    props: HfMarketRegimeBadgeProps,
    cx: Scope<NoEvent>,
) -> impl IntoElement {
    let _ = cx;
    let (label, color) = match props.regime.get_untracked() {
        Regime::Bull => (
            "Bullish",
            "text-emerald-400 bg-emerald-600/20 border-emerald-500/30",
        ),
        Regime::Bear => ("Bearish", "text-rose-400 bg-rose-600/20 border-rose-500/30"),
        Regime::Range => (
            "Range-bound",
            "text-cyan-400 bg-cyan-600/20 border-cyan-500/30",
        ),
    };

    let base = "hf-market-regime inline-flex items-center gap-1 rounded border px-2 py-0.5 text-[10px] font-bold";

    view()
        .flex_row()
        .items_center()
        .gap(px(4.0))
        .app_class(Some(SharedString::from(format!("{base} {color}"))))
        .child(text(label))
}
