//! HfSelect — dropdown select component with bfx-hf-ui-core styling.

use leptos::prelude::*;

/// HfSelect — dropdown select trigger.
#[component]
pub fn HfSelect(
    #[prop(optional)]
    value: Option<String>,
    #[prop(optional)]
    _placeholder: Option<&'static str>,
    #[prop(optional)]
    disabled: Option<bool>,
    #[prop(optional, into)]
    class: String,
    #[prop(optional)]
    on_change: Option<Callback<web_sys::Event>>,
    children: Children,
) -> impl IntoView {
    let disabled = disabled.unwrap_or(false);

    view! {
        <select
            class=format!("hf-select {class}").trim().to_string()
            prop:value=value.unwrap_or_default()
            disabled=disabled
            on:change=move |ev| {
                if let Some(cb) = on_change {
                    cb.run(ev);
                }
            }
        >
            {children()}
        </select>
    }
}

/// HfSelectTrigger — the clickable trigger part of a select.
#[component]
pub fn HfSelectTrigger(
    #[prop(optional)]
    _value: Option<String>,
    #[prop(optional)]
    placeholder: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="hf-select__trigger">
            <span class="hf-select__value">
                {move || placeholder.unwrap_or("Select").to_string()}
            </span>
            <svg
                viewBox="0 0 24 24"
                stroke="currentColor"
                fill="none"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                class="hf-select__chevron"
                width="16"
                height="16"
            >
                <polyline points="6 9 12 15 18 9" />
            </svg>
        </div>
        {children()}
    }
}

/// HfSelectContent — the dropdown content of a select.
#[component]
pub fn HfSelectContent(
    #[prop(optional, into)]
    class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!("hf-select__content {class}").trim().to_string()>
            {children()}
        </div>
    }
}

/// HfSelectItem — a single selectable item in a select.
#[component]
pub fn HfSelectItem(
    value: &'static str,
    #[prop(optional)]
    selected: Option<bool>,
    #[prop(optional, into)]
    class: String,
    children: Children,
) -> impl IntoView {
    let selected = selected.unwrap_or(false);

    view! {
        <div
            class=format!("hf-select__item {} {}", if selected { "hf-select__item--selected" } else { "" }, class).trim().to_string()
            data-value=value
        >
            {children()}
        </div>
    }
}

/// HfSelectSeparator — separator between select items.
#[component]
pub fn HfSelectSeparator(
    #[prop(optional, into)]
    class: String,
) -> impl IntoView {
    view! {
        <div class=format!("hf-select__separator {class}").trim().to_string() />
    }
}