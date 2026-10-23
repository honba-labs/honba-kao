//! HfContextMenu — right-click (`contextmenu`) menu primitive.

use leptos::prelude::*;

#[derive(Clone, Copy)]
struct ContextMenuState {
    open: RwSignal<bool>,
    pos: RwSignal<(i32, i32)>,
}

#[component]
pub fn HfContextMenu(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let state = ContextMenuState {
        open: RwSignal::new(false),
        pos: RwSignal::new((0, 0)),
    };
    provide_context(state);

    view! {
        <div
            class=format!("hf-context-menu {class}").trim().to_string()
            data-state=move || if state.open.get() { "open" } else { "closed" }
        >
            {children()}
        </div>
    }
}

#[component]
pub fn HfContextMenuTrigger(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let state = use_context::<ContextMenuState>();

    view! {
        <div
            class=format!("hf-context-menu__trigger {class}").trim().to_string()
            on:contextmenu=move |ev: web_sys::MouseEvent| {
                ev.prevent_default();
                if let Some(s) = state {
                    s.pos.set((ev.client_x(), ev.client_y()));
                    s.open.set(true);
                }
            }
        >
            {children()}
        </div>
    }
}

#[component]
pub fn HfContextMenuContent(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let state = use_context::<ContextMenuState>();
    let is_open = move || state.map(|s| s.open.get()).unwrap_or(false);
    let style = move || {
        state.map(|s| {
            let (x, y) = s.pos.get();
            format!("left:{x}px;top:{y}px;")
        })
    };

    view! {
        <div
            role="menu"
            style=style
            class=move || {
                format!(
                    "hf-context-menu__content {} {class}",
                    if is_open() { "hf-context-menu__content--open" } else { "hf-context-menu__content--hidden" }
                )
                .trim()
                .to_string()
            }
            aria-hidden=move || (!is_open()).to_string()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn HfContextMenuItem(
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_select: Option<Callback<web_sys::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let state = use_context::<ContextMenuState>();
    let disabled = disabled.unwrap_or(false);

    view! {
        <button
            type="button"
            role="menuitem"
            disabled=disabled
            class=format!("hf-context-menu__item {class}").trim().to_string()
            on:click=move |ev| {
                if !disabled {
                    if let Some(cb) = on_select {
                        cb.run(ev);
                    }
                    if let Some(s) = state {
                        s.open.set(false);
                    }
                }
            }
        >
            {children()}
        </button>
    }
}

#[component]
pub fn HfContextMenuSeparator(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <div role="separator" class=format!("hf-context-menu__separator {class}").trim().to_string() />
    }
}
