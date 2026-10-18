//! Modal dialog component matching bfx-hf-ui-core modal style.

use leptos::prelude::*;

#[component]
pub fn HfModal(
    title: &'static str,
    #[prop(default = false)] is_open: bool,
    #[prop(optional)] on_close: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    if !is_open {
        return None;
    }

    let handle_close = move |_| {
        if let Some(cb) = on_close {
            cb.run(());
        }
    };

    Some(view! {
        <div class="hfui-modal__backdrop" on:click=handle_close>
            <div class="hfui-modal__container" on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()>
                <div class="hfui-modal__header">
                    <span class="hfui-modal__title">{title}</span>
                    <button class="hfui-panel__icon-button" on:click=handle_close>
                        "✕"
                    </button>
                </div>
                <div class="hfui-modal__body">
                    {children()}
                </div>
            </div>
        </div>
    })
}
