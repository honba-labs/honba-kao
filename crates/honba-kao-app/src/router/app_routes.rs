//! The route table.
//!
//! Signed-in users land on Portfolio (the console's default workspace);
//! unknown paths fall back to `/`, which itself redirects authenticated users
//! there and still shows the marketing landing to guests. Gated screens are
//! gated inline, exactly like the old `web/src/router/AppRoutes.tsx`
//! (`isLoggedIn ? <Screen/> : <Redirect/>`) expressions.

use leptos::prelude::*;
use leptos_router::components::{Redirect, Route, Routes};
use leptos_router::path;

use crate::components::auth::{LoginPage, RegisterPage, ResetPasswordPage};
use crate::components::modals::SetupPage;
use crate::contexts::use_auth;
use crate::pages::{
    FundsPage, LandingPage, PortfolioPage, ScreenerPage, SettingsPage, SimulatorPage,
    StrategyStudioPage, TradebookPage, WorkbenchPage,
};
use crate::router::app_chrome::AppChrome;
use crate::router::paths::{Page, ROUTES};

#[component]
pub fn AppRoutes() -> impl IntoView {
    let auth = use_auth();

    view! {
        // Unknown paths fall through to `/`, which routes by session state.
        <Routes fallback=|| view! { <Redirect path=ROUTES::HOME /> }>
            // --- Public, full-bleed screens -----------------------------------
            <Route path=path!("/") view=|| view! { <RootRoute /> } />
            <Route path=path!("/login") view=|| view! { <LoginPage /> } />
            <Route path=path!("/register") view=|| view! { <RegisterPage /> } />
            <Route path=path!("/reset-password") view=|| view! { <ResetPasswordPage /> } />
            <Route path=path!("/setup") view=|| view! { <SetupPage /> } />

            // --- Gated workspace ----------------------------------------------
            <Route path=path!("/portfolio") view=move || {
                if auth.is_authenticated() {
                    view! {
                        <AppChrome current_page=Page::Portfolio>
                            <PortfolioPage />
                        </AppChrome>
                    }
                        .into_any()
                } else {
                    view! { <Redirect path=ROUTES::LOGIN /> }.into_any()
                }
            } />

            <Route path=path!("/screener") view=move || {
                if auth.is_authenticated() {
                    view! {
                        <AppChrome current_page=Page::Screener>
                            <ScreenerPage />
                        </AppChrome>
                    }
                        .into_any()
                } else {
                    view! { <Redirect path=ROUTES::LOGIN /> }.into_any()
                }
            } />

            <Route path=path!("/workbench") view=move || {
                if auth.is_authenticated() {
                    view! {
                        <AppChrome current_page=Page::Workbench>
                            <WorkbenchPage />
                        </AppChrome>
                    }
                        .into_any()
                } else {
                    view! { <Redirect path=ROUTES::LOGIN /> }.into_any()
                }
            } />

            <Route path=path!("/strategy") view=move || {
                if auth.is_authenticated() {
                    view! {
                        <AppChrome current_page=Page::Strategy>
                            <StrategyStudioPage />
                        </AppChrome>
                    }
                        .into_any()
                } else {
                    view! { <Redirect path=ROUTES::LOGIN /> }.into_any()
                }
            } />

            <Route path=path!("/simulator") view=move || {
                if auth.is_authenticated() {
                    view! {
                        <AppChrome current_page=Page::Simulator>
                            <SimulatorPage />
                        </AppChrome>
                    }
                        .into_any()
                } else {
                    view! { <Redirect path=ROUTES::LOGIN /> }.into_any()
                }
            } />

            <Route path=path!("/tradebook") view=move || {
                if auth.is_authenticated() {
                    view! {
                        <AppChrome current_page=Page::Tradebook>
                            <TradebookPage />
                        </AppChrome>
                    }
                        .into_any()
                } else {
                    view! { <Redirect path=ROUTES::LOGIN /> }.into_any()
                }
            } />

            // --- User cluster ---------------------------------------------------
            <Route path=path!("/settings") view=move || {
                if auth.is_authenticated() {
                    view! {
                        <AppChrome current_page=Page::Settings>
                            <SettingsPage />
                        </AppChrome>
                    }
                        .into_any()
                } else {
                    view! { <Redirect path=ROUTES::LOGIN /> }.into_any()
                }
            } />

            <Route path=path!("/funds") view=move || {
                if auth.is_authenticated() {
                    view! {
                        <AppChrome current_page=Page::Funds>
                            <FundsPage />
                        </AppChrome>
                    }
                        .into_any()
                } else {
                    view! { <Redirect path=ROUTES::LOGIN /> }.into_any()
                }
            } />
        </Routes>
    }
}

/// `/`: authenticated users go straight to Portfolio (the default landing);
/// guests see the marketing landing.
#[component]
fn RootRoute() -> impl IntoView {
    let auth = use_auth();

    view! {
        {if auth.is_authenticated() {
            view! { <Redirect path=ROUTES::PORTFOLIO /> }.into_any()
        } else {
            view! { <LandingPage /> }.into_any()
        }}
    }
}
