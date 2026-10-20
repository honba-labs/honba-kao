//! The filters modal — the Leptos port of the labs app's
//! `filters-modal.tsx`: sidebar categories (Descriptive / Fundamentals and
//! Valuation / Technicals and Momentum) with the local-draft flow
//! (`countActive` live badge, Reset All / Close / Apply), plus the data
//! sections the labs app renders per asset class.
//!
//! All edits target a local `AdvancedFilters` draft; `Apply` publishes via
//! `store.set_advanced_filters` (which also resets pagination).

use leptos::prelude::*;

use honba_kao_ui_core::icons::outline::x_close;
use honba_kao_ui_core::{HfButton, HfIconButton};

use crate::data::indices::{IndexItem, indices_for_market};
use crate::stores::filter_engine::{AdvancedFilters, count_active_filters};
use crate::stores::screener_store::ScreenerStore;
use crate::utils::percent;

/// Sidebar sections in the labs order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Section {
    Descriptive,
    Fundamentals,
    Technicals,
}

impl Section {
    const LIST: [Section; 3] = [
        Section::Descriptive,
        Section::Fundamentals,
        Section::Technicals,
    ];

    fn label(self) -> &'static str {
        match self {
            Section::Descriptive => "Descriptive",
            Section::Fundamentals => "Financials & Valuation",
            Section::Technicals => "Technicals & Momentum",
        }
    }
}

/// `v → f64 | None` (labs: empty input clears the bound).
fn parse_f64(raw: String) -> Option<f64> {
    raw.trim().parse::<f64>().ok()
}

#[component]
pub fn FiltersModal() -> impl IntoView {
    let store = ScreenerStore::take();
    let draft = RwSignal::new(AdvancedFilters::default_filters());
    let active_section = RwSignal::new(Section::Descriptive);

    // Re-seed the draft on every open.
    Effect::new(move |_| {
        if store.filter_modal.get() {
            draft.set(store.advanced_filters.get_untracked());
            active_section.set(Section::Descriptive);
        }
    });

    let sectors = Memo::new(move |_| {
        let mut names = store
            .universe()
            .iter()
            .map(|inst| inst.sector.clone())
            .filter(|name| !name.is_empty())
            .collect::<Vec<String>>();
        names.sort();
        names.dedup();
        names
    });

    let market_indices: Memo<Vec<&'static IndexItem>> =
        Memo::new(move |_| indices_for_market(store.market.get_untracked()));

    let reset_all = move |_| {
        draft.set(AdvancedFilters::default_filters());
    };

    let apply = move |_| {
        store.set_advanced_filters(draft.get_untracked());
        store.filter_modal.set(false);
    };

    let active_count = Memo::new(move |_| count_active_filters(&draft.get()));

    // Number-input binding helper for one `Option<f64>` field.`RwSignal` is
    // Copy + 'static, so a plain fn keeps the returned closure `'static`.
    fn number_bound(
        draft: RwSignal<AdvancedFilters>,
        set_field: fn(&mut AdvancedFilters, Option<f64>),
    ) -> impl Fn(leptos::ev::Event) + 'static {
        move |ev: leptos::ev::Event| {
            let input = event_target::<web_sys::HtmlInputElement>(&ev);
            let value = parse_f64(input.value());
            draft.update(|filters| set_field(filters, value));
        }
    }

    // Checkbox binding helper for one bool field.
    fn flag_bound(
        draft: RwSignal<AdvancedFilters>,
        set_field: fn(&mut AdvancedFilters, bool),
    ) -> impl Fn(leptos::ev::Event) + 'static {
        move |ev: leptos::ev::Event| {
            let checked = event_target::<web_sys::HtmlInputElement>(&ev).checked();
            draft.update(|filters| set_field(filters, checked));
        }
    }

    fn num_input(
        id: &'static str,
        placeholder: &'static str,
        value: Option<f64>,
        on_input: impl Fn(leptos::ev::Event) + 'static,
    ) -> impl IntoView {
        view! {
            <input
                id=id
                class="sc-filters__num"
                type="number"
                step="any"
                placeholder=placeholder
                prop:value=move || value.map(|v| v.to_string()).unwrap_or_default()
                on:input=on_input
            />
        }
    }

    view! {
        <Show when=move || store.filter_modal.get()>
            <div class="hfui-modal__backdrop" on:click=move |_| store.filter_modal.set(false)>
                <div class="hfui-modal__container sc-filters" on:click=move |event| event.stop_propagation()>

                    <div class="hfui-modal__header">
                        <div class="sc-filters__title">
                            <span>"Filters"</span>
                            <span class="sc-filters__hint">
                                {move || format!("{} filters active", active_count.get())}
                            </span>
                        </div>
                        <HfIconButton
                            icon=x_close()
                            size=14
                            unstyled=true
                            class="hfui-panel__icon-button"
                            title="Close"
                            on_click=Callback::new(move |_| store.filter_modal.set(false))
                        />
                    </div>

                    <div class="hfui-modal__body sc-filters__body">
                        // Sidebar sections
                        <div class="sc-filters__sections">
                            {Section::LIST
                                .iter()
                                .map(|section| {
                                    let section = *section;
                                    let active = move || active_section.get() == section;
                                    view! {
                                        <button
                                            class=move || {
                                                if active() {
                                                    "sc-filters__section sc-filters__section--on"
                                                } else {
                                                    "sc-filters__section"
                                                }
                                            }
                                            on:click=move |_| active_section.set(section)
                                        >
                                            {section.label()}
                                        </button>
                                    }
                                })
                                .collect_view()}
                        </div>

                        // Section panes
                        <div class="sc-filters__pane">
                            <Show when=move || active_section.get() == Section::Descriptive>
                                <div class="sc-filters__grid">
                                    <label class="sc-filters__field" for="sc-f-sector">
                                        "Sector"
                                        <select
                                            id="sc-f-sector"
                                            prop:value=move || draft.get().sector.clone()
                                            on:change=move |ev| {
                                                let select = event_target::<web_sys::HtmlSelectElement>(&ev);
                                                let value = select.value();
                                                draft.update(|filters| filters.sector = value);
                                            }
                                        >
                                            <option value="all">"All sectors"</option>
                                            {move || {
                                                sectors.get().into_iter().map(|name| {
                                                    let name_cmp = name.clone();
                                                    let name_val = name.clone();
                                                    let selected = move || draft.get().sector == name_cmp;
                                                    view! {
                                                        <option value=name_val prop:selected=selected>
                                                            {name}
                                                        </option>
                                                    }
                                                }).collect_view()
                                            }}
                                        </select>
                                    </label>

                                    <label class="sc-filters__field" for="sc-f-cap">
                                        "Market cap tier"
                                        <select
                                            id="sc-f-cap"
                                            prop:value=move || draft.get().market_cap_tier.clone()
                                            on:change=move |ev| {
                                                let select = event_target::<web_sys::HtmlSelectElement>(&ev);
                                                let value = select.value();
                                                draft.update(|filters| filters.market_cap_tier = value);
                                            }
                                        >
                                            <option value="all">"All tiers"</option>
                                            <option value="mega">"Mega (20L Cr+)"</option>
                                            <option value="large">"Large (5L Cr+)"</option>
                                            <option value="mid">"Mid (1-5L Cr)"</option>
                                            <option value="small">"Small"</option>
                                        </select>
                                    </label>

                                    <label class="sc-filters__field" for="sc-f-exchange">
                                        "Exchange"
                                        <select
                                            id="sc-f-exchange"
                                            prop:value=move || draft.get().exchange.clone()
                                            on:change=move |ev| {
                                                let select = event_target::<web_sys::HtmlSelectElement>(&ev);
                                                let value = select.value();
                                                draft.update(|filters| filters.exchange = value);
                                            }
                                        >
                                            <option value="all">"All exchanges"</option>
                                            <option value="NSE">"NSE"</option>
                                            <option value="BSE">"BSE"</option>
                                        </select>
                                    </label>

                                    // Indices multi-select chips
                                    <div class="sc-filters__field sc-filters__field--wide">
                                        "Indices"
                                        <div class="sc-filters__chips">
                                            {market_indices.get().iter().map(|index| {
                                                let code = index.code.to_string();
                                                let code_toggle = code.clone();
                                                let on = move || draft.get().indices.contains(&code);
                                                view! {
                                                    <button
                                                        class=move || {
                                                            if on() {
                                                                "sc-filters__chip sc-filters__chip--on"
                                                            } else {
                                                                "sc-filters__chip"
                                                            }
                                                        }
                                                        title=index.name
                                                        on:click=move |_| {
                                                            draft.update(|filters| {
                                                                if filters.indices.iter().any(|entry| entry == &code_toggle) {
                                                                    filters.indices.retain(|entry| entry != &code_toggle);
                                                                } else {
                                                                    filters.indices.push(code_toggle.clone());
                                                                }
                                                            });
                                                        }
                                                    >
                                                        {index.code}
                                                    </button>
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>

                                    // Price & volume bounds
                                    <label class="sc-filters__field" for="sc-f-minprice">
                                        "Min price"
                                        {num_input("sc-f-minprice", "0", draft.get().min_price, number_bound(draft, |filters, value| filters.min_price = value))}
                                    </label>
                                    <label class="sc-filters__field" for="sc-f-maxprice">
                                        "Max price"
                                        {num_input("sc-f-maxprice", "0", draft.get().max_price, number_bound(draft, |filters, value| filters.max_price = value))}
                                    </label>
                                    <label class="sc-filters__field" for="sc-f-minchg">
                                        "Min chg %"
                                        {num_input("sc-f-minchg", "-5", draft.get().min_change_percent, number_bound(draft, |filters, value| filters.min_change_percent = value))}
                                    </label>
                                    <label class="sc-filters__field" for="sc-f-maxchg">
                                        "Max chg %"
                                        {num_input("sc-f-maxchg", "10", draft.get().max_change_percent, number_bound(draft, |filters, value| filters.max_change_percent = value))}
                                    </label>
                                    <label class="sc-filters__field" for="sc-f-minvol">
                                        "Min volume"
                                        {num_input("sc-f-minvol", "2000000", draft.get().min_volume, number_bound(draft, |filters, value| filters.min_volume = value))}
                                    </label>
                                </div>
                            </Show>

                            <Show when=move || active_section.get() == Section::Fundamentals>
                                <div class="sc-filters__grid">
                                    <label class="sc-filters__field" for="sc-f-pemin">
                                        "P/E min"
                                        {num_input("sc-f-pemin", "5", draft.get().pe_min, number_bound(draft, |filters, value| filters.pe_min = value))}
                                    </label>
                                    <label class="sc-filters__field" for="sc-f-pemax">
                                        "P/E max"
                                        {num_input("sc-f-pemax", "25", draft.get().pe_max, number_bound(draft, |filters, value| filters.pe_max = value))}
                                    </label>
                                    <label class="sc-filters__field" for="sc-f-div">
                                        "Min div yield %"
                                        {num_input("sc-f-div", "1.5", draft.get().min_dividend_yield, number_bound(draft, |filters, value| filters.min_dividend_yield = value))}
                                    </label>
                                    <label class="sc-filters__field" for="sc-f-roce">
                                        "Min ROCE %"
                                        {num_input("sc-f-roce", "15", draft.get().min_roce, number_bound(draft, |filters, value| filters.min_roce = value))}
                                    </label>
                                    <label class="sc-filters__field" for="sc-f-margin">
                                        "Min net margin %"
                                        {num_input("sc-f-margin", "10", draft.get().min_net_margin, number_bound(draft, |filters, value| filters.min_net_margin = value))}
                                    </label>

                                    // ETF fields (labs renders them in the same section)
                                    <label class="sc-filters__field" for="sc-f-aum">
                                        "Min AUM"
                                        {num_input("sc-f-aum", "0", draft.get().min_aum, number_bound(draft, |filters, value| filters.min_aum = value))}
                                    </label>
                                    <label class="sc-filters__field" for="sc-f-ter">
                                        "Max expense ratio %"
                                        {num_input("sc-f-ter", "0.2", draft.get().max_expense_ratio, number_bound(draft, |filters, value| filters.max_expense_ratio = value))}
                                    </label>
                                    <label class="sc-filters__field" for="sc-f-nav">
                                        "Min NAV return %"
                                        {num_input("sc-f-nav", "0", draft.get().min_nav_return, number_bound(draft, |filters, value| filters.min_nav_return = value))}
                                    </label>

                                    // Bond fields
                                    <label class="sc-filters__field" for="sc-f-ytw">
                                        "Min YTW %"
                                        {num_input("sc-f-ytw", "0", draft.get().min_ytw, number_bound(draft, |filters, value| filters.min_ytw = value))}
                                    </label>
                                    <label class="sc-filters__field" for="sc-f-coupon">
                                        "Min coupon %"
                                        {num_input("sc-f-coupon", "0", draft.get().min_coupon, number_bound(draft, |filters, value| filters.min_coupon = value))}
                                    </label>
                                    <label class="sc-filters__field" for="sc-f-rating">
                                        "Credit rating"
                                        <select
                                            id="sc-f-rating"
                                            prop:value=move || draft.get().credit_rating.clone()
                                            on:change=move |ev| {
                                                let select = event_target::<web_sys::HtmlSelectElement>(&ev);
                                                let value = select.value();
                                                draft.update(|filters| filters.credit_rating = value);
                                            }
                                        >
                                            <option value="all">"Any"</option>
                                            <option value="AAA">"AAA"</option>
                                            <option value="AA">"AA"</option>
                                            <option value="A">"A"</option>
                                        </select>
                                    </label>

                                    // MF fields
                                    <label class="sc-filters__field" for="sc-f-cagr3">
                                        "Min 3Y CAGR %"
                                        {num_input("sc-f-cagr3", "10", draft.get().min_cagr3y, number_bound(draft, |filters, value| filters.min_cagr3y = value))}
                                    </label>
                                    <label class="sc-filters__field" for="sc-f-cagr5">
                                        "Min 5Y CAGR %"
                                        {num_input("sc-f-cagr5", "12", draft.get().min_cagr5y, number_bound(draft, |filters, value| filters.min_cagr5y = value))}
                                    </label>
                                    <label class="sc-filters__field" for="sc-f-sharpe">
                                        "Min Sharpe"
                                        {num_input("sc-f-sharpe", "1", draft.get().min_sharpe_ratio, number_bound(draft, |filters, value| filters.min_sharpe_ratio = value))}
                                    </label>
                                </div>
                            </Show>

                            <Show when=move || active_section.get() == Section::Technicals>
                                <div class="sc-filters__grid">
                                    <label class="sc-filters__field" for="sc-f-trating">
                                        "Technical rating"
                                        <select
                                            id="sc-f-trating"
                                            prop:value=move || draft.get().technical_rating.clone()
                                            on:change=move |ev| {
                                                let select = event_target::<web_sys::HtmlSelectElement>(&ev);
                                                let value = select.value();
                                                draft.update(|filters| filters.technical_rating = value);
                                            }
                                        >
                                            <option value="all">"All ratings"</option>
                                            <option value="Strong Buy">"Strong Buy"</option>
                                            <option value="Buy">"Buy"</option>
                                            <option value="Neutral">"Neutral"</option>
                                            <option value="Sell">"Sell"</option>
                                            <option value="Strong Sell">"Strong Sell"</option>
                                        </select>
                                    </label>
                                    <label class="sc-filters__field" for="sc-f-rsi-min">
                                        "RSI min"
                                        {num_input("sc-f-rsi-min", "30", draft.get().rsi_min, number_bound(draft, |filters, value| filters.rsi_min = value))}
                                    </label>
                                    <label class="sc-filters__field" for="sc-f-rsi-max">
                                        "RSI max"
                                        {num_input("sc-f-rsi-max", "70", draft.get().rsi_max, number_bound(draft, |filters, value| filters.rsi_max = value))}
                                    </label>

                                    <label class="sc-filters__check">
                                        <input
                                            type="checkbox"
                                            prop:checked=move || draft.get().price_above_200_sma
                                            on:change=flag_bound(draft, |filters, value| filters.price_above_200_sma = value)
                                        />
                                        "Price above 200 SMA"
                                    </label>
                                    <label class="sc-filters__check">
                                        <input
                                            type="checkbox"
                                            prop:checked=move || draft.get().price_above_50_sma
                                            on:change=flag_bound(draft, |filters, value| filters.price_above_50_sma = value)
                                        />
                                        "Price above 50 SMA"
                                    </label>
                                    <label class="sc-filters__check">
                                        <input
                                            type="checkbox"
                                            prop:checked=move || draft.get().near_52_week_high
                                            on:change=flag_bound(draft, |filters, value| filters.near_52_week_high = value)
                                        />
                                        "Near 52-week high (within 95%)"
                                    </label>
                                </div>
                            </Show>
                        </div>
                    </div>

                    <div class="hfui-modal__footer sc-filters__footer">
                        <span class="sc-filters__count">
                            {move || format!("{} filters active", active_count.get())}
                        </span>
                        <span class="sc-filters__actions">
                            <HfButton unstyled=true class="wb-btn" on_click=Callback::new(reset_all)>
                                "Reset all"
                            </HfButton>
                            <HfButton unstyled=true class="wb-btn wb-btn--solid" on_click=Callback::new(apply)>
                                "Apply filters"
                            </HfButton>
                            <HfButton
                                unstyled=true
                                class="wb-btn"
                                on_click=Callback::new(move |_| store.filter_modal.set(false))
                            >
                                "Close"
                            </HfButton>
                        </span>
                    </div>
                </div>
            </div>
        </Show>
    }
}

/// Keep `percent`/`IndexItem` linked for future extensions (see sum of labs
/// filter-source refs).
#[allow(dead_code)]
fn _linkage_note() {
    let _ = percent;
    let _ = count_active_filters;
}
