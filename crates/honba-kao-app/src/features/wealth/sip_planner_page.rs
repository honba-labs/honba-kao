use honba_kao_ui_core::molecules::wealth::HfSipCalculatorCard;
use honba_kao_ui_core::templates::{HfPage, HfTwoColumnSplitLayout};
use leptos::prelude::*;

#[component]
pub fn SipPlannerPage() -> impl IntoView {
    let monthly = RwSignal::new(10_000.0);
    let years = RwSignal::new(10.0);
    let annual_return = RwSignal::new(12.0);

    let primary = view! {
        <div class="max-w-xl mx-auto">
            <HfSipCalculatorCard
                monthly=monthly
                years=years
                annual_return=annual_return
            />
        </div>
    }
    .into_any();

    let rail = view! {
        <div class="p-4 rounded border border-border-soft bg-surface-base flex flex-col gap-3">
            <h3 class="text-xs font-semibold text-text-primary mb-1">"Mandate Controls"</h3>
            <span class="text-xs text-text-muted">"Step-up, pause, and SWP rules."</span>
        </div>
    }
    .into_any();

    view! {
        <HfPage
            title="SIP Planner & Mandate Organizer"
            subtitle="Systematic Investment Plan projection, step-up calculator, and cashflow modeling"
        >
            <HfTwoColumnSplitLayout primary=primary rail=rail />
        </HfPage>
    }
}
