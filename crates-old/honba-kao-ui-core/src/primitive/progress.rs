//! HfProgress — determinate progress indicator (`role="progressbar"`).

use leptos::prelude::*;

/// Determinate progress percentage, clamped to [0, 100].
///
/// Extracted as a pure function so it can be unit-tested without rendering.
pub(crate) fn progress_pct(value: f64, max: f64) -> f64 {
    let max = max.max(f64::EPSILON);
    let value = value.clamp(0.0, max);
    (value / max) * 100.0
}

#[component]
pub fn HfProgress(
    #[prop(optional)] value: Option<f64>,
    #[prop(optional)] max: Option<f64>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let max = max.unwrap_or(100.0);
    let value = value.unwrap_or(0.0);
    let pct = progress_pct(value, max);

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

#[cfg(test)]
mod tests {
    use super::progress_pct;

    #[test]
    fn progress_pct_clamps_to_zero() {
        assert!((progress_pct(-50.0, 100.0) - 0.0).abs() < f64::EPSILON);
        assert!((progress_pct(0.0, 100.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn progress_pct_clamps_to_hundred() {
        assert!((progress_pct(150.0, 100.0) - 100.0).abs() < f64::EPSILON);
        assert!((progress_pct(100.0, 100.0) - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn progress_pct_midpoints() {
        assert!((progress_pct(50.0, 100.0) - 50.0).abs() < f64::EPSILON);
        assert!((progress_pct(25.0, 100.0) - 25.0).abs() < f64::EPSILON);
        assert!((progress_pct(0.5, 1.0) - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn progress_pct_avoids_division_by_zero() {
        // max <= 0 is floored to f64::EPSILON; any positive value saturates.
        assert!((progress_pct(7.0, 0.0) - 100.0).abs() < f64::EPSILON);
        assert!((progress_pct(0.0, 0.0) - 0.0).abs() < f64::EPSILON);
    }
}
