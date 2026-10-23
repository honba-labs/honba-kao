//! HfPopover — anchored floating panel toggled by a trigger.

use leptos::prelude::*;

#[component]
pub fn HfPopover(
    #[prop(optional)] default_open: Option<bool>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let open = RwSignal::new(default_open.unwrap_or(false));
    provide_context(open);

    view! {
        <div
            class=format!("hf-popover {class}").trim().to_string()
            data-state=move || if open.get() { "open" } else { "closed" }
        >
            {children()}
        </div>
    }
}

#[component]
pub fn HfPopoverTrigger(
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_click: Option<Callback<web_sys::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let open = use_context::<RwSignal<bool>>().unwrap_or_else(|| RwSignal::new(false));

    view! {
        <button
            type="button"
            class=format!("hf-popover__trigger {class}").trim().to_string()
            aria-haspopup="dialog"
            aria-expanded=move || open.get().to_string()
            on:click=move |ev| {
                open.update(|v| *v = !*v);
                if let Some(cb) = on_click {
                    cb.run(ev);
                }
            }
        >
            {children()}
        </button>
    }
}

#[component]
pub fn HfPopoverContent(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let open = use_context::<RwSignal<bool>>().unwrap_or_else(|| RwSignal::new(false));

    view! {
        <div
            class=move || {
                format!(
                    "hf-popover__content {} {class}",
                    if open.get() { "hf-popover__content--open" } else { "hf-popover__content--hidden" }
                )
                .trim()
                .to_string()
            }
            role="dialog"
            aria-hidden=move || (!open.get()).to_string()
        >
            {children()}
        </div>
    }
}
