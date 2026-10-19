//! Registration screen. Mirrors `web/src/components/auth/RegisterPage.tsx`.

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

use crate::components::auth::{AuthField, AuthShell};
use crate::constants::auth as creds;
use crate::contexts::use_auth;
use crate::router::paths::ROUTES;

#[component]
pub fn RegisterPage() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);

    let submit = move || match auth.register(&name.get(), &email.get(), &password.get()) {
        Ok(()) => {
            error.set(None);
            navigate(ROUTES::WELCOME, Default::default());
        }
        Err(message) => error.set(Some(message)),
    };

    view! {
        <AuthShell
            title="Create account"
            subtitle="Start with paper trading — no broker needed"
        >
            <form
                class="auth-form"
                on:submit=move |event| {
                    event.prevent_default();
                    submit();
                }
            >
                <AuthField
                    label="Full name"
                    value=name
                    placeholder="Test Trader"
                    autocomplete="name"
                    autofocus=true
                />
                <AuthField
                    label="Email"
                    value=email
                    input_type="email"
                    placeholder="you@example.com"
                    autocomplete="username"
                />
                <AuthField
                    label="Password"
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

                <button class="hfui-button green block" type="submit">
                    "Create account"
                </button>
            </form>

            <p class="auth-card__note">
                "Or sign in with the demo account "
                <code>{creds::DEMO_EMAIL}" / "{creds::DEMO_PASSWORD}</code>
            </p>

            <div class="auth-card__links">
                <A href=ROUTES::LOGIN>"I already have an account"</A>
            </div>
        </AuthShell>
    }
}
