//! HfDrawer — slide-out panel anchored to a viewport edge (`bottom` by default).

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HfDrawerSide {
    Bottom,
    Right,
    Left,
    Top,
}

impl HfDrawerSide {
    pub fn as_str(self) -> &'static str {
        match self {
            HfDrawerSide::Bottom => "bottom",
            HfDrawerSide::Right => "right",
            HfDrawerSide::Left => "left",
            HfDrawerSide::Top => "top",
        }
    }
}

#[component]
pub fn HfDrawer(
    #[prop(optional)] default_open: Option<bool>,
    #[prop(optional)] side: Option<HfDrawerSide>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let open = RwSignal::new(default_open.unwrap_or(false));
    let side = side.unwrap_or(HfDrawerSide::Bottom);
    provide_context(open);

    view! {
        <div
            class=format!("hf-drawer hf-drawer--{} {class}", side.as_str()).trim().to_string()
            data-state=move || if open.get() { "open" } else { "closed" }
        >
            {children()}
        </div>
    }
}

#[component]
pub fn HfDrawerTrigger(
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_click: Option<Callback<web_sys::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let open = use_context::<RwSignal<bool>>().unwrap_or_else(|| RwSignal::new(false));

    view! {
        <button
            type="button"
            class=format!("hf-drawer__trigger {class}").trim().to_string()
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
pub fn HfDrawerContent(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let open = use_context::<RwSignal<bool>>().unwrap_or_else(|| RwSignal::new(false));

    view! {
        <div
            class=move || {
                format!(
                    "hf-drawer__content {} {class}",
                    if open.get() { "hf-drawer__content--open" } else { "hf-drawer__content--hidden" }
                )
                .trim()
                .to_string()
            }
            role="dialog"
            aria-modal="true"
            aria-hidden=move || (!open.get()).to_string()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn HfDrawerHeader(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <header class=format!("hf-drawer__header {class}").trim().to_string()>
            {children()}
        </header>
    }
}

#[component]
pub fn HfDrawerFooter(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <footer class=format!("hf-drawer__footer {class}").trim().to_string()>
            {children()}
        </footer>
    }
}
