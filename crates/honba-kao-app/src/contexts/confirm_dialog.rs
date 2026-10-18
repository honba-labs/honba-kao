//! Confirm-dialog context. Mirrors
//! `web/src/components/common/ConfirmDialog.tsx`'s provider/host split.

use leptos::prelude::*;

use crate::components::ui::{ButtonVariant, HfButton};

#[derive(Copy, Clone)]
pub struct ConfirmDialogContext {
    pub open: RwSignal<bool>,
    pub title: RwSignal<String>,
    pub message: RwSignal<String>,
    on_confirm: RwSignal<Option<Callback<()>>>,
}

impl ConfirmDialogContext {
    pub fn new() -> Self {
        Self {
            open: RwSignal::new(false),
            title: RwSignal::new(String::new()),
            message: RwSignal::new(String::new()),
            on_confirm: RwSignal::new(None),
        }
    }

    pub fn request(
        &self,
        title: impl Into<String>,
        message: impl Into<String>,
        on_confirm: Callback<()>,
    ) {
        self.title.set(title.into());
        self.message.set(message.into());
        self.on_confirm.set(Some(on_confirm));
        self.open.set(true);
    }

    pub fn close(&self) {
        self.open.set(false);
    }

    pub fn confirm(&self) {
        if let Some(cb) = self.on_confirm.get() {
            cb.run(());
        }
        self.close();
    }
}

impl Default for ConfirmDialogContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn use_confirm_dialog() -> ConfirmDialogContext {
    expect_context::<ConfirmDialogContext>()
}

/// Renders the active confirmation, if any. Mounted once at the app root.
#[component]
pub fn ConfirmDialogHost() -> impl IntoView {
    let ctx = use_confirm_dialog();

    view! {
        {move || ctx.open.get().then(|| view! {
            <div class="hfui-modal__backdrop" on:click=move |_| ctx.close()>
                <div
                    class="hfui-modal__container"
                    on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                >
                    <div class="hfui-modal__header">
                        <span class="hfui-modal__title">{move || ctx.title.get()}</span>
                        <button class="hfui-panel__icon-button" on:click=move |_| ctx.close()>
                            "✕"
                        </button>
                    </div>
                    <div class="hfui-modal__body">
                        <p style="margin: 0 0 12px; color: var(--shade-6); font-size: 13px;">
                            {move || ctx.message.get()}
                        </p>
                        <div class="modal-actions">
                            <HfButton
                                variant=ButtonVariant::Ghost
                                on_click=Callback::new(move |_| ctx.close())
                            >
                                "Cancel"
                            </HfButton>
                            <HfButton
                                variant=ButtonVariant::Green
                                on_click=Callback::new(move |_| ctx.confirm())
                            >
                                "Confirm"
                            </HfButton>
                        </div>
                    </div>
                </div>
            </div>
        })}
    }
}
