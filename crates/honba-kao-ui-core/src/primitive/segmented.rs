//! HfSegmentedControl — segmented control / pill group component.
//!
//! Mirrors bfx-hf-ui-core `.hfui-segmented-control` and provides a group of
//! mutually-exclusive selectable pills, suitable for asset-class filtering
//! or view toggles (e.g. "IN/US", "Table/Heatmap").

use leptos::prelude::*;

/// Segmented control variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HfSegmentedControlVariant {
    Default,
   Inline,
}

impl HfSegmentedControlVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            HfSegmentedControlVariant::Default => "default",
            HfSegmentedControlVariant::Inline => "inline",
        }
    }
}

/// HfSegmentedControl — container for segmented control pills.
#[component]
pub fn HfSegmentedControl(
    #[prop(optional)]
    variant: Option<HfSegmentedControlVariant>,
    #[prop(optional, into)]
    class: String,
    children: Children,
) -> impl IntoView {
    let variant = variant.unwrap_or(HfSegmentedControlVariant::Default);
    let var_class = match variant {
        HfSegmentedControlVariant::Default => "hf-segmented-control--default",
        HfSegmentedControlVariant::Inline => "hf-segmented-control--inline",
    };

    view! {
        <div
            class=format!("hf-segmented-control {var_class} {class}").trim().to_string()
        >
            {children()}
        </div>
    }
}

/// HfSegmentedControlItem — a single pill in a segmented control.
#[component]
pub fn HfSegmentedControlItem(
    #[prop(optional, into)]
    class: String,
    #[prop(optional)]
    selected: Option<bool>,
    #[prop(optional)]
    disabled: Option<bool>,
    children: Children,
) -> impl IntoView {
    let selected = selected.unwrap_or(false);
    let disabled = disabled.unwrap_or(false);

    let item_class = format!(
        "hf-segmented-control__item {} {}",
        if selected { "hf-segmented-control__item--selected" } else { "" },
        class
    )
    .trim()
    .to_string();

    view! {
        <div
            class=item_class
            disabled=disabled
            role="button"
            tabindex=if disabled { "-1" } else { "0" }
            aria-disabled=disabled
            on:click=move |_| {
                // select this item; parent HfSegmentedControl
                // would typically handle the state, but we fire
                // an event for the parent to react to.
                // In a full impl, the parent would have an RwSignal
                // and update the selected state.
            }
        >
            {children()}
        </div>
    }
}
