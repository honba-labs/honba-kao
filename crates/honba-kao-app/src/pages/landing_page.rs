//! Landing page. Mirrors `web/src/pages/LandingPage.tsx`.
//!
//! The landing page carries its own marketing nav (no app chrome).

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::common::{BrandLogo, SiteFooter};
use crate::components::landing::{BrandFeatures, BrandHero, BrandStats};
use crate::constants::branding;
use crate::router::paths::ROUTES;

#[component]
pub fn LandingPage() -> impl IntoView {
    view! {
        <div class="landing">
            <header class="landing__nav">
                <div class="header-bar__brand">
                    <BrandLogo />
                    <div class="header-bar__brand-text">
                        <span class="header-bar__brand-name">{branding::APP_NAME}</span>
                        <span class="header-bar__brand-sub">{branding::APP_NAME_LATIN}</span>
                    </div>
                </div>

                <nav class="landing__nav-links">
                    <A href=ROUTES::FAQ>"FAQ"</A>
                    <A href=ROUTES::DATA>"Data"</A>
                </nav>

                <div class="header-bar__spacer" />

                <span class="header-bar__cta header-bar__cta--ghost">
                    <A href=ROUTES::LOGIN>"Sign in"</A>
                </span>
                <span class="header-bar__cta">
                    <A href=ROUTES::REGISTER>"Get started"</A>
                </span>
            </header>

            <BrandHero />
            <BrandStats />
            <BrandFeatures />
            <SiteFooter />
        </div>
    }
}
