//! Beginner onboarding page (`/welcome`). Mirrors
//! `web/src/pages/BeginnerOnboardingPage.tsx`.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::ui::{HfPanel, HfPanelHeader, PageHeader};
use crate::router::paths::ROUTES;

const STEPS: &[(&str, &str)] = &[
    (
        "1 · Pick an agent template",
        "Start from a documented strategy — trend, mean-reversion or breakout — and give it a name.",
    ),
    (
        "2 · Set your risk envelope",
        "Per-trade size, day loss limit and single-name exposure. Every order is checked against these.",
    ),
    (
        "3 · Rehearse in paper mode",
        "Run against live data with simulated fills. Watch the decision log to see what would have been blocked.",
    ),
    (
        "4 · Go live when ready",
        "Connect a broker adapter under Settings and switch the trader out of paper mode.",
    ),
];

#[component]
pub fn BeginnerOnboardingPage() -> impl IntoView {
    view! {
        <div class="console">
            <PageHeader title="Welcome to honba-kao" subtitle="Four steps to your first supervised agent">
                <span class="header-bar__cta">
                    <A href=ROUTES::TRADERS>"Go to traders"</A>
                </span>
            </PageHeader>

            <HfPanel>
                <HfPanelHeader title="Getting started" counter=STEPS.len() />
                <div class="hfui-panel__content steps">
                    {STEPS
                        .iter()
                        .map(|(title, body)| {
                            view! {
                                <div class="step">
                                    <strong>{*title}</strong>
                                    <p class="muted small">{*body}</p>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </HfPanel>
        </div>
    }
}
