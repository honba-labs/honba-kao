//! Registration screen. Mirrors `web/src/components/auth/RegisterPage.tsx`.

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

use crate::components::auth::{AuthField, AuthShell};
use crate::contexts::use_auth;
use crate::router::paths::ROUTES;

#[component]
pub fn RegisterPage() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    view! {
        <AuthShell title="Create account" subtitle="Start with paper trading — no broker needed">
            <AuthField label="Full name" placeholder="Demo Trader" />
            <AuthField label="Email" input_type="email" placeholder="you@example.com" />
            <AuthField label="Password" input_type="password" placeholder="At least 10 characters" />

            <button
                class="hfui-button green block"
                on:click=move |_| {
                    auth.login("Demo Trader");
                    navigate(ROUTES::WELCOME, Default::default());
                }
            >
                "Create account"
            </button>

            <div class="auth-card__links">
                <A href=ROUTES::LOGIN>"I already have an account"</A>
            </div>
        </AuthShell>
    }
}
