//! MetricCard — Institutional KPI card with delta badge, details and sparkline.
//!
//! Models OpenTerminalUI's `MetricCard.tsx`. Provides a standardized card for
//! liquidation metrics, P&L, holdings count, and F&O posture.

use leptos::prelude::*;

use crate::home::sparkline_cell::SparklineCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MetricTone {
    #[default]
    Accent,
    Up,
    Down,
    Neutral,
}

impl MetricTone {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Accent => "accent",
            Self::Up => "up",
            Self::Down => "down",
            Self::Neutral => "neutral",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetricCardDelta {
    pub label: String,
    pub tone: MetricTone,
}

impl MetricCardDelta {
    pub fn new(label: impl Into<String>, tone: MetricTone) -> Self {
        Self {
            label: label.into(),
            tone,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetricCardDetail {
    pub label: String,
    pub value: String,
    pub tone: MetricTone,
}

impl MetricCardDetail {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            tone: MetricTone::Neutral,
        }
    }

    pub fn with_tone(mut self, tone: MetricTone) -> Self {
        self.tone = tone;
        self
    }
}

#[component]
pub fn MetricCard(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(optional)] delta: Option<MetricCardDelta>,
    #[prop(optional, default = MetricTone::Accent)] tone: MetricTone,
    #[prop(optional)] badge: Option<AnyView>,
    #[prop(optional)] details: Option<Vec<MetricCardDetail>>,
    #[prop(optional)] sparkline_points: Option<Vec<f64>>,
    #[prop(optional, into)] sparkline_aria_label: Option<String>,
    #[prop(optional)] footer: Option<AnyView>,
    #[prop(optional)] children: Option<Children>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let card_class = format!("ot-home-widget-card ot-home-widget-metric-card {class}")
        .trim()
        .to_string();

    let delta_view = delta.map(|d| {
        view! {
            <div class="ot-home-widget-delta" data-tone=d.tone.as_str()>
                {d.label}
            </div>
        }
    });

    let sparkline_view = sparkline_points.and_then(|pts| {
        if pts.len() > 1 {
            let aria_lbl = sparkline_aria_label.unwrap_or_else(|| format!("{label} sparkline"));
            Some(view! {
                <SparklineCell
                    class="ot-home-widget-card-sparkline"
                    points=pts
                    aria_label=aria_lbl
                    show_tooltip=true
                />
            })
        } else {
            None
        }
    });

    let details_view = details.and_then(|d_list| {
        if d_list.is_empty() {
            None
        } else {
            Some(view! {
                <dl class="ot-home-widget-metric-details">
                    {d_list.into_iter().map(|item| {
                        let tone_str = item.tone.as_str();
                        view! {
                            <div class="ot-home-widget-metric-detail">
                                <dt class="ot-home-widget-metric-detail-label">{item.label}</dt>
                                <dd class="ot-home-widget-metric-detail-value" data-tone=tone_str>
                                    {item.value}
                                </dd>
                            </div>
                        }
                    }).collect_view()}
                </dl>
            })
        }
    });

    let badge_view = badge.map(|b| {
        view! {
            <div class="ot-home-widget-card-badge">{b}</div>
        }
    });

    let body_view = children.map(|c| {
        view! {
            <div class="ot-home-widget-card-body">{c()}</div>
        }
    });

    let footer_view = footer.map(|f| {
        view! {
            <div class="ot-home-widget-card-footer">{f}</div>
        }
    });

    view! {
        <section class=card_class aria-label=label.clone()>
            <header class="ot-home-widget-card-header">
                <div>
                    <p class="ot-home-widget-card-kicker">{label.clone()}</p>
                    <p class="ot-home-widget-card-value" data-tone=tone.as_str()>
                        {value}
                    </p>
                </div>
                {badge_view}
            </header>

            {delta_view}
            {sparkline_view}
            {details_view}
            {body_view}
            {footer_view}
        </section>
    }
}
