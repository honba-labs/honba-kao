//! `HfPortfolioOverlapPill` — pairwise portfolio overlap percentage indicator.

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Overlap tier label and its Tailwind text colour: High above 50%,
/// Moderate above 25%, Low otherwise.
fn overlap_level(overlap: f64) -> (&'static str, &'static str) {
    if overlap > 50.0 {
        ("High", "text-rose-400")
    } else if overlap > 25.0 {
        ("Moderate", "text-amber-400")
    } else {
        ("Low", "text-emerald-400")
    }
}

/// Props for [`HfPortfolioOverlapPill`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfPortfolioOverlapPillProps {
    /// Overlap percentage, `0.0 ..= 100.0`.
    pub overlap: Prop<f64>,
    #[prop(default)]
    pub class: Prop<Option<String>>,
}

/// Pairwise overlap as `Overlap  NN%  Tier`.
#[component]
pub fn HfPortfolioOverlapPill(
    props: HfPortfolioOverlapPillProps,
    cx: Scope<NoEvent>,
) -> impl IntoElement {
    let _ = cx;
    let overlap = props.overlap;
    let class = props.class;

    view()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .rounded(px(4.0))
        .px(px(8.0))
        .py(px(2.0))
        .surface_base()
        .border_soft()
        .text_size(px(12.0))
        .app_class(move || {
            let base = "hf-overlap-pill font-mono";
            Some(SharedString::from(match class.get() {
                Some(extra) => format!("{base} {extra}"),
                None => base.to_string(),
            }))
        })
        .child(text("Overlap").text_muted().text_size(px(12.0)))
        .child(
            view()
                .app_class(move || Some(SharedString::from(overlap_level(overlap.get()).1)))
                .child(text(move || format!("{:.0}%", overlap.get())).text_size(px(12.0))),
        )
        .child(
            text(move || overlap_level(overlap.get()).0)
                .text_muted()
                .text_size(px(12.0)),
        )
}
