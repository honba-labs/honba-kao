//! IntelWire — streaming headlines with sentiment chips.
//!
//! Extraction of the Intel Wire section inline in OpenTerminalUI's
//! `HomePage.tsx` (plan-1.md §3 2.4): the top 5–10 breaking headlines with
//! source, timestamp and sentiment chips (`Bullish` / `Bearish` / `Neutral`
//! with confidence %).

use leptos::prelude::*;

use honba_kao_ui_core::{HfButton, HfButtonSize, HfButtonVariant};

/// Sentiment carried through from the news API payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntelSentiment {
    Bullish,
    Bearish,
    Neutral,
}

impl IntelSentiment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bullish => "bullish",
            Self::Bearish => "bearish",
            Self::Neutral => "neutral",
        }
    }

    /// Display label, e.g. `"Bullish"`.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Bullish => "Bullish",
            Self::Bearish => "Bearish",
            Self::Neutral => "Neutral",
        }
    }

    /// Tolerant parse for API labels like `"Bullish"` / `"NEGATIVE"`.
    pub fn from_label(label: &str) -> Option<Self> {
        let normalized = label.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "bullish" | "bull" | "buy" | "positive" => Some(Self::Bullish),
            "bearish" | "bear" | "sell" | "negative" => Some(Self::Bearish),
            "neutral" => Some(Self::Neutral),
            _ => None,
        }
    }
}

/// A sentiment classification with a 0..1 confidence score.
#[derive(Debug, Clone, PartialEq)]
pub struct IntelSentimentScore {
    pub label: IntelSentiment,
    pub confidence: f64,
}

impl IntelSentimentScore {
    pub fn new(label: IntelSentiment, confidence: f64) -> Self {
        Self { label, confidence }
    }

    /// `0.87` -> `"87%"`, rounded; non-finite scores render `"--"`.
    pub fn confidence_label(&self) -> String {
        format_confidence(self.confidence)
    }
}

/// One headline on the wire.
#[derive(Debug, Clone, PartialEq)]
pub struct IntelWireItem {
    pub id: String,
    pub source: String,
    /// RFC 3339 publish timestamp, when the payload carries one.
    pub published_at: Option<String>,
    pub title: String,
    pub url: Option<String>,
    pub sentiment: Option<IntelSentimentScore>,
}

impl IntelWireItem {
    pub fn new(id: impl Into<String>, source: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            published_at: None,
            title: title.into(),
            url: None,
            sentiment: None,
        }
    }

    pub fn with_published_at(mut self, published_at: impl Into<String>) -> Self {
        self.published_at = Some(published_at.into());
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn with_sentiment(mut self, sentiment: IntelSentimentScore) -> Self {
        self.sentiment = Some(sentiment);
        self
    }
}

/// `0.87` -> `"87%"`; non-finite scores render `"--"`.
pub fn format_confidence(confidence: f64) -> String {
    if !confidence.is_finite() {
        return "--".to_string();
    }
    format!("{}%", (confidence * 100.0).round() as i64)
}

/// `2026-01-15T09:05:00Z` -> `"09:05"` (24h). `None` renders empty;
/// unparseable timestamps render verbatim.
pub fn format_intel_time(published_at: Option<&str>) -> String {
    let Some(raw) = published_at else {
        return String::new();
    };
    match chrono::DateTime::parse_from_rfc3339(raw) {
        Ok(parsed) => parsed.format("%H:%M").to_string(),
        Err(_) => raw.to_string(),
    }
}

#[component]
pub fn IntelWire(
    items: Vec<IntelWireItem>,
    /// How many headlines to show (plan: top 5–10).
    #[prop(optional, default = 5)]
    limit: usize,
    /// Wired by the page to navigate to the news desk.
    #[prop(optional)]
    on_open_news: Option<Callback<web_sys::MouseEvent>>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let open_news = on_open_news.map(|cb| {
        view! {
            <HfButton
                variant=HfButtonVariant::Ghost
                size=HfButtonSize::Small
                on_click=cb
                class="ot-home-intel-open"
            >
                "Open News"
            </HfButton>
        }
    });

    let body = if items.is_empty() {
        view! {
            <div class="ot-home-intel-empty">
                <p class="ot-home-intel-empty-title">"Start the news radar"</p>
                <p class="ot-home-intel-empty-message">
                    "Create a watchlist or open the news desk so the home wire has symbols and headlines to prioritize."
                </p>
            </div>
        }
        .into_any()
    } else {
        let rows = items
            .into_iter()
            .take(limit)
            .map(|item| {
                let kicker_time = format_intel_time(item.published_at.as_deref());
                let kicker = if kicker_time.is_empty() {
                    item.source.clone()
                } else {
                    format!("{} • {}", item.source, kicker_time)
                };
                let sentiment = item.sentiment.clone().map(|score| {
                    view! {
                        <span
                            class="ot-home-intel-sentiment"
                            data-tone=score.label.as_str()
                        >
                            {score.label.label()} " " {score.confidence_label()}
                        </span>
                    }
                });
                let link = item.url.clone().map(|url| {
                    view! {
                        <a
                            class="ot-home-intel-link"
                            href=url
                            target="_blank"
                            rel="noreferrer"
                        >
                            <span class="ot-home-intel-item-body">
                                <span class="ot-home-intel-kicker">{kicker.clone()}</span>
                                <span class="ot-home-intel-headline">{item.title.clone()}</span>
                            </span>
                            {sentiment.clone()}
                        </a>
                    }
                    .into_any()
                });
                let content = link.unwrap_or_else(|| {
                    view! {
                        <span class="ot-home-intel-item-body">
                            <span class="ot-home-intel-kicker">{kicker.clone()}</span>
                            <span class="ot-home-intel-headline">{item.title.clone()}</span>
                        </span>
                        {sentiment}
                    }
                    .into_any()
                });
                view! { <li class="ot-home-intel-item">{content}</li> }
            })
            .collect_view();
        view! {
            <ol class="ot-home-intel-list" role="list" aria-label="Latest headlines">
                {rows}
            </ol>
        }
        .into_any()
    };

    let wire_class = format!("ot-home-intel-wire {class}").trim().to_string();

    view! {
        <section class=wire_class aria-label="Intel Wire">
            <header class="ot-home-intel-header">
                <div>
                    <h2 class="ot-home-intel-title">"Intel Wire"</h2>
                    <p class="ot-home-intel-subtitle">
                        "Latest headlines from the news polling loop with sentiment carried through from the API payload."
                    </p>
                </div>
                {open_news}
            </header>
            {body}
        </section>
    }
}
