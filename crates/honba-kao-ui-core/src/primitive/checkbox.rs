//! HfCheckbox — accessible checkbox with bfx-hf-ui-core styling.

use leptos::prelude::*;

#[component]
pub fn HfCheckbox(
    #[prop(optional)]
    checked: Option<bool>,
    #[prop(optional)]
    disabled: Option<bool>,
    #[prop(optional, into)]
    class: String,
    #[prop(optional)]
    on_change: Option<Callback<web_sys::Event>>,
    children: Children,
) -> impl IntoView {
    let checked = checked.unwrap_or(false);
    let disabled = disabled.unwrap_or(false);

    view! {
        <div
            class=format!("hf-checkbox {} {}", if checked { "hf-checkbox--checked" } else { "" }, class).trim().to_string()
        >
            <input
                type="checkbox"
                checked=checked
                disabled=disabled
                class="hf-checkbox__input"
                on:change=move |ev| {
                    if let Some(cb) = on_change {
                        cb.run(ev);
                    }
                }
            />
            <span class="hf-checkbox__box" />
            <span class="hf-checkbox__label">{children()}</span>
        </div>
    }
}