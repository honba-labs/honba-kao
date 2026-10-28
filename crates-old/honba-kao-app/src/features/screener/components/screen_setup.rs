//! Screen setup deck — the tab bar (Library/Custom/Formula/Saved/Public),
//! the active-filter summary bar (chip list + "Edit rules"/"Multi-market"/
//! "AI brief" toggles) and the Saved/Public tab content. The three
//! expandable drawers (`QueryBar`+`QueryBuilder`, `MultiMarketScanPanel`,
//! `CustomFormulaScreener`) are composed here from their own components.
//! Milestone 3.2-3.5 of `plan-screener.md`, `screener-design.md` §3.2.

use leptos::prelude::*;

use honba_kao_ui_core::HfButton;

use crate::features::screener::components::custom_formula::CustomFormulaScreener;
use crate::features::screener::components::multi_market_scan::MultiMarketScanPanel;
use crate::features::screener::components::query_builder::{QueryBar, QueryBuilder};
use crate::features::screener::stores::filter_engine::{AdvancedFilters, count_active_filters};
use crate::features::screener::stores::screener_store::{ScreenerStore, SetupTab};

/// The tab bar order + labels.
const TABS: [(SetupTab, &str); 5] = [
    (SetupTab::Library, "Library"),
    (SetupTab::Custom, "Custom"),
    (SetupTab::Formula, "Custom Formula"),
    (SetupTab::Saved, "Saved"),
    (SetupTab::Public, "Public"),
];

/// Human-readable active-filter chip labels for the filter summary bar — a
/// readable subset of the predicates `count_active_filters` also counts.
/// Pure; unit-tested in `tests/screener_components_test.rs`.
pub fn active_filter_chips(filters: &AdvancedFilters) -> Vec<String> {
    let mut chips = Vec::new();
    let all = "all";

    if filters.sector != all && !filters.sector.is_empty() {
        chips.push(format!("Sector = {}", filters.sector));
    }
    if filters.market_cap_tier != all && !filters.market_cap_tier.is_empty() {
        chips.push(format!("Market cap = {}", filters.market_cap_tier));
    }
    if filters.exchange != all && !filters.exchange.is_empty() {
        chips.push(format!("Exchange = {}", filters.exchange));
    }
    if let Some(min) = filters.pe_min {
        chips.push(format!("P/E >= {min}"));
    }
    if let Some(max) = filters.pe_max {
        chips.push(format!("P/E <= {max}"));
    }
    if let Some(min) = filters.min_dividend_yield {
        chips.push(format!("Div yield >= {min}%"));
    }
    if let Some(min) = filters.min_roce {
        chips.push(format!("ROCE >= {min}%"));
    }
    if let Some(min) = filters.min_net_margin {
        chips.push(format!("Net margin >= {min}%"));
    }
    if filters.technical_rating != all && !filters.technical_rating.is_empty() {
        chips.push(format!("Rating = {}", filters.technical_rating));
    }
    if let Some(min) = filters.rsi_min {
        chips.push(format!("RSI >= {min}"));
    }
    if let Some(max) = filters.rsi_max {
        chips.push(format!("RSI <= {max}"));
    }
    if filters.near_52_week_high {
        chips.push("Near 52W high".to_string());
    }
    if filters.price_above_200_sma {
        chips.push("Price > 200 SMA".to_string());
    }
    if filters.price_above_50_sma {
        chips.push("Price > 50 SMA".to_string());
    }
    if filters.asset_class != all && !filters.asset_class.is_empty() {
        chips.push(format!("Asset class = {}", filters.asset_class));
    }
    if filters.issuer_type != all && !filters.issuer_type.is_empty() {
        chips.push(format!("Issuer type = {}", filters.issuer_type));
    }
    if filters.credit_rating != all && !filters.credit_rating.is_empty() {
        chips.push(format!("Credit rating = {}", filters.credit_rating));
    }
    if filters.mf_category != all && !filters.mf_category.is_empty() {
        chips.push(format!("MF category = {}", filters.mf_category));
    }
    if let Some(min) = filters.min_cagr3y {
        chips.push(format!("3Y CAGR >= {min}%"));
    }
    if let Some(min) = filters.min_cagr5y {
        chips.push(format!("5Y CAGR >= {min}%"));
    }

    chips
}

/// How many chips show inline before the "+N more" overflow chip.
const VISIBLE_CHIPS: usize = 4;

#[component]
pub fn ScreenSetup() -> impl IntoView {
    let store = ScreenerStore::take();

    let chips = Memo::new(move |_| active_filter_chips(&store.advanced_filters.get()));
    let filter_count = Memo::new(move |_| count_active_filters(&store.advanced_filters.get()));
    let has_chip_overflow = Memo::new(move |_| chips.get().len() > VISIBLE_CHIPS);

    view! {
        <div class="sc-screen-setup">
            <div class="sc-tab-bar">
                {TABS
                    .into_iter()
                    .map(|(tab, label)| {
                        let is_active = move || store.setup_tab.get() == tab;
                        view! {
                            <button
                                class="sc-tab-bar__item"
                                data-active=move || is_active().to_string()
                                on:click=move |_| {
                                    store.set_setup_tab(tab);
                                    if tab == SetupTab::Custom {
                                        store.query_open.set(true);
                                    }
                                }
                            >
                                {label}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>

            <div class="sc-filter-summary">
                <div class="sc-filter-chips">
                    <span class="sc-filter-count">
                        {move || format!("{} active filters", filter_count.get())}
                    </span>
                    {move || {
                        chips
                            .get()
                            .into_iter()
                            .take(VISIBLE_CHIPS)
                            .map(|chip| view! { <span class="sc-chip">{chip}</span> })
                            .collect_view()
                    }}
                    <Show when=move || has_chip_overflow.get()>
                        <span class="sc-chip-overflow">
                            {move || format!("+{} more", chips.get().len().saturating_sub(VISIBLE_CHIPS))}
                        </span>
                    </Show>
                </div>
                <div class="sc-filter-actions">
                    <HfButton
                        class="sc-filter-actions__btn"
                        active=Signal::derive(move || store.query_open.get())
                        on_click=Callback::new(move |_| store.toggle_query_open())
                    >
                        "Edit rules ▾"
                    </HfButton>
                    <HfButton
                        class="sc-filter-actions__btn"
                        active=Signal::derive(move || store.scan_open.get())
                        on_click=Callback::new(move |_| store.toggle_scan_open())
                    >
                        "Multi-market ▾"
                    </HfButton>
                    <HfButton
                        class="sc-filter-actions__btn"
                        active=Signal::derive(move || store.ai_open.get())
                        on_click=Callback::new(move |_| store.toggle_ai_open())
                    >
                        "AI brief ▾"
                    </HfButton>
                </div>
            </div>

            <Show when=move || store.query_open.get()>
                <div class="sc-query-container">
                    <QueryBar />
                    <Show when=move || store.setup_tab.get() == SetupTab::Custom>
                        <QueryBuilder />
                    </Show>
                </div>
            </Show>

            <Show when=move || store.scan_open.get()>
                <MultiMarketScanPanel />
            </Show>

            <Show when=move || store.setup_tab.get() == SetupTab::Formula>
                <CustomFormulaScreener />
            </Show>

            <Show when=move || store.setup_tab.get() == SetupTab::Saved>
                <SavedScreensList />
            </Show>

            <Show when=move || store.setup_tab.get() == SetupTab::Public>
                <div class="sc-public-screens sc-empty-note">
                    "No public screens available yet — community sharing is a later milestone."
                </div>
            </Show>

            <Show when=move || store.ai_open.get()>
                <div class="sc-ai-brief sc-empty-note">
                    "AI briefing on the top matches isn't wired up yet — this drawer is a placeholder."
                </div>
            </Show>
        </div>
    }
}

/// The Saved tab's content: the saved-view list with restore-and-run /
/// delete actions over `store.saved_views`.
#[component]
fn SavedScreensList() -> impl IntoView {
    let store = ScreenerStore::take();

    view! {
        <div class="sc-saved-views">
            <Show when=move || store.saved_views.get().is_empty()>
                <div class="sc-empty-note">
                    "No saved views yet — use \"Save view\" in the hero deck above."
                </div>
            </Show>
            {move || {
                store
                    .saved_views
                    .get()
                    .into_iter()
                    .map(|view| {
                        let view_for_restore = view.clone();
                        let view_id_for_delete = view.id.clone();
                        view! {
                            <div class="sc-saved-view-row">
                                <span class="sc-saved-view-row__name">{view.name.clone()}</span>
                                <div class="sc-saved-view-row__actions">
                                    <HfButton
                                        class="sc-saved-view-row__btn"
                                        on_click=Callback::new(move |_| {
                                            store.restore_view(&view_for_restore);
                                            store.run_screen();
                                        })
                                    >
                                        "Restore & run"
                                    </HfButton>
                                    <HfButton
                                        class="sc-saved-view-row__btn"
                                        on_click=Callback::new(move |_| {
                                            store.delete_saved_view(&view_id_for_delete);
                                        })
                                    >
                                        "Delete"
                                    </HfButton>
                                </div>
                            </div>
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}
