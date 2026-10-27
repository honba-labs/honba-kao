//! ScoreBadge — composite factor score badge (0-100), tone-coded by band.

use leptos::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScoreTone {
    High,
    Mid,
    Low,
}

impl ScoreTone {
    pub fn as_class(&self) -> &'static str {
        match self {
            ScoreTone::High => "badge-high",
            ScoreTone::Mid => "badge-mid",
            ScoreTone::Low => "badge-low",
        }
    }
}

/// Classify a composite factor score (0-100) into a display tone.
///
/// `>= 75.0` -> High, `>= 50.0` -> Mid, else Low. NaN is treated as Low.
pub fn score_tone(score: f64) -> ScoreTone {
    if score.is_nan() {
        return ScoreTone::Low;
    }
    if score >= 75.0 {
        ScoreTone::High
    } else if score >= 50.0 {
        ScoreTone::Mid
    } else {
        ScoreTone::Low
    }
}

#[component]
pub fn ScoreBadge(score: f64, #[prop(optional, into)] class: String) -> impl IntoView {
    let tone = score_tone(score);
    let display = if score.is_nan() {
        0
    } else {
        score.clamp(0.0, 100.0).round() as i64
    };

    view! {
        <span class=format!("sc-score-badge {} {class}", tone.as_class())
            .trim()
            .to_string()>
            {display}
        </span>
    }
}
