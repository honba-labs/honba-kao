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
    let sent = RwSignal::new(false);

    view! {
        <AuthShell title="Reset password" subtitle="We'll email you a one-time link">
            <AuthField label="Email" input_type="email" placeholder="you@example.com" />

            <button
                class="hfui-button green block"
                on:click=move |_| sent.set(true)
            >
                "Send reset link"
            </button>

            {move || sent.get().then(|| view! {
                <p class="auth-card__note">
                    "If that address is on file, a reset link is on its way."
                </p>
            })}

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
