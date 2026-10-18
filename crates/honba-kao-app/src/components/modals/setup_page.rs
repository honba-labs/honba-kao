//! First-run setup wizard. Mirrors `web/src/components/modals/SetupPage.tsx`.
//!
//! Shown by the router when the system reports `initialized == false` and no
//! session exists (see `api::config::get_system_config`).

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::components::auth::{AuthField, AuthShell};
use crate::router::paths::ROUTES;

#[component]
pub fn SetupPage() -> impl IntoView {
    let navigate = use_navigate();

    view! {
        <AuthShell
            title="First-run setup"
            subtitle="Create the owner account for this instance"
        >
            <AuthField label="Owner name" placeholder="Demo Trader" />
            <AuthField label="Email" input_type="email" placeholder="owner@example.com" />
            <AuthField label="Master password" input_type="password" placeholder="Choose a strong password" />

            <button
                class="hfui-button green block"
                on:click=move |_| navigate(ROUTES::LOGIN, Default::default())
            >
                "Finish setup"
            </button>

            <p class="auth-card__note">
                "Trading starts in paper mode. Connect a broker later under Settings."
            </p>
        </AuthShell>
    }
}
