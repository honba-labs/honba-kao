//! SentimentChart — horizontal 0–100 sentiment gauge.

use leptos::prelude::*;

/// Clamp a sentiment score into the 0..=100 gauge range. Non-finite input
/// pins to the neutral midpoint.
pub fn sentiment_score_clamped(score: f64) -> f64 {
    if score.is_finite() {
        score.clamp(0.0, 100.0)
    } else {
        50.0
    }
}

/// Horizontal gauge: bearish on the left, bullish on the right, marker at
/// `score`.
#[component]
pub fn SentimentChart(
    /// Sentiment score, 0 (max bearish) to 100 (max bullish).
    score: f64,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let score = sentiment_score_clamped(score);
    view! {
        <div
            class=format!("hf-sentiment-chart {class}").trim().to_string()
            role="meter"
            aria-label="Sentiment score"
            aria-valuemin="0"
            aria-valuemax="100"
            aria-valuenow=format!("{score:.0}")
        >
            <div class="hf-sentiment-chart__track">
                <div class="hf-sentiment-chart__fill" style=format!("width: {score:.1}%") />
                <div class="hf-sentiment-chart__marker" style=format!("left: {score:.1}%") />
            </div>
            <div class="hf-sentiment-chart__scale">
                <span>"Bearish"</span>
                <span>{format!("{score:.0}")}</span>
                <span>"Bullish"</span>
            </div>
        </div>
    }
}
