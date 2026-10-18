//! Stats strip. Mirrors `web/src/components/landing/brand/BrandStats.tsx`.

use leptos::prelude::*;

struct Stat {
    value: &'static str,
    label: &'static str,
}

const STATS: &[Stat] = &[
    Stat {
        value: "3",
        label: "agent templates",
    },
    Stat {
        value: "< 15 ms",
        label: "median gate latency",
    },
    Stat {
        value: "100%",
        label: "orders risk-checked",
    },
    Stat {
        value: "T+1",
        label: "settlement aware",
    },
];

#[component]
pub fn BrandStats() -> impl IntoView {
    view! {
        <section class="stats">
            {STATS
                .iter()
                .map(|s| {
                    view! {
                        <div class="stat">
                            <span class="stat__value">{s.value}</span>
                            <span class="stat__label">{s.label}</span>
                        </div>
                    }
                })
                .collect_view()}
        </section>
    }
}
