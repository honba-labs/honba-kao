//! The route table.
//!
//! Signed-in users land on Portfolio (the console's default workspace);
//! unknown paths fall back to `/`, which itself redirects authenticated users
//! there and still shows the marketing landing to guests.
//!
//! The gated workspace screens are children of one pathless parent route
//! whose view, [`AppShell`], mounts the [`AppChrome`] once and renders the
//! matched page through `<Outlet/>`. Navigating between workspace screens
//! therefore swaps only the outlet content; the header, nav rail and their
//! state survive.

use leptos::prelude::*;
use leptos_router::components::{Outlet, ParentRoute, Redirect, Route, Routes};
use leptos_router::hooks::use_location;
use leptos_router::path;

use crate::shared::contexts::use_auth;
use crate::features::account::{FundsPage, SettingsPage};
use crate::features::auth::{LoginPage, RegisterPage, ResetPasswordPage, SetupPage};
use crate::features::chart_workstation::ChartWorkstationPage;
use crate::features::dossiers::DossiersPage;
use crate::features::dom::DomPage;
use crate::features::fno::{
    FlowPage, FuturesPage, GreeksHeatmapPage, OiPage, OptionChainPage, OptionGreeksPage,
    StrategyBuilderPage,
};
use crate::features::landing::LandingPage;
use crate::features::launchpad::{LaunchpadPage, PopoutPage};
use crate::features::mta::MtaPage;
use crate::features::portfolio::PortfolioPage;
use crate::features::screener::ScreenerPage;
use crate::features::security_hub::SecurityHubPage;
use crate::features::simulator::SimulatorPage;
use crate::features::strategy_studio::StrategyStudioPage;
use crate::features::tape::TapePage;
use crate::features::tradebook::TradebookPage;
use crate::features::workbench::WorkbenchPage;
use crate::router::app_chrome::AppChrome;
use crate::router::paths::{Page, ROUTES, get_current_page_for_path};

#[component]
pub fn AppRoutes() -> impl IntoView {
    view! {
        // Unknown paths fall through to `/`, which routes by session state.
        <Routes fallback=|| view! { <Redirect path=ROUTES::HOME /> }>
            // --- Public, full-bleed screens -----------------------------------
            <Route path=path!("/") view=|| view! { <RootRoute /> } />
            <Route path=path!("/login") view=|| view! { <LoginPage /> } />
            <Route path=path!("/register") view=|| view! { <RegisterPage /> } />
            <Route path=path!("/reset-password") view=|| view! { <ResetPasswordPage /> } />
            <Route path=path!("/setup") view=|| view! { <SetupPage /> } />
            // Pop-out window: a bare single panel, no shell chrome.
            <Route path=path!("/equity/launchpad/popout") view=|| view! { <PopoutPage /> } />

            // --- Gated workspace (persistent chrome) --------------------------
            <ParentRoute path=path!("") view=AppShell>
                <Route path=path!("/portfolio") view=PortfolioPage />
                <Route path=path!("/screener") view=ScreenerPage />
                <Route path=path!("/equity/security/:ticker") view=SecurityHubPage />
                <Route path=path!("/equity/research-dossiers") view=DossiersPage />
                <Route path=path!("/fno") view=OptionChainPage />
                <Route path=path!("/fno/strategy") view=StrategyBuilderPage />
                <Route path=path!("/fno/oi") view=OiPage />
                <Route path=path!("/fno/greeks") view=GreeksHeatmapPage />
                <Route path=path!("/fno/futures") view=FuturesPage />
                <Route path=path!("/fno/flow") view=FlowPage />
                <Route path=path!("/equity/option-greeks") view=OptionGreeksPage />
                <Route path=path!("/equity/chart-workstation") view=ChartWorkstationPage />
                <Route path=path!("/equity/mta") view=MtaPage />
                <Route path=path!("/equity/dom") view=DomPage />
                <Route path=path!("/equity/tape") view=TapePage />
                <Route path=path!("/equity/launchpad") view=LaunchpadPage />
                <Route path=path!("/workbench") view=WorkbenchPage />
                <Route path=path!("/strategy") view=StrategyStudioPage />
                <Route path=path!("/simulator") view=SimulatorPage />
                <Route path=path!("/tradebook") view=TradebookPage />

                // --- User cluster -------------------------------------------
                <Route path=path!("/settings") view=SettingsPage />
                <Route path=path!("/funds") view=FundsPage />
            </ParentRoute>
        </Routes>
    }
}

/// Layout for the gated workspace: guests are sent to sign-in, hosts get the
/// chrome mounted once with the matched child page in its `<main>` outlet.
///
/// The session is checked when the shell mounts. The shell is only left
/// mounted across navigations between its own children, and the only way a
/// session ends (sign-out) navigates to `/`, which unmounts the shell.
#[component]
fn AppShell() -> impl IntoView {
    let auth = use_auth();

    if !auth.is_authenticated() {
        return view! { <Redirect path=ROUTES::LOGIN /> }.into_any();
    }

    let location = use_location();
    let current_page = Memo::new(move |_| {
        location
            .pathname
            .with(|pathname| get_current_page_for_path(pathname).unwrap_or(Page::Home))
    });

    view! {
        <AppChrome current_page=current_page>
            <Outlet />
        </AppChrome>
    }
    .into_any()
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
