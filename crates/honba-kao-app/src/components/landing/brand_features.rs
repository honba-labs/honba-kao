//! Feature grid. Mirrors `web/src/components/landing/brand/BrandFeatures.tsx`.

use leptos::prelude::*;

struct Feature {
    title: &'static str,
    body: &'static str,
    glyph: &'static str,
}

const FEATURES: &[Feature] = &[
    Feature {
        title: "Agents, not scripts",
        body: "Point a model at a mandate; it plans entries, sizing and exits within your limits.",
        glyph: "◎",
    },
    Feature {
        title: "Risk gates on every order",
        body: "Drawdown, circuit and exposure checks run before anything reaches the book.",
        glyph: "⛨",
    },
    Feature {
        title: "Full audit trail",
        body: "Every approval and rejection is logged with the reason it was decided.",
        glyph: "▤",
    },
    Feature {
        title: "Paper first",
        body: "Rehearse on live data with simulated fills before you connect a broker.",
        glyph: "◔",
    },
    Feature {
        title: "Strategy library",
        body: "Start from documented strategies, fork them, and backtest the changes.",
        glyph: "▥",
    },
    Feature {
        title: "Indian market native",
        body: "NSE and BSE sessions, circuit bands and T+1 settlement are first-class.",
        glyph: "₹",
    },
];

#[component]
pub fn BrandFeatures() -> impl IntoView {
    view! {
        <section class="features">
            <h2 class="section-title">"Built for supervised autonomy"</h2>
            <div class="features__grid">
                {FEATURES
                    .iter()
                    .map(|f| {
                        view! {
                            <article class="feature">
                                <span class="feature__glyph">{f.glyph}</span>
                                <h3 class="feature__title">{f.title}</h3>
                                <p class="feature__body">{f.body}</p>
                            </article>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}
