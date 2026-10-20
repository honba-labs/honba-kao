//! HfBadge — run-state badge (RUNNING / PAUSED / STOPPED), modeled after
//! bfx-hf-ui-core `.hfui-badge`.

use leptos::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HfTraderState {
    Running,
    Paused,
    Stopped,
}

impl HfTraderState {
    pub fn as_class(self) -> &'static str {
        match self {
            HfTraderState::Running => "running",
            HfTraderState::Paused => "paused",
            HfTraderState::Stopped => "stopped",
        }
    }

    pub fn as_label(self) -> &'static str {
        match self {
            HfTraderState::Running => "RUNNING",
            HfTraderState::Paused => "PAUSED",
            HfTraderState::Stopped => "STOPPED",
        }
    }
}

#[component]
pub fn HfBadge(state: HfTraderState) -> impl IntoView {
    view! {
        <span class=format!("hfui-badge {}", state.as_class())>
            {state.as_label()}
        </span>
    }
}
