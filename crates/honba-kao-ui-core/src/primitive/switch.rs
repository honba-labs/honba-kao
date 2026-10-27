//! HfSwitch — accessible toggle switch with bfx-hf-ui-core styling.

use leptos::prelude::*;

#[component]
pub fn HfSwitch(
    #[prop(optional)] checked: Option<bool>,
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_change: Option<Callback<web_sys::MouseEvent>>,
) -> impl IntoView {
    let checked = checked.unwrap_or(false);
    let disabled = disabled.unwrap_or(false);

    view! {
        <button
            class=format!("hf-switch {} {}", if checked { "hf-switch--checked" } else { "" }, class).trim().to_string()
            role="switch"
            aria-checked=checked
            disabled=disabled
            on:click=move |ev| {
                if !disabled {
                    if let Some(cb) = on_change {
                        cb.run(ev);
                    }
                }
            }
        >
            <span class="hf-switch__thumb" />
        </button>
    }
}
