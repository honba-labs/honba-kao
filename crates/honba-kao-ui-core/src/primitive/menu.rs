//! HfMenu — accessible dropdown menu (`role="menu"` / `role="menuitem"`).

use leptos::prelude::*;

#[component]
pub fn HfMenu(
    #[prop(optional)] default_open: Option<bool>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let open = RwSignal::new(default_open.unwrap_or(false));
    provide_context(open);

    view! {
        <div
            class=format!("hf-menu {class}").trim().to_string()
            data-state=move || if open.get() { "open" } else { "closed" }
        >
            {children()}
        </div>
    }
}

#[component]
pub fn HfMenuTrigger(
    #[prop(optional, into)] class: String,
    #[prop(optional)] disabled: Option<bool>,
    children: Children,
) -> impl IntoView {
    let open = use_context::<RwSignal<bool>>().unwrap_or_else(|| RwSignal::new(false));
    let disabled = disabled.unwrap_or(false);

    view! {
        <button
            type="button"
            class=format!("hf-menu__trigger {class}").trim().to_string()
            disabled=disabled
            aria-haspopup="menu"
            aria-expanded=move || open.get().to_string()
            on:click=move |_| {
                if !disabled {
                    open.update(|v| *v = !*v);
                }
            }
        >
            {children()}
        </button>
    }
}

#[component]
pub fn HfMenuContent(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let open = use_context::<RwSignal<bool>>().unwrap_or_else(|| RwSignal::new(false));

    view! {
        <div
            class=move || {
                format!(
                    "hf-menu__content {} {class}",
                    if open.get() { "hf-menu__content--open" } else { "hf-menu__content--hidden" }
                )
                .trim()
                .to_string()
            }
            role="menu"
            aria-hidden=move || (!open.get()).to_string()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn HfMenuItem(
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_select: Option<Callback<web_sys::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let open = use_context::<RwSignal<bool>>();
    let disabled = disabled.unwrap_or(false);

    view! {
        <button
            type="button"
            role="menuitem"
            disabled=disabled
            class=format!("hf-menu__item {class}").trim().to_string()
            on:click=move |ev| {
                if !disabled {
                    if let Some(cb) = on_select {
                        cb.run(ev);
                    }
                    if let Some(open) = open {
                        open.set(false);
                    }
                }
            }
        >
            {children()}
        </button>
    }
}

#[component]
pub fn HfMenuSeparator(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <div role="separator" class=format!("hf-menu__separator {class}").trim().to_string() />
    }
}
