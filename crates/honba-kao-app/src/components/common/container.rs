//! Centered content container. Mirrors `web/src/components/common/Container.tsx`.

use leptos::prelude::*;

#[component]
pub fn Container(
    #[prop(default = "container")] class: &'static str,
    children: Children,
) -> impl IntoView {
    view! { <div class=class>{children()}</div> }
}
