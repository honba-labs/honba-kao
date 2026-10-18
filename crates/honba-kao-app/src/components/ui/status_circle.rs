//! Status circle indicator matching bfx-hf-ui-core .hfui-statusbar__statuscircle.

use leptos::prelude::*;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum CircleStatus {
    #[default]
    Green,
    Yellow,
    Red,
}

impl CircleStatus {
    pub fn as_class(&self) -> &'static str {
        match self {
            CircleStatus::Green => "green",
            CircleStatus::Yellow => "yellow",
            CircleStatus::Red => "red",
        }
    }
}

#[component]
pub fn HfStatusCircle(
    #[prop(default = CircleStatus::Green)] status: CircleStatus,
) -> impl IntoView {
    view! {
        <span class=format!("hfui-statusbar__statuscircle {}", status.as_class()) />
    }
}
