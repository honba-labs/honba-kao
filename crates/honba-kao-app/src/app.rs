//! Application root. Mirrors `web/src/App.tsx`: a provider stack wrapped
//! around the router.
//!
//! React context providers (`LanguageProvider`, `AuthProvider`,
//! `ConfirmDialogProvider`) become `provide_context(..)` calls here, and
//! Zustand stores become provided reactive stores.

use leptos::prelude::*;
use leptos_router::components::Router;

use crate::contexts::{AuthContext, ConfirmDialogContext, ConfirmDialogHost, LanguageContext};
use crate::router::AppRoutes;
use crate::theme::ThemeContext;

#[component]
pub fn App() -> impl IntoView {
    // Contexts (≈ React context providers).
    provide_context(ThemeContext::new());
    provide_context(LanguageContext::new());
    provide_context(AuthContext::new());
    provide_context(ConfirmDialogContext::new());

    view! {
        <Router>
            <AppRoutes />
        </Router>
        <ConfirmDialogHost />
    }
}
