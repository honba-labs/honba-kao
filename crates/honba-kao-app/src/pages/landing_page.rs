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
    let on_login_required = Callback::new(move |requested: String| {
        feature.set(requested);
        login_open.set(true);
    });

    view! {
        <div class="app-shell">
            <HeaderBar current_page=Page::Home on_login_required=on_login_required />
            <main class="app-main app-main--landing">
                <FlowLanding />
            </main>
            <LoginRequiredOverlay is_open=login_open feature=feature />
        </div>
    }
}
