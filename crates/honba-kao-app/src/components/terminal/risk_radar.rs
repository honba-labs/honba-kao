//! Risk radar: margin utilisation and exposure limits.

use leptos::prelude::*;

use crate::components::ui::{HfPanel, HfPanelHeader};
use crate::utils::{compact_currency, percent};

struct Gauge {
    name: &'static str,
    used: f64,
    limit: f64,
}

const GAUGES: &[Gauge] = &[
    Gauge {
        name: "Margin used",
        used: 84_500.0,
        limit: 500_000.0,
    },
    Gauge {
        name: "Day loss limit",
        used: 12_400.0,
        limit: 50_000.0,
    },
    Gauge {
        name: "Single-name exposure",
        used: 210_000.0,
        limit: 400_000.0,
    },
];

#[component]
pub fn RiskRadar() -> impl IntoView {
    view! {
        <HfPanel>
            <HfPanelHeader title="Risk radar" />

            <div class="hfui-panel__content gauges">
                {GAUGES
                    .iter()
                    .map(|g| {
                        let pct = (g.used / g.limit) * 100.0;
                        let level = if pct > 80.0 {
                            "risk-high"
                        } else if pct > 50.0 {
                            "risk-mid"
                        } else {
                            "risk-low"
                        };
                        view! {
                            <div class="gauge">
                                <div class="gauge__head">
                                    <span>{g.name}</span>
                                    <span class="tnum muted">
                                        {compact_currency(g.used)}" / "{compact_currency(g.limit)}
                                    </span>
                                </div>
                                <div class="gauge__track">
                                    <span
                                        class=format!("gauge__fill {}", level)
                                        style=format!("width: {:.1}%;", pct.min(100.0))
                                    />
                                </div>
                                <span class="gauge__pct mono">{percent(pct)}</span>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        </HfPanel>
    }
}
