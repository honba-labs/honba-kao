//! Brand mark. Mirrors `web/src/components/common/BrandLogo.tsx`.

use leptos::prelude::*;

use crate::constants::branding;

#[component]
pub fn BrandLogo(#[prop(default = 36)] size: u32) -> impl IntoView {
    view! {
        <span
            class="brand-logo"
            style=format!("width: {0}px; height: {0}px;", size)
            aria-label=branding::APP_NAME_LATIN
        >
            "本"
        </span>
    }
}
