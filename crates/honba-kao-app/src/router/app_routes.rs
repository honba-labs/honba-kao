//! The route table. Mirrors `web/src/router/AppRoutes.tsx`.
//!
//! Every path is taken from [`crate::router::paths::ROUTES`]. Authenticated
//! screens are gated inline, exactly like the reference app's
//! `isLoggedIn ? <Screen/> : <Fallback/>` expressions.

use leptos::prelude::*;
use leptos_router::components::{Redirect, Route, Routes};
use leptos_router::path;

use crate::components::auth::{LoginPage, RegisterPage, ResetPasswordPage};
use crate::components::modals::SetupPage;
use crate::components::trader::{AITradersPage, CompetitionPage, TraderLaunchGuestPage};
use crate::contexts::use_auth;
use crate::pages::{
    BeginnerOnboardingPage, DataPage, FaqPage, LandingPage, PageNotFound, PortfolioPage,
    SettingsPage, StrategyStudioPage, TraderDashboardPage, TradingPlanningPage,
};
use crate::router::app_chrome::AppChrome;
use crate::router::paths::{Page, ROUTES};

#[component]
pub fn AppRoutes() -> impl IntoView {
    let auth = use_auth();

    view! {
        <Routes fallback=|| view! { <PageNotFound /> }>
            // --- Public, full-bleed screens -----------------------------------
            <Route path=path!("/") view=|| view! { <LandingPage /> } />
            <Route path=path!("/login") view=|| view! { <LoginPage /> } />
            <Route path=path!("/register") view=|| view! { <RegisterPage /> } />
            <Route path=path!("/reset-password") view=|| view! { <ResetPasswordPage /> } />
            <Route path=path!("/setup") view=|| view! { <SetupPage /> } />

            // --- Public, chrome-wrapped ---------------------------------------
            <Route path=path!("/faq") view=move || {
                view! {
                    <AppChrome current_page=Page::Faq>
                        <FaqPage />
                    </AppChrome>
                }
            } />
            <Route path=path!("/data") view=move || {
                view! {
                    <AppChrome current_page=Page::Data>
                        <DataPage />
                    </AppChrome>
                }
            } />

            // --- Gated --------------------------------------------------------
            <Route path=path!("/welcome") view=move || {
                if auth.is_authenticated() {
                    view! {
                        <AppChrome current_page=Page::Traders>
                            <BeginnerOnboardingPage />
                        </AppChrome>
                    }
                        .into_any()
                } else {
                    view! { <Redirect path=ROUTES::LOGIN /> }.into_any()
                }
            } />

            <Route path=path!("/traders") view=move || {
                if auth.is_authenticated() {
                    view! {
                        <AppChrome current_page=Page::Traders>
                            <AITradersPage />
                        </AppChrome>
                    }
                        .into_any()
                } else {
                    view! {
                        <AppChrome current_page=Page::Traders>
                            <TraderLaunchGuestPage />
                        </AppChrome>
                    }
                        .into_any()
                }
            } />

            <Route path=path!("/dashboard") view=move || {
                if auth.is_authenticated() {
                    view! {
                        <AppChrome current_page=Page::Trader>
                            <TraderDashboardPage />
                        </AppChrome>
                    }
                        .into_any()
                } else {
                    view! { <Redirect path=ROUTES::HOME /> }.into_any()
                }
            } />

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

            <Route path=path!("/strategy") view=move || {
                if auth.is_authenticated() {
                    view! {
                        <AppChrome current_page=Page::Strategy>
                            <StrategyStudioPage />
                        </AppChrome>
                    }
                        .into_any()
                } else {
                    view! { <LandingPage /> }.into_any()
                }
            } />

            <Route path=path!("/strategy-market") view=move || {
                if auth.is_authenticated() {
                    view! { <Redirect path=ROUTES::STRATEGY /> }.into_any()
                } else {
                    view! { <LandingPage /> }.into_any()
                }
            } />

            <Route path=path!("/competition") view=move || {
                if auth.is_authenticated() {
                    view! {
                        <AppChrome current_page=Page::Competition>
                            <CompetitionPage />
                        </AppChrome>
                    }
                        .into_any()
                } else {
                    view! { <LandingPage /> }.into_any()
                }
            } />

            <Route path=path!("/settings") view=move || {
                if auth.is_authenticated() {
                    view! {
                        <AppChrome current_page=Page::Data>
                            <SettingsPage />
                        </AppChrome>
                    }
                        .into_any()
                } else {
                    view! { <Redirect path=ROUTES::LOGIN /> }.into_any()
                }
            } />

            <Route path=path!("/frontier") view=move || {
                if auth.is_authenticated() {
                    view! {
                        <AppChrome current_page=Page::Planning>
                            <TradingPlanningPage />
                        </AppChrome>
                    }
                        .into_any()
                } else {
                    view! { <Redirect path=ROUTES::LOGIN /> }.into_any()
                }
            } />

            // --- Legacy redirect (kept for deep links) ------------------------
            <Route path=path!("/planning") view=|| view! { <Redirect path=ROUTES::PLANNING /> } />
        </Routes>
    }
}
