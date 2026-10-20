//! HfSeparator — visual separator between content groups.

use leptos::prelude::*;

#[component]
pub fn HfSeparator(
    #[prop(optional, into)]
    class: String,
) -> impl IntoView {
    view! {
        <div class=format!("hf-separator {class}").trim().to_string() />
    }
}