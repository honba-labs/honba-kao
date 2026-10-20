//! HfFilterBar — segmented pills, HfSelect, HfSwitch, search.
//! Mirrors bfx-hf-ui-core filter bar used in screener and workbench.

use leptos::prelude::*;

#[component]
pub fn HfFilterBar(
    children: Children,
) -> impl IntoView {
    view! {
        <div class="hf-filter-bar">
            {children()}
        </div>
    }
}

/// HfFilterBarSection — a group of related filters within the filter bar.
#[component]
pub fn HfFilterBarSection(
    #[prop(optional, into)]
    class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!("hf-filter-bar__section {class}").trim().to_string()>
            {children()}
        </div>
    }
}

/// HfFilterBarActions — action buttons at the end of the filter bar.
#[component]
pub fn HfFilterBarActions(
    #[prop(optional, into)]
    class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!("hf-filter-bar__actions {class}").trim().to_string()>
            {children()}
        </div>
    }
}