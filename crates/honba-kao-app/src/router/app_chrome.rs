//! Shared layout wrapper for authenticated/app screens. Mirrors the `AppChrome`
//! component defined in `web/src/router/AppRoutes.tsx`.
//!
//! It renders the top bar, the iconized side rail (honba-specific), a `<main>`
//! content region, the site footer, and the login-required overlay whose open
//! state is owned here. Pages like the landing screen compose the chrome
//! themselves with `show_rail = false`.
//!
//! The chrome also owns the palette-studio modal so the rail's account cluster
//! can summon it from any page.

use leptos::prelude::*;

use crate::router::chrome::{HeaderBar, NavRail, SiteFooter};
use crate::router::paths::Page;
use crate::shared::components::{CommandBar, LoginRequiredOverlay, PaletteStudio};

#[component]
pub fn AppChrome(
    /// The page to highlight; reactive because the chrome stays mounted
    /// while the routed page beneath it changes.
    #[prop(into, default = Page::Home.into())]
    current_page: Signal<Page>,
    /// The side icon rail is shared by the workspace screens; marketing
    /// surfaces (landing) opt out.
    #[prop(default = true)]
    show_rail: bool,
    /// noFX shows `SiteFooter` on chrome pages except full-bleed workspaces.
    #[prop(default = false)]
    show_footer: bool,
    children: Children,
) -> impl IntoView {
    let login_open = RwSignal::new(false);
    let feature = RwSignal::new(String::new());
    let on_login_required = Callback::new(move |requested_feature: String| {
        feature.set(requested_feature);
        login_open.set(true);
    });

    // Chrome-level palette studio; the rail's account cluster opens it.
    let palette_open = RwSignal::new(false);

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
                        <NavRail
                            current_page=current_page
                            on_login_required=on_login_required
                            palette_open=palette_open
                        />
                    }
                })}
            <main class="app-main">
                // Persistent GO bar (Ctrl+G); lives in the chrome so it survives navigation.
                <CommandBar />
                {children()}
            </main>
            {show_footer.then(|| view! { <SiteFooter /> })}
            <PaletteStudio
                is_open=Signal::derive(move || palette_open.get())
                on_close=Callback::new(move |_| palette_open.set(false))
            />
            <LoginRequiredOverlay is_open=login_open feature=feature />
        </div>
    }
}
