//! HfTabs — accessible tab component with bfx-hf-ui-core styling.

use leptos::prelude::*;

/// HfTabs — tab container managing active state.
#[component]
pub fn HfTabs(
    #[prop(optional)] default_value: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let default_value = default_value.unwrap_or("");
    let active = leptos::prelude::RwSignal::new(default_value.to_string());

    view! {
        <div class="hf-tabs" data-active=move || active.get()>
            {children()}
        </div>
    }
}

/// HfTabsList — the list of tab triggers.
#[component]
pub fn HfTabsList(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <div class=format!("hf-tabs__list {class}").trim().to_string() role="tablist">
            {children()}
        </div>
    }
}

/// HfTabsTrigger — a single tab trigger.
#[component]
pub fn HfTabsTrigger(
    value: &'static str,
    #[prop(optional)] on_click: Option<Callback<web_sys::MouseEvent>>,
    #[prop(optional, into)] _class: String,
    children: Children,
) -> impl IntoView {
    let parent_active = leptos::prelude::use_context::<leptos::prelude::RwSignal<String>>()
        .unwrap_or_else(|| leptos::prelude::RwSignal::new(value.to_string()));

    let is_active = move || parent_active.get() == value;

    view! {
        <button
            class=move || format!("hf-tabs__trigger {}", if is_active() { "hf-tabs__trigger--active" } else { "" }).trim().to_string()
            role="tab"
            aria-selected=move || if is_active() { "true" } else { "false" }
            data-value=value
            on:click=move |ev| {
                parent_active.set(value.to_string());
                if let Some(cb) = on_click {
                    cb.run(ev);
                }
            }
        >
            {children()}
        </button>
    }
}

/// HfTabsContent — the content panel for a tab.
#[component]
pub fn HfTabsContent(
    value: &'static str,
    #[prop(optional, into)] _class: String,
    children: Children,
) -> impl IntoView {
    let parent_active = leptos::prelude::use_context::<leptos::prelude::RwSignal<String>>()
        .unwrap_or_else(|| leptos::prelude::RwSignal::new(value.to_string()));

    let is_active = move || parent_active.get() == value;

    view! {
        <div
            class=move || format!("hf-tabs__content {}", if is_active() { "" } else { "hf-tabs__content--hidden" }).trim().to_string()
            role="tabpanel"
            data-value=value
        >
            {children()}
        </div>
    }
}
