//! The server backtest report — the Leptos port of honba-fe's `report`
//! dock tab (`results-panel.tsx`), consolidating what kao used to spread
//! over the `validation` and `payload` tabs.
//!
//! Top to bottom, like fe: the run status pill and run id, the run error
//! (or the blocked-plan notice), the provenance-chipped metrics ribbon, the
//! trust badges with the not-modelled assumptions, then the raw request
//! payload with the fields the contract could not carry. The journal,
//! equity and heatmap fe nested here live on their own dock tabs.

use honba_kao_ui_core::SvgIcon;
use honba_kao_ui_core::icons::outline::alert_triangle;
use leptos::prelude::*;

use crate::features::simulator::components::metrics_ribbon::MetricsRibbon;
use crate::features::simulator::components::validation_badges::ValidationBadges;
use crate::features::simulator::stores::simulator_store::SimulatorStore;

#[component]
pub fn ReportPanel(
    /// The simulator store (run lifecycle, last plan, analytics bundle).
    store: SimulatorStore,
) -> impl IntoView {
    let run = store.run.state;
    let last_plan = store.run.last_plan;
    let analytics = store.analytics_bundle;

    let status = Memo::new(move |_| run.with(|state| state.status.as_str().to_string()));
    let run_id = Memo::new(move |_| run.with(|state| state.run_id.clone()));
    let error = Memo::new(move |_| run.with(|state| state.error.clone()));
    let blocked = Memo::new(move |_| {
        last_plan.with(|plan| plan.as_ref().is_some_and(|plan| !plan.issues.is_empty()))
    });
    let metrics = Memo::new(move |_| analytics.with(|bundle| bundle.ribbon.clone()));
    let validation = Memo::new(move |_| {
        run.with(|state| {
            state
                .response
                .as_ref()
                .and_then(|response| response.validation.as_ref().map(honba_kao_client::wire::map_validation_block))
        })
    });
    let not_modelled = Memo::new(move |_| {
        run.with(|state| {
            state
                .response
                .as_ref()
                .and_then(|response| response.assumptions.clone())
        })
        .and_then(|assumptions| assumptions.get("not_modelled").cloned())
        .and_then(|value| serde_json::from_value::<Vec<String>>(value).ok())
        .unwrap_or_default()
    });
    // fe showed `JSON.stringify(lastPlan?.request, null, 2)`; until an
    // execute has run the card reports the absence rather than a request.
    let payload = Memo::new(move |_| {
        last_plan.with(|plan| {
            plan.as_ref()
                .and_then(|plan| serde_json::to_string_pretty(&plan.request).ok())
        })
    });
    let plan_not_modelled = Memo::new(move |_| {
        last_plan.with(|plan| plan.as_ref().map(|plan| plan.not_modelled.clone()).unwrap_or_default())
    });

    view! {
        <div class="sim-report-pane">
            <div class="sim-report-head">
                <span class="sim-report-title">
                    "Results"
                    {move || run_id.get().map(|id| view! { <span class="sim-report-run-id">{id}</span> })}
                </span>
                <span class=move || format!("sim-status sim-status--{}", status.get())>
                    {move || status.get()}
                </span>
            </div>

            {move || {
                error.get().map(|message| view! {
                    <div class="sim-run-alert" role="alert">
                        <SvgIcon path=alert_triangle() size=13 />
                        <span>{message}</span>
                    </div>
                })
            }}
            <Show when=move || blocked.get()>
                <div class="sim-run-alert" role="alert">
                    <SvgIcon path=alert_triangle() size=13 />
                    <span>"The panel is incomplete and the request is blocked — nothing was sent."</span>
                </div>
            </Show>

            <MetricsRibbon metrics=metrics />
            <ValidationBadges validation=validation not_modelled=not_modelled />

            <div class="sim-report-card">
                <div class="sim-report-card__label">"Request payload"</div>
                {move || match payload.get() {
                    Some(payload) => view! { <pre class="sim-payload">{payload}</pre> }.into_any(),
                    None => view! {
                        <div class="sim-empty">"No request has been submitted yet."</div>
                    }
                    .into_any(),
                }}
                {move || {
                    let fields = plan_not_modelled.get();
                    (!fields.is_empty()).then(|| view! {
                        <div class="sim-report-card__note">
                            {format!(
                                "Not sent — the run contract has no field for {} yet (see risk R5).",
                                fields.join(" and "),
                            )}
                        </div>
                    })
                }}
            </div>
        </div>
    }
}
