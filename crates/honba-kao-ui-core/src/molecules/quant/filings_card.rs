//! `HfFilingsCard` — corporate filing extract: a bold title over the excerpt.

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfFilingsCard`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfFilingsCardProps {
    pub title: Prop<String>,
    pub excerpt: Prop<String>,
    /// Extra Tailwind classes, applied after the card's own.
    #[prop(default)]
    pub class: Prop<Option<String>>,
}

/// Pure display: a bordered surface with the filing's title and excerpt.
///
/// The Leptos original used `whitespace-pre-wrap`; nsh has no equivalent
/// style, so the excerpt is laid out as ordinary wrapped text (line breaks
/// in the source string are not preserved).
#[component]
pub fn HfFilingsCard(props: HfFilingsCardProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let title = props.title;
    let excerpt = props.excerpt;
    let extra = props.class.get_untracked();

    let base = "hf-filings-card";
    let classes = match extra {
        Some(c) => format!("{base} {c}"),
        None => base.to_string(),
    };

    view()
        .flex_col()
        .gap(px(4.0))
        .p(px(12.0))
        .rounded(px(4.0))
        .surface_base()
        .border_soft()
        .text_size(px(12.0))
        .app_class(Some(SharedString::from(classes)))
        .child(
            text(move || title.get())
                .font_weight(FontWeight::SEMIBOLD)
                .text_size(px(12.0))
                .text_high(),
        )
        .child(text(move || excerpt.get()).text_size(px(12.0)).text_high())
}
