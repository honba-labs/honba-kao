//! IdeasRadar — algorithmic trade signal cards with trigger badges.
//!
//! Port of OpenTerminalUI's `components/ideas/IdeasRadar.tsx`, specialized per
//! plan-1.md §3 2.2: visual cards for algorithmic signals (Volume Shockers,
//! 52W Breakouts, Mean Reversion) with a badge tag indicating the trigger
//! criteria (e.g. `+3.8x vol`, `+4.2% momentum`).

use leptos::prelude::*;
use leptos_router::components::A;

use chrono::Datelike;
use honba_kao_ui_core::SvgIcon;
use honba_kao_ui_core::icons::outline::external_link;

use crate::router::paths::ROUTES;

/// Algorithmic signal families surfaced on the radar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdeaSignalCategory {
    VolumeShockers,
    Breakout52W,
    MeanReversion,
}

impl IdeaSignalCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::VolumeShockers => "volume-shockers",
            Self::Breakout52W => "breakout-52w",
            Self::MeanReversion => "mean-reversion",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::VolumeShockers => "Volume Shockers",
            Self::Breakout52W => "52W Breakouts",
            Self::MeanReversion => "Mean Reversion",
        }
    }

    /// Tolerant parse for data feeds that identify categories by label.
    pub fn from_label(label: &str) -> Option<Self> {
        let normalized = label.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "volume shockers" | "volume-shockers" => Some(Self::VolumeShockers),
            "52w breakouts" | "breakout-52w" | "52w breakout" => Some(Self::Breakout52W),
            "mean reversion" | "mean-reversion" => Some(Self::MeanReversion),
            _ => None,
        }
    }
}

/// One algorithmic signal entry on the radar.
#[derive(Debug, Clone, PartialEq)]
pub struct IdeaSignal {
    pub id: String,
    pub category: IdeaSignalCategory,
    pub symbol: Option<String>,
    pub headline: String,
    /// Trigger criteria badge text, e.g. `"+3.8x vol"` or `"+4.2% momentum"`.
    pub trigger: Option<String>,
    /// Calendar date (`YYYY-MM-DD`) the signal fired, when known.
    pub date: Option<String>,
}

impl IdeaSignal {
    pub fn new(
        id: impl Into<String>,
        category: IdeaSignalCategory,
        headline: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            category,
            symbol: None,
            headline: headline.into(),
            trigger: None,
            date: None,
        }
    }

    pub fn with_symbol(mut self, symbol: impl Into<String>) -> Self {
        self.symbol = Some(symbol.into());
        self
    }

    pub fn with_trigger(mut self, trigger: impl Into<String>) -> Self {
        self.trigger = Some(trigger.into());
        self
    }

    pub fn with_date(mut self, date: impl Into<String>) -> Self {
        self.date = Some(date.into());
        self
    }
}

/// Tone for a trigger badge, derived from its leading sign.
pub fn trigger_tone(trigger: Option<&str>) -> &'static str {
    match trigger.map(str::trim) {
        Some(t) if t.starts_with('+') => "up",
        Some(t) if t.starts_with('-') => "down",
        _ => "neutral",
    }
}

/// A signal flattened with its category label for rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct RadarEntry {
    pub category: IdeaSignalCategory,
    pub signal: IdeaSignal,
}

/// Groups signals in first-seen category order, capped at `max_categories`
/// categories with `items_per_category` entries each (reference defaults 4/2).
pub fn build_radar_entries(
    signals: Vec<IdeaSignal>,
    max_categories: usize,
    items_per_category: usize,
) -> Vec<RadarEntry> {
    let mut order: Vec<IdeaSignalCategory> = Vec::new();
    let mut grouped: Vec<RadarEntry> = Vec::new();
    let mut per_category: Vec<usize> = Vec::new();

    for signal in signals {
        if !order.contains(&signal.category) {
            order.push(signal.category);
        }
        let position = order
            .iter()
            .position(|c| *c == signal.category)
            .unwrap_or_default();
        if position >= max_categories {
            continue;
        }
        let count = per_category.get(position).copied().unwrap_or(0);
        if count >= items_per_category {
            continue;
        }
        if per_category.len() <= position {
            per_category.resize(position + 1, 0);
        }
        per_category[position] = count + 1;
        grouped.push(RadarEntry {
            category: signal.category,
            signal,
        });
    }

    grouped
}

/// Compact date rendering: `2026-01-05` -> `Jan 5`. Unparseable input is
/// returned verbatim; `None` renders empty.
pub fn format_idea_date(date: Option<&str>) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let Some(date) = date else {
        return String::new();
    };
    match chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(parsed) => format!("{} {}", MONTHS[parsed.month0() as usize], parsed.day()),
        Err(_) => date.to_string(),
    }
}

#[component]
pub fn IdeasRadar(
    signals: Vec<IdeaSignal>,
    #[prop(optional, into, default = "IN".to_string())] market: String,
    #[prop(optional, default = 4)] max_categories: usize,
    #[prop(optional, default = 2)] items_per_category: usize,
    #[prop(optional, default = false)] loading: bool,
    #[prop(optional, into)] error: Option<String>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let entries = build_radar_entries(signals, max_categories, items_per_category);

    let body = if loading {
        view! {
            <div class="ot-home-ideas-state" role="status">
                "Scanning ideas…"
            </div>
        }
        .into_any()
    } else if let Some(message) = error {
        view! {
            <div class="ot-home-ideas-state" data-tone="down" role="alert">
                {message}
            </div>
        }
        .into_any()
    } else if entries.is_empty() {
        view! {
            <div class="ot-home-ideas-state">
                "No ideas right now — check the ideas board."
            </div>
        }
        .into_any()
    } else {
        let rows = entries
            .into_iter()
            .map(|entry| {
                let category = entry.category;
                let trigger_tone_str = trigger_tone(entry.signal.trigger.as_deref());
                let trigger = entry.signal.trigger.clone().map(|trigger| {
                    view! {
                        <span
                            class="ot-home-ideas-trigger"
                            data-tone=trigger_tone_str
                            title="Trigger criteria"
                        >
                            {trigger}
                        </span>
                    }
                });
                let date = entry.signal.date.clone().map(|date| {
                    let formatted = format_idea_date(Some(&date));
                    view! { <span class="ot-home-ideas-date">{formatted}</span> }
                });
                let symbol_link = entry.signal.symbol.clone().map(|symbol| {
                    let href = format!("{}/{}", ROUTES::SECURITY_HUB, symbol);
                    view! {
                        <A href=href attr:class="ot-home-ideas-symbol">
                            {symbol}
                        </A>
                    }
                });
                view! {
                    <li class="ot-home-ideas-entry" data-category=category.as_str()>
                        <div class="ot-home-ideas-entry-top">
                            <span class="ot-home-ideas-category">{category.label()}</span>
                            {trigger}
                            {date}
                        </div>
                        <div class="ot-home-ideas-entry-main">
                            {symbol_link}
                            <span class="ot-home-ideas-headline">{entry.signal.headline}</span>
                        </div>
                    </li>
                }
            })
            .collect_view();
        view! { <ul class="ot-home-ideas-list" role="list">{rows}</ul> }.into_any()
    };

    let radar_class = format!("ot-home-ideas-radar {class}").trim().to_string();

    view! {
        <section class=radar_class aria-label="Ideas radar">
            <header class="ot-home-ideas-header">
                <div>
                    <h2 class="ot-home-ideas-title">"Ideas Radar"</h2>
                    <p class="ot-home-ideas-subtitle">
                        "Quick hits from the ideas board in the " {market} " desk."
                    </p>
                </div>
                <A href=ROUTES::HOTLISTS attr:class="ot-home-ideas-open">
                    "Open ideas"
                    <SvgIcon path=external_link() size=12 />
                </A>
            </header>
            <div class="ot-home-ideas-body">{body}</div>
        </section>
    }
}
