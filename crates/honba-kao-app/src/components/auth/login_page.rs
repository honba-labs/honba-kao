//! Sign-in screen. Mirrors `web/src/components/auth/LoginPage.tsx`.
//!
//! The submit handler performs a demo sign-in (no backend) and navigates to the
//! console, which is what the reference app does once the API answers.

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

use crate::components::auth::{AuthField, AuthShell};
use crate::contexts::use_auth;
use crate::router::paths::ROUTES;

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    view! {
        <AuthShell title="Sign in" subtitle="Access your agent console">
            <AuthField label="Email" input_type="email" placeholder="you@example.com" />
            <AuthField label="Password" input_type="password" placeholder="••••••••" />

            <button
                class="hfui-button green block"
                on:click=move |_| {
                    auth.login("Demo Trader");
                    navigate(ROUTES::TRADERS, Default::default());
                }
            >
                "Sign in"
            </button>

            <div class="auth-card__links">
                <A href=ROUTES::REGISTER>"Create an account"</A>
                <A href=ROUTES::RESET_PASSWORD>"Forgot password?"</A>
            </div>
        </AuthShell>
    }
}
