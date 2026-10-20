//! Sign-in screen. Mirrors `web/src/components/auth/LoginPage.tsx`.
//!
//! honba-kao has no auth backend yet: [`crate::shared::contexts::AuthContext`] validates
//! against the in-memory accounts seeded with the demo credentials, so this
//! form behaves like the reference app's API-backed one — inline errors, then a
//! redirect — without a network round-trip.

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

use honba_kao_ui_core::HfButton;

use crate::features::auth::components::{AuthField, AuthShell};
use crate::shared::constants::auth as creds;
use crate::shared::contexts::use_auth;
use crate::router::paths::ROUTES;

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);

    let submit = move || match auth.sign_in(&email.get(), &password.get()) {
        Ok(()) => {
            error.set(None);
            navigate(ROUTES::PORTFOLIO, Default::default());
        }
        Err(message) => error.set(Some(message)),
    };

    let fill_demo = move || {
        email.set(creds::DEMO_EMAIL.to_string());
        password.set(creds::DEMO_PASSWORD.to_string());
        error.set(None);
    };

    view! {
        <AuthShell title="Sign in" subtitle="Access your agent console">
            <form
                class="auth-form"
                on:submit=move |event| {
                    event.prevent_default();
                    submit();
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
                <AuthField
                    label="Password"
                    value=password
                    input_type="password"
                    placeholder="••••••••"
                    autocomplete="current-password"
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
                    "Sign in"
                </HfButton>
            </form>

            <HfButton unstyled=true class="auth-demo" on_click=Callback::new(move |_| fill_demo())>
                <span class="auth-demo__label">"Use the demo account"</span>
                <code class="auth-demo__creds">
                    {creds::DEMO_EMAIL}" / "{creds::DEMO_PASSWORD}
                </code>
            </HfButton>

            <div class="auth-card__links">
                <A href=ROUTES::REGISTER>"Create an account"</A>
                <A href=ROUTES::RESET_PASSWORD>"Forgot password?"</A>
            </div>
        </AuthShell>
    }
}
