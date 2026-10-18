//! FAQ page. Mirrors `web/src/pages/FAQPage.tsx`.

use leptos::prelude::*;

use crate::components::faq::FaqLayout;
use crate::components::ui::PageHeader;

#[component]
pub fn FaqPage() -> impl IntoView {
    view! {
        <div class="console">
            <PageHeader
                title="Frequently asked questions"
                subtitle="Getting started, risk, data and account"
            />
            <FaqLayout />
        </div>
    }
}
