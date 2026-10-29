//! `HfItayosePhaseCard` — call-auction (Itayose) phase timer and clearing info.

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfItayosePhaseCard`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfItayosePhaseCardProps {
    pub phase: Prop<String>,
    /// Seconds remaining in the current auction phase. A count, not money —
    /// plain `i64`.
    pub seconds_remaining: Prop<i64>,
}

/// Call-auction (Itayose) phase timer: the current phase name and a `MM:SS`
/// countdown to the next clearing.
#[component]
pub fn HfItayosePhaseCard(props: HfItayosePhaseCardProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let phase = props.phase.get_untracked();
    let secs = props.seconds_remaining.get_untracked().max(0);
    let formatted = format!("{:02}:{:02}", secs / 60, secs % 60);

    view()
        .flex_col()
        .rounded(px(8.0))
        .border_soft()
        .surface_base()
        .p(px(12.0))
        .text_size(px(12.0))
        .app_class(Some(SharedString::from("hf-itayose-card")))
        .child(
            text("Call Auction (Itayose)")
                .text_muted()
                .uppercase()
                .letter_spacing(px(0.5))
                .text_size(px(10.0)),
        )
        .child(
            view()
                .flex_row()
                .items_baseline()
                .gap(px(4.0))
                .mt(px(4.0))
                .child(text(phase).text_high().font_weight(FontWeight::SEMIBOLD))
                .child(text(formatted).text_positive().font_family("monospace")),
        )
}
