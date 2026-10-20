//! HfModal — confirm, alert, prompt dialogs.
//! Mirrors bfx-hf-ui-core Modal component.

use leptos::prelude::*;
use crate::{HfButton, HfButtonVariant};

/// HfModal — confirm, alert, prompt dialogs.
/// Uses CSS to show/hide instead of conditional rendering to avoid Send/Sync issues.
#[component]
pub fn HfModal(
    #[prop(optional)]
    open: Option<bool>,
    #[prop(optional)]
    on_close: Option<Callback<()>>,
    #[prop(optional, into)]
    title: Option<String>,
    #[prop(optional, into)]
    class: String,
    /// Modal body content
    #[prop(optional, into)]
    body: Option<AnyView>,
) -> impl IntoView {
    let open = open.unwrap_or(false);
    let title_signal = leptos::prelude::RwSignal::new(title);
    // Unwrap body
    let body_view: AnyView = body.unwrap_or_else(|| ().into_any());
    let display_style = if open { "display: flex" } else { "display: none" };

    view! {
        <div class="hf-modal__backdrop" style=display_style on:click=move |_| {
            if let Some(cb) = on_close {
                cb.run(());
            }
        }>
            <div class=format!("hf-modal__container {class}").trim().to_string() on:click=move |ev| ev.stop_propagation()>
                <div class="hf-modal__header">
                    {move || title_signal.get().map(|t| view! { <h3 class="hf-modal__title">{t}</h3> }.into_any()).unwrap_or_else(|| ().into_any())}
                    <button class="hf-modal__close" on:click=move |_| {
                        if let Some(cb) = on_close {
                            cb.run(());
                        }
                    }>
                        "×"
                    </button>
                </div>
                <div class="hf-modal__body">
                    {body_view}
                </div>
            </div>
        </div>
    }
}

/// HfConfirmDialog — confirmation dialog with confirm/cancel actions.
#[component]
pub fn HfConfirmDialog(
    #[prop(optional)]
    open: Option<bool>,
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
    let open = open.unwrap_or(false);
    
    let on_close_cb = {
        let on_cancel = on_cancel;
        Callback::new(move |_| {
            if let Some(cb) = on_cancel {
                cb.run(());
            }
        })
    };

    let on_cancel_click = {
        let on_cancel = on_cancel;
        Callback::new(move |_| {
            if let Some(cb) = on_cancel {
                cb.run(());
            }
        })
    };

    let on_confirm_click = {
        let on_confirm = on_confirm;
        Callback::new(move |_| {
            if let Some(cb) = on_confirm {
                cb.run(());
            }
        })
    };

    let msg = message.clone();
    let confirm_body: AnyView = view! {
        {move || msg.as_ref().map(|m| view! { <p class="hf-confirm__message">{m.clone()}</p> }.into_any()).unwrap_or_else(move || ().into_any())}
        <div class="hf-confirm__actions">
            <HfButton variant=HfButtonVariant::Ghost on_click=on_cancel_click>
                {cancel_label.unwrap_or_else(|| "Cancel".to_string())}
            </HfButton>
            <HfButton variant=HfButtonVariant::Red on_click=on_confirm_click>
                {confirm_label.unwrap_or_else(|| "Confirm".to_string())}
            </HfButton>
        </div>
    }.into_any();

    view! {
        <HfModal open=open on_close=on_close_cb title=title.unwrap_or_default() body=confirm_body />
    }
}