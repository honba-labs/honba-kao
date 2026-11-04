//! `HfSeparator` — accessible horizontal or vertical divider stroke.
//! Ported from `secured-finance-app` atom `Separator`.

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SeparatorOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[component]
pub fn HfSeparator(
    #[prop(into, optional)] orientation: Option<SeparatorOrientation>,
    #[prop(into, optional)] class: Option<String>,
) -> impl IntoView {
    let orientation = orientation.unwrap_or_default();
    let base = match orientation {
        SeparatorOrientation::Horizontal => "hf-separator h-px w-full",
        SeparatorOrientation::Vertical => "hf-separator w-px h-full",
    };
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    view! {
        <div class=format!("{} border-border-soft", classes) />
    }
}
