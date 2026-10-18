//! Guest view for `/traders` when signed out. Mirrors
//! `web/src/components/trader/TraderLaunchGuestPage.tsx`.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::trader::TradersList;
use crate::components::ui::PageHeader;
use crate::router::paths::ROUTES;

#[component]
pub fn TraderLaunchGuestPage() -> impl IntoView {
    view! {
        <div class="console">
            <PageHeader
                title="Launch an AI trader"
                subtitle="Sign in to configure agents — the list below is sample data"
            >
                <span class="header-bar__cta">
                    <A href=ROUTES::LOGIN>"Sign in"</A>
                </span>
            </PageHeader>

            <div class="guest-banner">
                <strong>"Read-only preview"</strong>
                <span>
                    "Creating or editing traders requires a session. Sign in to start in paper mode."
                </span>
            </div>

            <TradersList />
        </div>
    }
}
