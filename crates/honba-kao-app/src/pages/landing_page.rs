//! Landing page. Mirrors `web/src/pages/LandingPage.tsx`.
//!
//! The marketing surface runs inside the normal app chrome, so the theme
//! picker, language switch and sign-in entry work before authentication.

use leptos::prelude::*;

use crate::components::common::{HeaderBar, LoginRequiredOverlay};
use crate::components::landing::FlowLanding;
use crate::router::paths::Page;

#[component]
pub fn LandingPage() -> impl IntoView {
    let login_open = RwSignal::new(false);
    let feature = RwSignal::new(String::new());

    view! {
        <div class="app-shell app-shell--no-rail">
            <HeaderBar current_page=Page::Home />
            <main class="app-main app-main--landing">
                <FlowLanding />
            </main>
            <LoginRequiredOverlay is_open=login_open feature=feature />
        </div>
    }
}
