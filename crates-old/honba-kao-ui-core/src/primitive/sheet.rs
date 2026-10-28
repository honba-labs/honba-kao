//! HfSheet — overlay side sheet (mobile drawer) with backdrop.

use leptos::prelude::*;

#[component]
pub fn HfSheet(
    #[prop(optional, into)] open: Signal<bool>,
    #[prop(optional)] side: Option<&'static str>,
    #[prop(optional, into)] title: String,
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_close: Option<Callback<web_sys::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let side = side.unwrap_or("right");
    let aria_label = (!title.is_empty()).then_some(title);

    view! {
        <div
            class=move || {
                format!(
                    "hf-sheet hf-sheet--{side} {} {class}",
                    if open.get() { "hf-sheet--open" } else { "hf-sheet--closed" }
                )
                .trim()
                .to_string()
            }
            aria-hidden=move || (!open.get()).to_string()
        >
            <div
                class="hf-sheet__backdrop"
                aria-hidden="true"
                on:click=move |ev| {
                    if let Some(cb) = on_close {
                        cb.run(ev);
                    }
                }
            />
            <aside
                class="hf-sheet__panel"
                role="dialog"
                aria-modal="true"
                aria-label=aria_label
            >
                {children()}
            </aside>
        </div>
    }
}
