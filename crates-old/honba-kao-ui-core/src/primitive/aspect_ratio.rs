//! HfAspectRatio — maintains a fixed width-to-height ratio (`ratio = width / height`).

use leptos::prelude::*;

#[component]
pub fn HfAspectRatio(
    #[prop(optional)] ratio: Option<f64>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let ratio = ratio.unwrap_or(16.0 / 9.0).max(f64::EPSILON);

    view! {
        <div
            class=format!("hf-aspect-ratio {class}").trim().to_string()
            style=format!("aspect-ratio:{ratio:.4};")
        >
            {children()}
        </div>
    }
}
