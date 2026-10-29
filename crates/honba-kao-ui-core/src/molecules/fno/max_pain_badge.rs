//! `HfMaxPainBadge` — max pain strike indicator.

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;

/// Props for [`HfMaxPainBadge`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfMaxPainBadgeProps {
    pub strike: Prop<f64>,
    /// Extra Tailwind classes, applied after the badge's own.
    #[prop(default)]
    pub class: Prop<Option<String>>,
}

/// Amber pill with a sentiment glyph and the max pain strike.
#[component]
pub fn HfMaxPainBadge(props: HfMaxPainBadgeProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let strike = props.strike;
    let extra = props.class.get_untracked();

    let base = "hf-max-pain inline-flex items-center gap-1 rounded border border-amber-500/30 bg-amber-600/20 px-2 py-0.5 text-[10px] font-bold text-amber-400";
    let classes = match extra {
        Some(c) => format!("{base} {c}"),
        None => base.to_string(),
    };

    view()
        .flex_row()
        .items_center()
        .gap(px(4.0))
        .app_class(Some(SharedString::from(classes)))
        .child(text("⬤").text_size(px(8.0)))
        .child(text(move || format!("Max Pain {:.0}", strike.get())))
}
