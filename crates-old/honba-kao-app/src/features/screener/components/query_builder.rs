//! Query bar + GUI query builder. `QueryBar` is the raw expression editor
//! over `store.query`, run through `store.run_custom_formula`. `QueryBuilder`
//! is a *static* row list (Field | Operator | Value | combinator) bound to a
//! handful of representative `AdvancedFilters` fields — not a general rule
//! engine (`screener-design.md` §3.2 scopes this down explicitly). Milestone
//! 3.3 of `plan-screener.md`.

use leptos::prelude::*;

use honba_kao_ui_core::{HfButton, HfButtonVariant, HfTextarea};

use crate::features::screener::stores::filter_engine::AdvancedFilters;
use crate::features::screener::stores::screener_store::ScreenerStore;

/// `""` → nothing to run yet, non-empty → ready. Pure; unit-tested in
/// `tests/screener_components_test.rs`.
pub fn query_status_label(query: &str) -> &'static str {
    if query.trim().is_empty() {
        "Empty"
    } else {
        "Ready"
    }
}

/// Parse one query-builder value cell; an empty/invalid cell clears the
/// bound field (mirrors `filters_modal.rs`'s `parse_f64`). Pure; tested.
pub fn parse_filter_value(raw: &str) -> Option<f64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse::<f64>().ok()
}

/// The GUI builder's fixed (field, operator) row contract — stable and
/// unit-tested so the rendered rows can't silently drift from it.
pub fn query_builder_rows() -> &'static [(&'static str, &'static str)] {
    &[
        ("P/E", ">="),
        ("P/E", "<="),
        ("Dividend yield %", ">="),
        ("ROCE %", ">="),
        ("Net margin %", ">="),
        ("RSI (14)", ">="),
        ("RSI (14)", "<="),
    ]
}

#[component]
pub fn QueryBar() -> impl IntoView {
    let store = ScreenerStore::take();

    view! {
        <div class="sc-query-bar">
            <HfTextarea
                class="sc-query-bar__input"
                placeholder="sector == 'Technology' && pe < 25 && roce > 15"
                rows=3
                prop:value=move || store.query.get()
                on_input=Callback::new(move |ev| {
                    let textarea = event_target::<web_sys::HtmlTextAreaElement>(&ev);
                    store.set_query(textarea.value());
                })
            />
            <div class="sc-query-bar__footer">
                <span class="sc-chip">{move || query_status_label(&store.query.get())}</span>
                <HfButton
                    class="sc-query-bar__run"
                    variant=HfButtonVariant::Green
                    on_click=Callback::new(move |_| store.run_custom_formula())
                >
                    "Run query"
                </HfButton>
            </div>
        </div>
    }
}

/// A row's bound signal + its non-capturing field setter.
type RowBinding = (Signal<Option<f64>>, fn(&mut AdvancedFilters, Option<f64>));

/// Bind one query-builder row's number input to a single `AdvancedFilters`
/// field via a non-capturing setter (mirrors `filters_modal.rs`'s
/// `number_bound`).
fn number_bound(
    store: ScreenerStore,
    set_field: fn(&mut AdvancedFilters, Option<f64>),
) -> impl Fn(web_sys::Event) + 'static {
    move |ev: web_sys::Event| {
        let input = event_target::<web_sys::HtmlInputElement>(&ev);
        let value = parse_filter_value(&input.value());
        let mut filters = store.advanced_filters.get_untracked();
        set_field(&mut filters, value);
        store.set_advanced_filters(filters);
    }
}

/// One builder row: Field label, Operator, Value input, and (unless last) an
/// "AND" combinator badge.
fn query_builder_row(
    field: &'static str,
    operator: &'static str,
    value: Signal<Option<f64>>,
    on_input: impl Fn(web_sys::Event) + 'static,
    show_combinator: bool,
) -> impl IntoView {
    view! {
        <div class="sc-qb-row">
            <span class="sc-qb-row__field">{field}</span>
            <span class="sc-qb-row__op">{operator}</span>
            <input
                class="sc-qb-row__value"
                type="number"
                step="any"
                prop:value=move || value.get().map(|v| v.to_string()).unwrap_or_default()
                on:input=on_input
            />
            <Show when=move || show_combinator>
                <span class="sc-qb-combinator">"AND"</span>
            </Show>
        </div>
    }
}

#[component]
pub fn QueryBuilder() -> impl IntoView {
    let store = ScreenerStore::take();
    let rows = query_builder_rows();

    view! {
        <div class="sc-query-builder">
            {rows
                .iter()
                .enumerate()
                .map(|(idx, (field, operator))| {
                    let field = *field;
                    let operator = *operator;
                    let (value, setter): RowBinding = match (field, operator) {
                        ("P/E", ">=") => {
                            (Signal::derive(move || store.advanced_filters.get().pe_min), |f, v| f.pe_min = v)
                        }
                        ("P/E", "<=") => {
                            (Signal::derive(move || store.advanced_filters.get().pe_max), |f, v| f.pe_max = v)
                        }
                        ("Dividend yield %", ">=") => {
                            (
                                Signal::derive(move || store.advanced_filters.get().min_dividend_yield),
                                |f, v| f.min_dividend_yield = v,
                            )
                        }
                        ("ROCE %", ">=") => {
                            (
                                Signal::derive(move || store.advanced_filters.get().min_roce),
                                |f, v| f.min_roce = v,
                            )
                        }
                        ("Net margin %", ">=") => {
                            (
                                Signal::derive(move || store.advanced_filters.get().min_net_margin),
                                |f, v| f.min_net_margin = v,
                            )
                        }
                        ("RSI (14)", ">=") => {
                            (
                                Signal::derive(move || store.advanced_filters.get().rsi_min),
                                |f, v| f.rsi_min = v,
                            )
                        }
                        ("RSI (14)", "<=") => {
                            (
                                Signal::derive(move || store.advanced_filters.get().rsi_max),
                                |f, v| f.rsi_max = v,
                            )
                        }
                        _ => (Signal::derive(|| None), |_, _| {}),
                    };
                    let show_combinator = idx + 1 < rows.len();
                    query_builder_row(field, operator, value, number_bound(store, setter), show_combinator)
                })
                .collect_view()}
        </div>
    }
}
