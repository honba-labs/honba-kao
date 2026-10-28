//! HfTooltip — hover/focus tooltip primitive (`role="tooltip"`).

use leptos::prelude::*;

#[component]
pub fn HfTooltip(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let open = RwSignal::new(false);
    provide_context(open);

    view! {
        <span
            class=format!("hf-tooltip {class}").trim().to_string()
            on:mouseenter=move |_| open.set(true)
            on:mouseleave=move |_| open.set(false)
            on:focusin=move |_| open.set(true)
            on:focusout=move |_| open.set(false)
        >
            {children()}
        </span>
    }
}

#[component]
pub fn HfTooltipTrigger(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <span class=format!("hf-tooltip__trigger {class}").trim().to_string()>
            {children()}
        </span>
    }
}

#[component]
pub fn HfTooltipContent(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let open = use_context::<RwSignal<bool>>().unwrap_or_else(|| RwSignal::new(false));

    view! {
        <span
            class=move || {
                format!(
                    "hf-tooltip__content {} {class}",
                    if open.get() { "hf-tooltip__content--open" } else { "hf-tooltip__content--hidden" }
                )
                .trim()
                .to_string()
            }
            role="tooltip"
            aria-hidden=move || (!open.get()).to_string()
        >
            {children()}
        </span>
    }
}
