//! Password reset screen. Mirrors
//! `web/src/components/auth/ResetPasswordPage.tsx`.

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

use crate::components::auth::{AuthField, AuthShell};
use crate::router::paths::ROUTES;

#[component]
pub fn ResetPasswordPage() -> impl IntoView {
    let navigate = use_navigate();
    let email = RwSignal::new(String::new());
    let sent = RwSignal::new(false);

    view! {
        <AuthShell title="Reset password" subtitle="We'll email you a one-time link">
            <form
                class="auth-form"
                on:submit=move |event| {
                    event.prevent_default();
                    if !email.get().trim().is_empty() {
                        sent.set(true);
                    }
                }
            >
                <AuthField
                    label="Email"
                    value=email
                    input_type="email"
                    placeholder="you@example.com"
                    autocomplete="username"
                    autofocus=true
                />

                <button class="hfui-button green block" type="submit">
                    "Send reset link"
                </button>
            </form>

            {move || {
                sent
                    .get()
                    .then(|| {
                        view! {
                            <p class="auth-card__note">
                                "If that address is on file, a reset link is on its way."
                            </p>
                        }
                    })
            }}

            <div class="auth-card__links">
                <A href=ROUTES::LOGIN>"Back to sign in"</A>
                <button
                    class="link-button"
                    on:click=move |_| navigate(ROUTES::HOME, Default::default())
                >
                    "Home"
                </button>
            </div>
        </AuthShell>
    }
}
