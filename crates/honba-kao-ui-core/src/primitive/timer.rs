//! `HfTimer` — countdown timer [`Text`] + signal.

use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfTimer`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfTimerProps {
    /// Seconds remaining (or elapsed).
    pub seconds: Prop<u64>,
}

/// A countdown timer display.
#[component]
pub fn HfTimer(props: HfTimerProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let secs = props.seconds.get_untracked();

    let (m, s) = (secs / 60, secs % 60);

    view().flex().child(
        text(format!("{m:02}:{s:02}"))
            .text_high()
            .font_weight(FontWeight::BOLD)
            .font_family("monospace"),
    )
}
