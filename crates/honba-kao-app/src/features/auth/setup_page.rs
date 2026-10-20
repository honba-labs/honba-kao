//! First-run setup wizard. Mirrors `web/src/components/modals/SetupPage.tsx`.
//!
//! Shown by the router when the system reports `initialized == false` and no
//! session exists (see `api::config::get_system_config`).

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use honba_kao_ui_core::HfButton;

use crate::features::auth::components::{AuthField, AuthShell};
use crate::shared::contexts::use_auth;
use crate::router::paths::ROUTES;

#[component]
pub fn SetupPage() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);

    let submit = move || match auth.register(&name.get(), &email.get(), &password.get()) {
        Ok(()) => navigate(ROUTES::PORTFOLIO, Default::default()),
        Err(message) => error.set(Some(message)),
    };

    view! {
        <AuthShell
            title="First-run setup"
            subtitle="Create the owner account for this instance"
        >
            <form
                class="auth-form"
                on:submit=move |event| {
                    event.prevent_default();
                    submit();
                }
            >
                <AuthField
                    label="Owner name"
                    value=name
                    placeholder="Test Trader"
                    autocomplete="name"
                    autofocus=true
                />
                <AuthField
                    label="Email"
                    value=email
                    input_type="email"
                    placeholder="owner@example.com"
                    autocomplete="username"
                />
                <AuthField
                    label="Master password"
                    value=password
                    input_type="password"
                    placeholder="At least 8 characters"
                    autocomplete="new-password"
                />

                {move || {
                    error
                        .get()
                        .map(|message| {
                            view! {
                                <p class="auth-error" role="alert">
                                    {message}
                                </p>
                            }
                        })
                }}

                <HfButton unstyled=true class="hfui-button green block" button_type="submit">
                    "Finish setup"
                </HfButton>
            </form>

            <p class="auth-card__note">
                "Trading starts in paper mode. Connect a broker later under Settings."
            </p>
        </AuthShell>
    }
}
