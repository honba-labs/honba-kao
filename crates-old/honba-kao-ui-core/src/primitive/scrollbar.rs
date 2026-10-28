//! HfScrollbar — custom token-styled scrollbar track and thumb.

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HfScrollbarOrientation {
    Vertical,
    Horizontal,
}

impl HfScrollbarOrientation {
    pub fn as_str(self) -> &'static str {
        match self {
            HfScrollbarOrientation::Vertical => "vertical",
            HfScrollbarOrientation::Horizontal => "horizontal",
        }
    }
}

#[component]
pub fn HfScrollbar(
    #[prop(optional)] orientation: Option<HfScrollbarOrientation>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let orientation = orientation.unwrap_or(HfScrollbarOrientation::Vertical);
    let orient_str = orientation.as_str();

    view! {
        <div
            role="scrollbar"
            aria-orientation=orient_str
            data-orientation=orient_str
            class=format!("hf-scrollbar hf-scrollbar--{orient_str} {class}").trim().to_string()
        >
            <div class="hf-scrollbar__thumb" />
        </div>
    }
}
