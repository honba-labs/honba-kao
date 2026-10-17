//! Badges and markers.

use leptos::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TraderState {
    Running,
    Paused,
    Stopped,
}

impl TraderState {
    pub fn as_class(&self) -> &'static str {
        match self {
            TraderState::Running => "running",
            TraderState::Paused => "paused",
            TraderState::Stopped => "stopped",
        }
    }

    pub fn as_label(&self) -> &'static str {
        match self {
            TraderState::Running => "RUNNING",
            TraderState::Paused => "PAUSED",
            TraderState::Stopped => "STOPPED",
        }
    }
}

#[component]
pub fn HfBadge(state: TraderState) -> impl IntoView {
    view! {
        <span class=format!("hfui-badge {}", state.as_class())>
            {state.as_label()}
        </span>
    }
}

#[component]
pub fn HfRowMarker(#[prop(default = true)] is_buy: bool) -> impl IntoView {
    let class = if is_buy { "green" } else { "red" };
    view! {
        <span class=format!("row-marker {}", class) />
    }
}
