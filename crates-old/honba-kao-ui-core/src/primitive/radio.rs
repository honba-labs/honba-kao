//! HfRadioGroup — accessible radio group with bfx-hf-ui-core styling.

use leptos::prelude::*;

#[component]
pub fn HfRadioGroup(
    #[prop(optional)] _value: Option<String>,
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] _on_change: Option<Callback<web_sys::Event>>,
    children: Children,
) -> impl IntoView {
    let _disabled = disabled.unwrap_or(false);

    view! {
        <div
            class=format!("hf-radio-group {class}").trim().to_string()
            role="radiogroup"
        >
            {children()}
        </div>
    }
}

/// HfRadioGroupItem — single radio item.
#[component]
pub fn HfRadioGroupItem(
    value: &'static str,
    #[prop(optional)] checked: Option<bool>,
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_change: Option<Callback<web_sys::Event>>,
    children: Children,
) -> impl IntoView {
    let checked = checked.unwrap_or(false);
    let disabled = disabled.unwrap_or(false);

    view! {
        <div
            class=format!("hf-radio-group__item {} {}", if checked { "hf-radio-group__item--checked" } else { "" }, class).trim().to_string()
        >
            <input
                type="radio"
                value=value
                checked=checked
                disabled=disabled
                class="hf-radio-group__input"
                on:change=move |ev| {
                    if let Some(cb) = on_change {
                        cb.run(ev);
                    }
                }
            />
            <span class="hf-radio-group__circle" />
            <span class="hf-radio-group__label">{children()}</span>
        </div>
    }
}
