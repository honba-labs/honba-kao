//! HfToolbar — toolbar/layout primitive for action rows and filter bars.
//!
//! Mirrors bfx-hf-ui-core `.hfui-toolbar` and provides a horizontal flex
//! container for grouping buttons, inputs, and other primitives into a
//! consistent header/filter bar layout.

use leptos::prelude::*;

/// Toolbar variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HfToolbarVariant {
    Default,
    Condensed,
}

impl HfToolbarVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            HfToolbarVariant::Default => "default",
            HfToolbarVariant::Condensed => "condensed",
        }
    }
}

/// HfToolbar — horizontal layout toolbar.
#[component]
pub fn HfToolbar(
    #[prop(optional)] variant: Option<HfToolbarVariant>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] direction: Option<&'static str>, // "row" | "column"
    children: Children,
) -> impl IntoView {
    let variant = variant.unwrap_or(HfToolbarVariant::Default);
    let dir = direction.unwrap_or("row");
    let var_class = match variant {
        HfToolbarVariant::Default => "hf-toolbar--default",
        HfToolbarVariant::Condensed => "hf-toolbar--condensed",
    };

    let direction_class = match dir {
        "row" => "hf-toolbar--row",
        "column" => "hf-toolbar--column",
        _ => "hf-toolbar--row",
    };

    view! {
        <div
            class=format!("hf-toolbar {var_class} {direction_class} {class}").trim().to_string()
        >
            {children()}
        </div>
    }
}

/// HfToolbarSpacer — flexible space between toolbar items.
#[component]
pub fn HfToolbarSpacer(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <div class=format!("hf-toolbar-spacer {class}").trim().to_string() />
    }
}
