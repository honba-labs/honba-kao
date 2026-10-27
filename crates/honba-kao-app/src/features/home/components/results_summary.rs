//! ResultsSummaryCards & IntelligenceTimeline — earnings scorecards and the
//! corporate events / filings feed.
//!
//! Port of OpenTerminalUI's dashboard intelligence pair (plan-1.md §3 2.5),
//! specialized per the plan: quarterly corporate earnings scorecards (QoQ /
//! YoY revenue & PAT growth) and a corporate filings / events feed.

use leptos::prelude::*;

use chrono::{Datelike, Timelike};

use super::portfolio_mini_chart::format_inr_grouping;

/// A quarterly corporate earnings scorecard.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EarningsScorecard {
    pub symbol: String,
    pub company: String,
    /// Reported quarter label, e.g. `"Q2 FY26"`.
    pub quarter: String,
    /// Revenue in ₹ crore.
    pub revenue_cr: Option<f64>,
    /// Revenue growth vs the previous quarter, in percent.
    pub revenue_qoq_pct: Option<f64>,
    /// Revenue growth vs the year-ago quarter, in percent.
    pub revenue_yoy_pct: Option<f64>,
    /// Profit after tax in ₹ crore.
    pub pat_cr: Option<f64>,
    /// PAT growth vs the previous quarter, in percent.
    pub pat_qoq_pct: Option<f64>,
    /// PAT growth vs the year-ago quarter, in percent.
    pub pat_yoy_pct: Option<f64>,
}

impl EarningsScorecard {
    pub fn new(
        symbol: impl Into<String>,
        company: impl Into<String>,
        quarter: impl Into<String>,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            company: company.into(),
            quarter: quarter.into(),
            ..Self::default()
        }
    }

    pub fn with_revenue(mut self, cr: f64, qoq_pct: f64, yoy_pct: f64) -> Self {
        self.revenue_cr = Some(cr);
        self.revenue_qoq_pct = Some(qoq_pct);
        self.revenue_yoy_pct = Some(yoy_pct);
        self
    }

    pub fn with_pat(mut self, cr: f64, qoq_pct: f64, yoy_pct: f64) -> Self {
        self.pat_cr = Some(cr);
        self.pat_qoq_pct = Some(qoq_pct);
        self.pat_yoy_pct = Some(yoy_pct);
        self
    }
}

/// Tone of a growth figure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GrowthTone {
    Up,
    #[default]
    Neutral,
    Down,
}

impl GrowthTone {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Neutral => "neutral",
            Self::Down => "down",
        }
    }
}

/// Tone for a growth percentage: positive -> up, negative -> down.
pub fn growth_tone(value: Option<f64>) -> GrowthTone {
    match value {
        Some(v) if v.is_finite() && v > 0.0 => GrowthTone::Up,
        Some(v) if v.is_finite() && v < 0.0 => GrowthTone::Down,
        _ => GrowthTone::Neutral,
    }
}

/// Formats a growth percentage like the reference `pct()` helper: fractions
/// at or below magnitude 1 are scaled by 100, the sign is always shown, and
/// missing / non-finite values render `"--"`.
pub fn format_growth_pct(value: Option<f64>) -> String {
    let Some(next) = value.filter(|v| v.is_finite()) else {
        return "--".to_string();
    };
    let scaled = if next.abs() <= 1.0 {
        next * 100.0
    } else {
        next
    };
    format!("{scaled:+.1}%")
}

/// Formats an absolute ₹ crore figure, e.g. `₹1,23,457 Cr`.
pub fn format_cr(value: Option<f64>) -> String {
    match value {
        Some(v) if v.is_finite() => format!("₹{} Cr", format_inr_grouping(v)),
        _ => "--".to_string(),
    }
}

#[component]
pub fn ResultsSummaryCards(
    scorecards: Vec<EarningsScorecard>,
    #[prop(optional, default = false)] loading: bool,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let has_scorecards = !scorecards.is_empty();
    let cards = scorecards
        .into_iter()
        .map(|card| {
            let revenue_qoq = card.revenue_qoq_pct;
            let revenue_yoy = card.revenue_yoy_pct;
            let pat_qoq = card.pat_qoq_pct;
            let pat_yoy = card.pat_yoy_pct;
            let revenue_value = format_cr(card.revenue_cr);
            let pat_value = format_cr(card.pat_cr);
            view! {
                <article class="ot-home-results-scorecard" aria-label=format!("{} earnings", card.symbol)>
                    <header class="ot-home-results-scorecard-header">
                        <div>
                            <span class="ot-home-results-symbol">{card.symbol.clone()}</span>
                            <span class="ot-home-results-company">{card.company.clone()}</span>
                        </div>
                        <span class="ot-home-results-quarter">{card.quarter.clone()}</span>
                    </header>
                    <dl class="ot-home-results-rows">
                        <div class="ot-home-results-row">
                            <dt class="ot-home-results-row-label">"Revenue"</dt>
                            <dd class="ot-home-results-row-value">
                                {revenue_value}
                                <span class="ot-home-results-growth" data-tone=growth_tone(revenue_qoq).as_str()>
                                    {format_growth_pct(revenue_qoq)} " QoQ"
                                </span>
                                <span class="ot-home-results-growth" data-tone=growth_tone(revenue_yoy).as_str()>
                                    {format_growth_pct(revenue_yoy)} " YoY"
                                </span>
                            </dd>
                        </div>
                        <div class="ot-home-results-row">
                            <dt class="ot-home-results-row-label">"PAT"</dt>
                            <dd class="ot-home-results-row-value">
                                {pat_value}
                                <span class="ot-home-results-growth" data-tone=growth_tone(pat_qoq).as_str()>
                                    {format_growth_pct(pat_qoq)} " QoQ"
                                </span>
                                <span class="ot-home-results-growth" data-tone=growth_tone(pat_yoy).as_str()>
                                    {format_growth_pct(pat_yoy)} " YoY"
                                </span>
                            </dd>
                        </div>
                    </dl>
                </article>
            }
        })
        .collect_view();

    let body = if !has_scorecards && !loading {
        view! {
            <div class="ot-home-results-empty">
                <p class="ot-home-results-empty-title">"No results reported yet"</p>
                <p class="ot-home-results-empty-message">
                    "Quarterly earnings scorecards appear here as companies report revenue and PAT."
                </p>
            </div>
        }
        .into_any()
    } else {
        view! {
            <div class="ot-home-results-grid">{cards}</div>
        }
        .into_any()
    };

    let section_class = format!("ot-home-results {class}").trim().to_string();

    view! {
        <section class=section_class aria-label="Results summary">
            <header class="ot-home-results-header">
                <div>
                    <h3 class="ot-home-results-title">"Results Summary"</h3>
                    <p class="ot-home-results-subtitle">
                        "Quarterly corporate earnings scorecards — revenue and PAT growth, QoQ and YoY."
                    </p>
                </div>
                <Show when=move || loading>
                    <span class="ot-home-results-syncing" role="status">"Syncing…"</span>
                </Show>
            </header>
            {body}
        </section>
    }
}

/// Kind of a unified intelligence timeline event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineKind {
    Filing,
    CorporateAction,
    Earnings,
    Alert,
    News,
    ModelSignal,
    BacktestRun,
}

impl TimelineKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Filing => "filing",
            Self::CorporateAction => "corporate_action",
            Self::Earnings => "earnings",
            Self::Alert => "alert",
            Self::News => "news",
            Self::ModelSignal => "model_signal",
            Self::BacktestRun => "backtest_run",
        }
    }

    /// Uppercased badge label, e.g. `"CORPORATE ACTION"`.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Filing => "FILING",
            Self::CorporateAction => "CORPORATE ACTION",
            Self::Earnings => "EARNINGS",
            Self::Alert => "ALERT",
            Self::News => "NEWS",
            Self::ModelSignal => "MODEL SIGNAL",
            Self::BacktestRun => "BACKTEST RUN",
        }
    }
}

/// Badge tone for a timeline item, mirroring the reference `toneClass`:
/// sentiment wins, then alert/corporate-action warning, else accent.
pub fn timeline_tone(kind: TimelineKind, sentiment: Option<&str>) -> &'static str {
    let label = sentiment.unwrap_or_default().to_ascii_lowercase();
    if label.contains("bear") || label.contains("sell") || label.contains("negative") {
        return "down";
    }
    if label.contains("bull") || label.contains("buy") || label.contains("positive") {
        return "up";
    }
    match kind {
        TimelineKind::Alert | TimelineKind::CorporateAction => "warning",
        _ => "accent",
    }
}

/// One corporate event / filing summary on the timeline.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineItem {
    pub id: String,
    pub kind: TimelineKind,
    pub symbol: Option<String>,
    /// RFC 3339 event timestamp, when known.
    pub timestamp: Option<String>,
    pub source: Option<String>,
    pub title: String,
    pub url: Option<String>,
    /// Free-form sentiment label from the feed, e.g. `"Bullish"`.
    pub sentiment: Option<String>,
}

impl TimelineItem {
    pub fn new(id: impl Into<String>, kind: TimelineKind, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind,
            symbol: None,
            timestamp: None,
            source: None,
            title: title.into(),
            url: None,
            sentiment: None,
        }
    }

    pub fn with_symbol(mut self, symbol: impl Into<String>) -> Self {
        self.symbol = Some(symbol.into());
        self
    }

    pub fn with_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn with_sentiment(mut self, sentiment: impl Into<String>) -> Self {
        self.sentiment = Some(sentiment.into());
        self
    }
}

/// `2026-01-15T09:05:00Z` -> `"Jan 15, 09:05"`. `None` renders `"Pending"`;
/// unparseable timestamps render verbatim.
pub fn format_timeline_time(timestamp: Option<&str>) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let Some(raw) = timestamp else {
        return "Pending".to_string();
    };
    match chrono::DateTime::parse_from_rfc3339(raw) {
        Ok(parsed) => format!(
            "{} {}, {:02}:{:02}",
            MONTHS[parsed.month0() as usize],
            parsed.day(),
            parsed.hour(),
            parsed.minute()
        ),
        Err(_) => raw.to_string(),
    }
}

#[component]
pub fn IntelligenceTimeline(
    items: Vec<TimelineItem>,
    #[prop(optional, into, default = "Intelligence Timeline".to_string())] title: String,
    #[prop(optional, default = 18)] limit: usize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let rows = items
        .into_iter()
        .take(limit)
        .map(|item| {
            let tone = timeline_tone(item.kind, item.sentiment.as_deref());
            let time = format_timeline_time(item.timestamp.as_deref());
            let symbol = item.symbol.clone().map(|symbol| {
                view! { <span class="ot-home-timeline-symbol">{symbol}</span> }
            });
            let source = item.source.clone().map(|source| {
                view! { <span class="ot-home-timeline-source">"/ " {source}</span> }
            });
            let content = view! {
                <div class="ot-home-timeline-item-body">
                    <div class="ot-home-timeline-item-meta">
                        <span class="ot-home-timeline-badge" data-tone=tone>
                            {item.kind.label()}
                        </span>
                        {symbol}
                        <span class="ot-home-timeline-time">{time}</span>
                        {source}
                    </div>
                    <p class="ot-home-timeline-headline">{item.title.clone()}</p>
                </div>
            };
            let inner = if let Some(url) = item
                .url
                .clone()
                .filter(|url| url.starts_with("http://") || url.starts_with("https://"))
            {
                view! {
                    <a class="ot-home-timeline-link" href=url target="_blank" rel="noreferrer">
                        {content}
                    </a>
                }
                .into_any()
            } else {
                content.into_any()
            };
            view! {
                <li class="ot-home-timeline-item" data-kind=item.kind.as_str()>
                    {inner}
                </li>
            }
        })
        .collect_view();

    let body = if rows.is_empty() {
        view! {
            <div class="ot-home-timeline-empty">
                <p class="ot-home-timeline-empty-title">"No timeline events"</p>
                <p class="ot-home-timeline-empty-message">
                    "Add a watchlist, create an alert, or run a model preset to seed the unified desk timeline."
                </p>
            </div>
        }
        .into_any()
    } else {
        view! {
            <ol class="ot-home-timeline-list" role="list">
                {rows}
            </ol>
        }
        .into_any()
    };

    let section_class = format!("ot-home-timeline {class}").trim().to_string();

    view! {
        <section class=section_class aria-label=title.clone()>
            <header class="ot-home-timeline-header">
                <div>
                    <h3 class="ot-home-timeline-title">{title.clone()}</h3>
                    <p class="ot-home-timeline-subtitle">
                        "Corporate events, filing summaries, alerts, earnings and model signals."
                    </p>
                </div>
            </header>
            {body}
        </section>
    }
}
