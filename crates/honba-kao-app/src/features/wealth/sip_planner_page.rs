use honba_kao_ui_core::molecules::wealth::HfSipCalculatorCard;
use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn SipPlannerPage() -> impl IntoView {
    let monthly = RwSignal::new(10_000.0);
    let years = RwSignal::new(10.0);
    let annual_return = RwSignal::new(12.0);

    view! {
        <HfPage title="SIP Planner & Mandate Organizer" subtitle="Systematic Investment Plan projection, step-up calculator, and cashflow modeling">
            <div class="max-w-xl mx-auto">
                <HfSipCalculatorCard
                    monthly=monthly
                    years=years
                    annual_return=annual_return
                />
            </div>
        </HfPage>
    }
}
