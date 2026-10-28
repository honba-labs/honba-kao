//! 52-week range bar: a track from a low to a high with the last price marked.

use leptos::prelude::*;

/// Position of `last` within `[low, high]` as a percentage clamped to 0..=100.
/// A degenerate range (`high <= low`) pins to the middle.
pub fn range_position_pct(low: f64, high: f64, last: f64) -> f64 {
    if high <= low {
        return 50.0;
    }
    (((last - low) / (high - low)) * 100.0).clamp(0.0, 100.0)
}

#[component]
pub fn HfMiniRangeBar(
    /// Low label, e.g. "₹2,220.00". Pre-formatted by the caller.
    low_label: String,
    /// High label, e.g. "₹3,024.90".
    high_label: String,
    low: f64,
    high: f64,
    last: f64,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let pct = range_position_pct(low, high, last);
    view! {
        <div class=format!("hf-range-bar {class}").trim().to_string() role="meter" aria-label="52 week range" aria-valuemin=low aria-valuemax=high aria-valuenow=last>
            <span class="hf-range-bar__label">{low_label}</span>
            <div class="hf-range-bar__track">
                <div class="hf-range-bar__marker" style=format!("left: {pct:.1}%")></div>
            </div>
            <span class="hf-range-bar__label">{high_label}</span>
        </div>
    }
}
