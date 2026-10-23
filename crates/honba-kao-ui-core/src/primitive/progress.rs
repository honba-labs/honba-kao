//! HfProgress — determinate progress indicator (`role="progressbar"`).

use leptos::prelude::*;

#[component]
pub fn HfProgress(
    #[prop(optional)] value: Option<f64>,
    #[prop(optional)] max: Option<f64>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let max = max.unwrap_or(100.0).max(f64::EPSILON);
    let value = value.unwrap_or(0.0).clamp(0.0, max);
    let pct = (value / max) * 100.0;

    view! {
        <div
            role="progressbar"
            aria-valuemin="0"
            aria-valuemax=max.to_string()
            aria-valuenow=value.to_string()
            class=format!("hf-progress {class}").trim().to_string()
        >
            <div
                class="hf-progress__indicator"
                style=format!("width:{pct:.2}%;")
            />
        </div>
    }
}
