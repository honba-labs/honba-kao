//! Custom formula screener — a freeform mathematical expression editor
//! (`store.query`) plus a cheat sheet of the metric tokens it can reference,
//! derived from the same column registry the results table draws its
//! columns from. Run through `store.run_custom_formula`. Milestone 3.4 of
//! `plan-screener.md`.

use leptos::prelude::*;

use honba_kao_ui_core::{HfButton, HfButtonVariant, HfTextarea};

use crate::features::screener::stores::columns::columns_registry;
use crate::features::screener::stores::screener_store::ScreenerStore;

/// Non-numeric / categorical column ids excluded from the formula token
/// sheet — a formula operates on metrics, not labels.
const NON_FORMULA_TOKENS: &[&str] = &[
    "symbol",
    "exchange",
    "country",
    "sector",
    "technicalRating",
    "analystRating",
    "range52",
    "sparkline",
    "couponType",
    "maturityYear",
    "creditRating",
    "schemeType",
    "planType",
    "riskometer",
    "assetClass",
    "focus",
    "brand",
];

/// The formula cheat-sheet token list — every numeric metric exposed by the
/// column registry (25+, per `screener-design.md` §3.2). Pure; unit-tested
/// in `tests/screener_components_test.rs`.
pub fn formula_token_sheet() -> Vec<&'static str> {
    columns_registry()
        .iter()
        .map(|column| column.id)
        .filter(|id| !NON_FORMULA_TOKENS.contains(id))
        .collect()
}

#[component]
pub fn CustomFormulaScreener() -> impl IntoView {
    let store = ScreenerStore::take();
    let tokens = formula_token_sheet();

    view! {
        <div class="sc-custom-formula">
            <HfTextarea
                class="sc-custom-formula__input"
                placeholder="(pe < 20 && roce > 15) || dividendYield > 3"
                rows=3
                prop:value=move || store.query.get()
                on_input=Callback::new(move |ev| {
                    let textarea = event_target::<web_sys::HtmlTextAreaElement>(&ev);
                    store.set_query(textarea.value());
                })
            />

            <div class="sc-formula-cheatsheet">
                <span class="sc-formula-cheatsheet__label">
                    {format!("{} metric tokens", tokens.len())}
                </span>
                <div class="sc-formula-cheatsheet__tokens">
                    {tokens
                        .into_iter()
                        .map(|token| {
                            view! {
                                <button
                                    class="sc-formula-token"
                                    type="button"
                                    title=format!("Insert {token}")
                                    on:click=move |_| {
                                        let current = store.query.get_untracked();
                                        let next = if current.trim().is_empty() {
                                            token.to_string()
                                        } else {
                                            format!("{current} {token}")
                                        };
                                        store.set_query(next);
                                    }
                                >
                                    {token}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
            </div>

            <HfButton
                class="sc-custom-formula__run"
                variant=HfButtonVariant::Green
                on_click=Callback::new(move |_| store.run_custom_formula())
            >
                "Run formula"
            </HfButton>
        </div>
    }
}
