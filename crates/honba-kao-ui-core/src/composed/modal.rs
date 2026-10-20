//! HfModal — confirm, alert, prompt dialogs.
//! Mirrors bfx-hf-ui-core Modal component (`.hfui-modal`).
//!
//! The modal renders unconditionally; callers mount it only while it should
//! be visible (e.g. inside `move || open.get().then(...)`), so its body is
//! built on demand and dropped on close.

use leptos::prelude::*;

use crate::icons::outline::x_close;
use crate::{HfButton, HfButtonVariant, HfIconButton};

#[component]
pub fn HfModal(
    #[prop(optional, into)]
    title: String,
    #[prop(optional)]
    on_close: Option<Callback<()>>,
    #[prop(optional, into)]
    class: String,
    children: Children,
) -> impl IntoView {
    let close = move || {
        if let Some(cb) = on_close {
            cb.run(());
        }
    };

    view! {
        <div class="hfui-modal__backdrop" on:click=move |_| close()>
            <div
                class=format!("hfui-modal__container {class}").trim().to_string()
                on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
            >
                <div class="hfui-modal__header">
                    {(!title.is_empty()).then(|| view! { <span class="hfui-modal__title">{title}</span> })}
                    <HfIconButton
                        icon=x_close()
                        size=16
                        title="Close"
                        on_click=Callback::new(move |_| close())
                    />
                </div>
                <div class="hfui-modal__body">
                    {children()}
                </div>
            </div>
        </div>
    }
}

/// HfConfirmDialog — confirmation dialog with confirm/cancel actions.
#[component]
pub fn HfConfirmDialog(
    #[prop(optional)]
    on_confirm: Option<Callback<()>>,
    #[prop(optional)]
    on_cancel: Option<Callback<()>>,
    #[prop(optional, into)]
    title: Option<String>,
    #[prop(optional, into)]
    message: Option<String>,
    #[prop(optional, into)]
    confirm_label: Option<String>,
    #[prop(optional, into)]
    cancel_label: Option<String>,
) -> impl IntoView {
    let cancel = move || {
        if let Some(cb) = on_cancel {
            cb.run(());
        }
    };
    let confirm = move || {
        if let Some(cb) = on_confirm {
            cb.run(());
        }
    };

    view! {
        <HfModal title=title.unwrap_or_default() on_close=Callback::new(move |_| cancel())>
            {message.map(|m| view! { <p class="hf-confirm__message">{m}</p> })}
            <div class="modal-actions">
                <HfButton variant=HfButtonVariant::Ghost on_click=Callback::new(move |_| cancel())>
                    {cancel_label.unwrap_or_else(|| "Cancel".to_string())}
                </HfButton>
                <HfButton variant=HfButtonVariant::Red on_click=Callback::new(move |_| confirm())>
                    {confirm_label.unwrap_or_else(|| "Confirm".to_string())}
                </HfButton>
            </div>
        </HfModal>
    }
}
