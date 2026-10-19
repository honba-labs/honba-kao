//! Shared layout wrapper for authenticated/app screens. Mirrors the `AppChrome`
//! component defined in `web/src/router/AppRoutes.tsx`.
//!
//! It renders the top bar, the iconized side rail (honba-specific), a `<main>`
//! content region, the site footer, and the login-required overlay whose open
//! state is owned here. Pages like the landing screen compose the chrome
//! themselves with `show_rail = false`.

use leptos::prelude::*;

use crate::components::common::{HeaderBar, LoginRequiredOverlay, NavRail, SiteFooter};
use crate::router::paths::Page;

#[component]
pub fn AppChrome(
    #[prop(default = Page::Home)] current_page: Page,
    /// The side icon rail is shared by the workspace screens; marketing
    /// surfaces (landing) opt out.
    #[prop(default = true)] show_rail: bool,
    /// noFX shows `SiteFooter` on chrome pages except full-bleed workspaces.
    #[prop(default = false)] show_footer: bool,
    children: Children,
) -> impl IntoView {
    let login_open = RwSignal::new(false);
    let feature = RwSignal::new(String::new());
    let on_login_required = Callback::new(move |requested_feature: String| {
        feature.set(requested_feature);
        login_open.set(true);
    });

    view! {
        <div
            class=if show_rail {
                "app-shell"
            } else {
                "app-shell app-shell--no-rail"
            }
        >
            <HeaderBar current_page=current_page />
            {show_rail
                .then(|| {
                    view! {
                        <NavRail current_page=current_page on_login_required=on_login_required />
                    }
                })}
            <main class="app-main">{children()}</main>
            {show_footer.then(|| view! { <SiteFooter /> })}
            <LoginRequiredOverlay is_open=login_open feature=feature />
        </div>
    }
}
