//! HfSkeleton — loading placeholder block (`aria-busy="true"`).

use leptos::prelude::*;

#[component]
pub fn HfSkeleton(
    #[prop(optional, into)] width: String,
    #[prop(optional, into)] height: String,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let mut style = String::new();
    if !width.is_empty() {
        style.push_str(&format!("width:{width};"));
    }
    if !height.is_empty() {
        style.push_str(&format!("height:{height};"));
    }
    let style = (!style.is_empty()).then_some(style);

    view! {
        <div
            class=format!("hf-skeleton {class}").trim().to_string()
            style=style
            aria-hidden="true"
            aria-busy="true"
        />
    }
}
