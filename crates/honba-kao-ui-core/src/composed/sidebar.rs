//! HfSidebar — icon rail navigation (HfSidebarItem).
//! Mirrors bfx-hf-ui-core sidebar navigation.

use leptos::prelude::*;

#[component]
pub fn HfSidebar(
    #[prop(optional, into)]
    class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <aside class=format!("hf-sidebar {class}").trim().to_string()>
            {children()}
        </aside>
    }
}

/// HfSidebarItem — single sidebar navigation item.
#[component]
pub fn HfSidebarItem(
    /// The SVG path `d` data for the icon.
    icon: &'static str,
    /// Item label text.
    label: &'static str,
    /// Whether this item is active.
    #[prop(optional)]
    active: Option<bool>,
    #[prop(optional)]
    on_click: Option<Callback<web_sys::MouseEvent>>,
    #[prop(optional, into)]
    class: String,
) -> impl IntoView {
    let active = active.unwrap_or(false);

    view! {
        <button
            class=move || format!("hf-sidebar__item {} {}", if active { "hf-sidebar__item--active" } else { "" }, class).trim().to_string()
            on:click=move |ev| {
                if let Some(cb) = on_click {
                    cb.run(ev);
                }
            }
        >
            <svg
                viewBox="0 0 24 24"
                stroke="currentColor"
                fill="none"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                class="hf-sidebar__item-icon"
                width="16"
                height="16"
            >
                <path d=icon />
            </svg>
            <span class="hf-sidebar__item-label">{label}</span>
        </button>
    }
}