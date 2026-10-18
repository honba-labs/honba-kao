//! Marketing footer. Mirrors `web/src/components/common/SiteFooter.tsx`.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::ui::{CircleStatus, HfStatusCircle};
use crate::constants::branding;
use crate::router::paths::ROUTES;

#[component]
pub fn SiteFooter() -> impl IntoView {
    view! {
        <footer class="site-footer">
            <div class="site-footer__row">
                <span class="site-footer__brand">
                    {branding::APP_NAME}" · "{branding::APP_NAME_LATIN}
                </span>
                <span class="site-footer__status">
                    <HfStatusCircle status=CircleStatus::Green />
                    "All systems nominal"
                </span>
            </div>

            <nav class="site-footer__links">
                <A href=ROUTES::FAQ>"FAQ"</A>
                <A href=ROUTES::DATA>"Data"</A>
                <A href=ROUTES::SETTINGS>"Settings"</A>
                <A href=ROUTES::LOGIN>"Sign in"</A>
            </nav>

            <span class="site-footer__meta">
                "v"{branding::VERSION}" · paper-trading build · sample data"
            </span>
        </footer>
    }
}
