//! Decision log: the risk-gate audit trail.

use leptos::prelude::*;

use crate::components::ui::{HfPanel, HfPanelHeader};
use crate::types::DecisionRecord;

#[component]
pub fn DecisionLog(#[prop(optional)] decisions: Option<Vec<DecisionRecord>>) -> impl IntoView {
    let rows = move || {
        decisions
            .clone()
            .unwrap_or_else(crate::data::demo_data::demo_decisions)
    };

    view! {
        <HfPanel>
            <HfPanelHeader title="Decision log & risk audit" counter=rows().len()>
                <span class="muted small">"engine stream"</span>
            </HfPanelHeader>

            <div class="hfui-panel__content flush">
                <div class="decision-list">
                    {move || rows()
                        .into_iter()
                        .map(|item| {
                            let badge_class = if item.approved { "decision__badge ok" } else { "decision__badge reject" };
                            view! {
                                <div class="decision">
                                    <div class="decision__head">
                                        <span class="tnum muted">{item.time}</span>
                                        <span class=badge_class>
                                            {if item.approved { "APPROVED" } else { "REJECTED" }}
                                        </span>
                                    </div>
                                    <div class="decision__body">
                                        <strong>{item.action}</strong>
                                        <span class="muted small">{item.trader}</span>
                                    </div>
                                    <div class="decision__reason">
                                        "Gate: "<span class="mono">{item.reason}</span>
                                    </div>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
        </HfPanel>
    }
}
