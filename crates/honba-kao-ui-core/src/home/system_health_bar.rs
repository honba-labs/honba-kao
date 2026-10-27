//! SystemHealthBar — Multi-pill feed health, WebSocket status, and latency bar.
//!
//! Models OpenTerminalUI's `SystemHealthBar.tsx`. Displays pulsing tone indicators
//! for live market connectivity, OMS sync and data feed health.

use leptos::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemHealthTone {
    Ok,
    Warning,
    Critical,
    Offline,
    Stale,
    Info,
    Neutral,
}

impl SystemHealthTone {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warning => "warning",
            Self::Critical => "critical",
            Self::Offline => "offline",
            Self::Stale => "stale",
            Self::Info => "info",
            Self::Neutral => "neutral",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemHealthItem {
    pub id: String,
    pub label: String,
    pub value: String,
    pub tone: SystemHealthTone,
}

impl SystemHealthItem {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        value: impl Into<String>,
        tone: SystemHealthTone,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value: value.into(),
            tone,
        }
    }
}

#[component]
pub fn SystemHealthBar(
    items: Vec<SystemHealthItem>,
    #[prop(optional, into)] class: String,
    #[prop(optional, into, default = "System health indicators".to_string())] aria_label: String,
) -> impl IntoView {
    let bar_class = format!("ot-home-widget-status-bar {class}")
        .trim()
        .to_string();

    view! {
        <section class=bar_class aria-label=aria_label>
            <ul class="ot-home-widget-status-list">
                {items.into_iter().map(|item| {
                    let tone_str = item.tone.as_str();
                    view! {
                        <li class="ot-home-widget-status-item" data-tone=tone_str>
                            <span class="ot-home-widget-status-dot" aria-hidden="true"></span>
                            <span class="ot-home-widget-status-copy">
                                <span class="ot-home-widget-status-label">{item.label}</span>
                                <span class="ot-home-widget-status-value">{item.value}</span>
                            </span>
                        </li>
                    }
                }).collect_view()}
            </ul>
        </section>
    }
}
