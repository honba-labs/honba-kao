//! `HfSegmentedControl` and `HfTraderTabGroup` — horizontal pill / tab selectors.
//! `HfSegmentSelector` is an alias for `HfSegmentedControl`.
//! Ported from `secured-finance-app` atoms `Selector` and `TraderProTab`.

use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    pub value: String,
    pub label: String,
}

/// Horizontal pill selector for binary or ternary mode switching.
#[component]
pub fn HfSegmentedControl(
    #[prop(into)] value: RwSignal<String>,
    options: Vec<Segment>,
    #[prop(into, optional)] size: Option<&'static str>,
) -> impl IntoView {
    let size = size.unwrap_or("px-2.5 py-1");
    view! {
        <div class="hf-segmented-control inline-flex items-center gap-0.5 bg-surface-subtle rounded-lg p-0.5 border border-border-soft">
            {options.into_iter().map(|opt| {
                let opt_value = opt.value.clone();
                let opt_label = opt.label.clone();
                let opt_value_selected = opt_value.clone();
                let selected = move || value.get() == opt_value_selected;
                let on_click = move |_| {
                    value.set(opt_value.clone());
                };
                view! {
                    <button
                        type="button"
                        class=move || format!(
                            "hf-segment {} {} text-xs font-medium transition-colors",
                            size,
                            if selected() {
                                "bg-emerald-600 text-white shadow-sm"
                            } else {
                                "text-text-muted hover:text-text-high hover:bg-surface-elevated"
                            }
                        )
                        on:click=on_click
                    >
                        {opt_label}
                    </button>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

/// Terminal-style Buy/Sell and Order Type switcher tabs.
#[component]
pub fn HfTraderTabGroup(
    #[prop(into)] active: RwSignal<String>,
    tabs: Vec<(String, String)>, // (value, label)
    #[prop(into, optional)] on_change: Option<Callback<String>>,
) -> impl IntoView {
    view! {
        <div class="hf-trader-tab-group grid grid-cols-2 gap-1 bg-surface-subtle p-0.5 rounded-md border border-border-soft">
            {tabs.into_iter().map(|(value, label)| {
                let value_keep = value.clone();
                let value_for_active = value_keep.clone();
                let value_for_click = value_keep.clone();
                let is_active = move || active.get() == value_for_active;
                let on_change_clone = on_change.clone();
                let on_click = move |_| {
                    active.set(value_for_click.clone());
                    if let Some(cb) = on_change_clone.as_ref() {
                        cb.run(value_for_click.clone());
                    }
                };
                view! {
                    <button
                        type="button"
                        class=move || format!(
                            "hf-trader-tab py-1.5 text-xs font-bold rounded transition-colors {}",
                            if is_active() {
                                "bg-emerald-600 text-white shadow-sm"
                            } else {
                                "text-text-muted hover:text-text-high"
                            }
                        )
                        on:click=on_click
                    >
                        {label}
                    </button>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

/// Alias for `HfSegmentedControl` (binary / ternary mode switching).
pub use HfSegmentedControl as HfSegmentSelector;
