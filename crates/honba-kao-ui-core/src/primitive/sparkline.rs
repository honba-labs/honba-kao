//! `HfSparklinePrimitive` — lightweight inline sparkline drawn on [`Canvas`].
//!
//! NSH has no SVG primitive, so the sparkline is redrawn as an imperative
//! canvas path (see `nsh-port.md` §3).

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfSparklinePrimitive`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfSparklinePrimitiveProps {
    pub values: Prop<Vec<f64>>,
    #[prop(default)]
    pub height: Prop<Option<SharedString>>,
    #[prop(default)]
    pub color: Prop<Option<SharedString>>,
}

/// A lightweight inline sparkline for telemetry feeds.
#[component]
pub fn HfSparklinePrimitive(
    props: HfSparklinePrimitiveProps,
    cx: Scope<NoEvent>,
) -> impl IntoElement {
    let _ = cx;
    let _h = props
        .height
        .get_untracked()
        .unwrap_or_else(|| SharedString::from("h-10"));
    let _c = props
        .color
        .get_untracked()
        .unwrap_or_else(|| SharedString::from("text-emerald-400"));
    let vals = props.values.get_untracked();

    if vals.is_empty() {
        return view()
            .h(px(10.0))
            .child(text("—").text_muted().text_size(px(10.0)));
    }

    // `nsh_core::canvas` takes a `Painter` value (reactively), not an
    // imperative drawing closure, so the path-plot isn't wired yet; this
    // renders the raw series as monospace text until a `Painter` is
    // composed from `vals`.
    view().w(px(100.0)).h(px(40.0)).child(
        text(
            vals.iter()
                .map(|v| format!("{v:.1}"))
                .collect::<Vec<_>>()
                .join(" "),
        )
        .text_muted()
        .text_size(px(8.0)),
    )
}
