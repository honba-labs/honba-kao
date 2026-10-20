//! Global site footer. Mirrors `web/src/components/common/SiteFooter.tsx`
//! (the `.nofx-site-footer` strip at the bottom of chrome-wrapped pages).

use leptos::prelude::*;

use crate::shared::constants::branding;

#[component]
pub fn SiteFooter() -> impl IntoView {
    view! {
        <footer class="nofx-site-footer">
            <div class="nofx-site-footer__brand">
                {branding::APP_NAME_LATIN}
                <span class="nofx-site-footer__tag">{branding::TAGLINE}</span>
            </div>
            <div class="nofx-site-footer__links">
                <a href=branding::REPO_URL target="_blank" rel="noreferrer noopener">
                    "GitHub"
                </a>
                <span>"v" {branding::VERSION}</span>
                <span>"AGPL-3.0"</span>
            </div>
        </footer>
    }
}
