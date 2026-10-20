//! Application root. Mirrors `web/src/App.tsx`: a provider stack wrapped
//! around the router.
//!
//! React context providers (`LanguageProvider`, `AuthProvider`,
//! `ConfirmDialogProvider`) become `provide_context(..)` calls here, and
//! Zustand stores become provided reactive stores.

use leptos::prelude::*;
use leptos_router::components::Router;

use crate::shared::contexts::{AuthContext, ConfirmDialogContext, ConfirmDialogHost, LanguageContext};
use crate::shared::stores::TradingModeStore;
use crate::router::AppRoutes;
use crate::shared::theme::ThemeContext;

#[component]
pub fn App() -> impl IntoView {
    // Contexts (≈ React context providers).
    provide_context(ThemeContext::new());
    provide_context(LanguageContext::new());
    provide_context(AuthContext::new());
    provide_context(TradingModeStore::new());
    provide_context(ConfirmDialogContext::new());

    view! {
        <Router>
            <AppRoutes />
        </Router>
        <ConfirmDialogHost />
    }
}
