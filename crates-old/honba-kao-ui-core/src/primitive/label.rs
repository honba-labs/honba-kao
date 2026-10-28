//! HfLabel — accessible form label with bfx-hf-ui-core styling.

use leptos::prelude::*;

#[component]
pub fn HfLabel(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <label class=format!("hf-label {class}").trim().to_string()>
            {children()}
        </label>
    }
}
