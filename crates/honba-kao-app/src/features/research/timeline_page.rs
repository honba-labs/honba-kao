use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn TimelinePage() -> impl IntoView {
    view! {
        <HfPage title="Corporate Filings & Events Timeline" subtitle="Exchange announcements, insider trading disclosures, and board meetings">
            <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
                "NSE/BSE corporate filings, dividend ex-dates, bonus issues, and AGMs."
            </div>
        </HfPage>
    }
}
