//! HfScrollArea — scrollable viewport container with token-styled scrollbars.

use leptos::prelude::*;

#[component]
pub fn HfScrollArea(
    #[prop(optional, into)] max_height: String,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let style = (!max_height.is_empty()).then(|| format!("max-height:{max_height};"));

    view! {
        <div
            class=format!("hf-scroll-area {class}").trim().to_string()
            style=style
        >
            <div class="hf-scroll-area__viewport">
                {children()}
            </div>
        </div>
    }
}
