//! HfAvatar — user/account avatar with image and initials fallback.

use leptos::prelude::*;

#[component]
pub fn HfAvatar(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <span class=format!("hf-avatar {class}").trim().to_string()>
            {children()}
        </span>
    }
}

#[component]
pub fn HfAvatarImage(
    #[prop(into)] src: String,
    #[prop(optional, into)] alt: String,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    view! {
        <img
            src=src
            alt=alt
            class=format!("hf-avatar__image {class}").trim().to_string()
        />
    }
}

#[component]
pub fn HfAvatarFallback(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <span class=format!("hf-avatar__fallback {class}").trim().to_string()>
            {children()}
        </span>
    }
}
