//! HfCollapsible — single expandable/collapsible section.

use leptos::prelude::*;

#[component]
pub fn HfCollapsible(
    #[prop(optional)] default_open: Option<bool>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let open = RwSignal::new(default_open.unwrap_or(false));
    provide_context(open);

    view! {
        <div
            class=format!("hf-collapsible {class}").trim().to_string()
            data-state=move || if open.get() { "open" } else { "closed" }
        >
            {children()}
        </div>
    }
}

#[component]
pub fn HfCollapsibleTrigger(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let open = use_context::<RwSignal<bool>>().unwrap_or_else(|| RwSignal::new(false));

    view! {
        <button
            type="button"
            class=format!("hf-collapsible__trigger {class}").trim().to_string()
            aria-expanded=move || open.get().to_string()
            on:click=move |_| open.update(|v| *v = !*v)
        >
            {children()}
        </button>
    }
}

#[component]
pub fn HfCollapsibleContent(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let open = use_context::<RwSignal<bool>>().unwrap_or_else(|| RwSignal::new(false));

    view! {
        <div
            class=move || {
                format!(
                    "hf-collapsible__content {} {class}",
                    if open.get() { "hf-collapsible__content--open" } else { "hf-collapsible__content--hidden" }
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
