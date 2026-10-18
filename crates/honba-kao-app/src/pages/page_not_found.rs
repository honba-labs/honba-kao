//! 404 page. Mirrors `web/src/pages/PageNotFound.tsx`.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::router::paths::ROUTES;

#[component]
pub fn PageNotFound() -> impl IntoView {
    view! {
        <div class="not-found">
            <span class="not-found__code">"404"</span>
            <h1 class="not-found__title">"Route not found"</h1>
            <p class="not-found__body">"That path isn't part of honba-kao."</p>
            <span class="header-bar__cta">
                <A href=ROUTES::HOME>"Go home"</A>
            </span>
        </div>
    }
}
