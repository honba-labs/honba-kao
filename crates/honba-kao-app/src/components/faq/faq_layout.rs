//! FAQ layout: category sidebar + content. Mirrors
//! `web/src/components/faq/FAQLayout.tsx`.

use leptos::prelude::*;

use crate::components::faq::{FaqContent, FaqSidebar};

#[component]
pub fn FaqLayout() -> impl IntoView {
    let active = RwSignal::new(String::new());

    view! {
        <div class="faq-layout">
            <FaqSidebar active=active />
            <FaqContent active=active />
        </div>
    }
}
