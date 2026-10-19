//! Shared layout wrapper for authenticated/app screens. Mirrors the `AppChrome`
//! component defined in `web/src/router/AppRoutes.tsx`.
//!
//! It renders the header bar, a `<main>` content region, and the
//! login-required overlay whose open state is owned here.

use leptos::prelude::*;

use crate::components::common::{HeaderBar, LoginRequiredOverlay};
use crate::router::paths::Page;

#[component]
pub fn AppChrome(
    #[prop(default = Page::Home)] current_page: Page,
    children: Children,
) -> impl IntoView {
    let login_open = RwSignal::new(false);
    let feature = RwSignal::new(String::new());
    let on_login_required = Callback::new(move |requested_feature: String| {
        feature.set(requested_feature);
        login_open.set(true);
    });

    view! {
        <div class="app-shell">
            <HeaderBar current_page=current_page on_login_required=on_login_required />
            <main class="app-main">{children()}</main>
            <LoginRequiredOverlay is_open=login_open feature=feature />
        </div>
    }
}
