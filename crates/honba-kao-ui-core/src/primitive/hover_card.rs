//! HfHoverCard — preview card opened on pointer hover or focus.

use leptos::prelude::*;

#[component]
pub fn HfHoverCard(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let open = RwSignal::new(false);
    provide_context(open);

    view! {
        <div
            class=format!("hf-hover-card {class}").trim().to_string()
            data-state=move || if open.get() { "open" } else { "closed" }
            on:mouseenter=move |_| open.set(true)
            on:mouseleave=move |_| open.set(false)
            on:focusin=move |_| open.set(true)
            on:focusout=move |_| open.set(false)
        >
            {children()}
        </div>
    }
}

#[component]
pub fn HfHoverCardTrigger(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <span class=format!("hf-hover-card__trigger {class}").trim().to_string()>
            {children()}
        </span>
    }
}

#[component]
pub fn HfHoverCardContent(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let open = use_context::<RwSignal<bool>>().unwrap_or_else(|| RwSignal::new(false));

    view! {
        <div
            class=move || {
                format!(
                    "hf-hover-card__content {} {class}",
                    if open.get() { "hf-hover-card__content--open" } else { "hf-hover-card__content--hidden" }
                )
                .trim()
                .to_string()
            }
            aria-hidden=move || (!open.get()).to_string()
        >
            {children()}
        </div>
    }
}
