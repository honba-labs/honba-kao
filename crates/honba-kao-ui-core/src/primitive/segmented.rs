//! HfSegmentedControl — a single-choice row of toggle pills (buy/sell side,
//! table/heatmap view, interval tabs).
//!
//! Headless like the other primitives: it owns selection semantics and ARIA
//! (`role="group"`, `aria-pressed` per pill) and emits the
//! `hf-segmented`/`hf-segmented__item` class contract; the look comes from
//! the caller's container class, each segment's class and `active_class`.

use leptos::prelude::*;

/// One pill of an [`HfSegmentedControl`].
#[derive(Clone, Debug, PartialEq)]
pub struct HfSegment<T> {
    /// Value reported through `on_change` when this pill is clicked.
    pub value: T,
    /// Visible label.
    pub label: String,
    /// Extra classes for this pill only (e.g. a buy/sell colour modifier).
    pub class: String,
}

impl<T> HfSegment<T> {
    pub fn new(value: T, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            class: String::new(),
        }
    }

    /// Adds per-pill classes.
    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }
}

/// HfSegmentedControl — exclusive selection over `options`.
///
/// The pill whose value equals `value` gets `aria-pressed="true"` plus the
/// `hf-segmented__item--active` and `active_class` classes. Clicking any pill
/// reports its value through `on_change`; the caller owns the state.
///
/// ```rust
/// use leptos::prelude::*;
/// use honba_kao_ui_core::{HfSegment, HfSegmentedControl};
///
/// #[derive(Clone, Copy, PartialEq)]
/// enum Side { Buy, Sell }
///
/// #[component]
/// fn SidePicker() -> impl IntoView {
///     let side = RwSignal::new(Side::Buy);
///     view! {
///         <HfSegmentedControl
///             value=side
///             on_change=Callback::new(move |next| side.set(next))
///             options=vec![HfSegment::new(Side::Buy, "Buy"), HfSegment::new(Side::Sell, "Sell")]
///             aria_label="Order side"
///         />
///     }
/// }
/// ```
#[component]
pub fn HfSegmentedControl<T>(
    /// Currently selected value.
    #[prop(into)]
    value: Signal<T>,
    /// Called with the clicked pill's value.
    on_change: Callback<T>,
    /// Pills, in display order.
    options: Vec<HfSegment<T>>,
    /// Extra classes for the container.
    #[prop(optional, into)]
    class: String,
    /// Extra classes added to the selected pill.
    #[prop(optional, into)]
    active_class: String,
    /// Accessible name for the group.
    #[prop(optional, into)]
    aria_label: String,
) -> impl IntoView
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    let container_class = format!("hf-segmented {class}").trim().to_string();
    let aria_label = (!aria_label.is_empty()).then_some(aria_label);

    let pills = options
        .into_iter()
        .map(|segment| {
            let HfSegment { value: own, label, class } = segment;
            let active_class = active_class.clone();
            let selected = {
                let own = own.clone();
                Memo::new(move |_| value.with(|current| *current == own))
            };
            let pill_class = move || {
                let mut classes = vec!["hf-segmented__item"];
                if !class.is_empty() {
                    classes.push(&class);
                }
                if selected.get() {
                    classes.push("hf-segmented__item--active");
                    if !active_class.is_empty() {
                        classes.push(&active_class);
                    }
                }
                classes.join(" ")
            };
            view! {
                <button
                    type="button"
                    class=pill_class
                    aria-pressed=move || selected.get().to_string()
                    on:click=move |_| on_change.run(own.clone())
                >
                    {label}
                </button>
            }
        })
        .collect_view();

    view! {
        <div class=container_class role="group" aria-label=aria_label>
            {pills}
        </div>
    }
}
