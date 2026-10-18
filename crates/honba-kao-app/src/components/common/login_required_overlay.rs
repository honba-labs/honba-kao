//! Login-required overlay. Mirrors
//! `web/src/components/auth/LoginRequiredOverlay.tsx`.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::router::paths::ROUTES;

#[component]
pub fn LoginRequiredOverlay(is_open: RwSignal<bool>, feature: RwSignal<String>) -> impl IntoView {
    view! {
        {move || is_open.get().then(|| view! {
            <div class="hfui-modal__backdrop" on:click=move |_| is_open.set(false)>
                <div
                    class="hfui-modal__container"
                    on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                >
                    <div class="hfui-modal__header">
                        <span class="hfui-modal__title">"Sign in to continue"</span>
                        <button
                            class="hfui-panel__icon-button"
                            on:click=move |_| is_open.set(false)
                        >
                            "✕"
                        </button>
                    </div>
                    <div class="hfui-modal__body">
                        <p style="margin: 0 0 14px; color: var(--shade-6); font-size: 13px;">
                            <strong style="color: var(--shade-9);">{move || feature.get()}</strong>
                            " requires a signed-in session."
                        </p>
                        <div class="modal-actions">
                            <span class="header-bar__cta">
                                <A href=ROUTES::LOGIN>"Sign in"</A>
                            </span>
                        </div>
                    </div>
                </div>
            </div>
        })}
    }
}
