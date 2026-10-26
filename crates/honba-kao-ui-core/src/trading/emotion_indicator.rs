//! EmotionIndicator — icon + label for the market-emotion read.

use leptos::prelude::*;

use crate::icons::outline;
use crate::primitive::icons::HfIcon;

/// Dominant market emotion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Emotion {
    Confident,
    Fearful,
    Greedy,
    Neutral,
}

impl Emotion {
    fn parts(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Emotion::Confident => (
                "hf-emotion--confident",
                outline::shield_check(),
                "Confident",
            ),
            Emotion::Fearful => ("hf-emotion--fearful", outline::alert_triangle(), "Fearful"),
            Emotion::Greedy => ("hf-emotion--greedy", outline::trending_up(), "Greedy"),
            Emotion::Neutral => ("hf-emotion--neutral", outline::minus(), "Neutral"),
        }
    }
}

/// Icon plus label describing the current market emotion.
#[component]
pub fn EmotionIndicator(emotion: Emotion, #[prop(optional, into)] class: String) -> impl IntoView {
    let (tone, icon, label) = emotion.parts();
    view! {
        <span class=format!("hf-emotion {tone} {class}").trim().to_string()>
            <HfIcon path=icon size=16 />
            <span class="hf-emotion__label">{label}</span>
        </span>
    }
}
