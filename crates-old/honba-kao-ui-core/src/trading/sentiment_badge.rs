//! SentimentBadge — Bullish / Bearish / Neutral pill.

use leptos::prelude::*;

/// Market sentiment direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sentiment {
    Bullish,
    Bearish,
    Neutral,
}

impl Sentiment {
    fn parts(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Sentiment::Bullish => ("hf-sentiment-badge--bullish", "▲", "Bullish"),
            Sentiment::Bearish => ("hf-sentiment-badge--bearish", "▼", "Bearish"),
            Sentiment::Neutral => ("hf-sentiment-badge--neutral", "▬", "Neutral"),
        }
    }
}

/// Pill showing a sentiment direction with arrow glyph and label.
#[component]
pub fn SentimentBadge(
    sentiment: Sentiment,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let (tone, glyph, label) = sentiment.parts();
    view! {
        <span class=format!("hf-sentiment-badge {tone} {class}").trim().to_string()>
            <span class="hf-sentiment-badge__glyph" aria-hidden="true">{glyph}</span>
            <span class="hf-sentiment-badge__label">{label}</span>
        </span>
    }
}
