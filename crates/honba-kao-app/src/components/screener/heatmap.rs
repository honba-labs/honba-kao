//! The screener heatmap — the Leptos port of the labs app's
//! `screener-heatmap.tsx`: sector-grouped tile matrix (or one "All Markets"
//! bucket), tiles sized by market-cap tier and colored by the seven-tier
//! change% scale, with the five-step legend and a click-through to the
//! symbol detail drawer.

use leptos::prelude::*;

use crate::data::instruments::Instrument;
use crate::stores::screener::{DetailTab, ScreenerStore};
use crate::utils::format::{currency, percent};

/// The seven-tier change% color scale (labs `getHeatColor`).
fn heat_class(change_percent: f64) -> &'static str {
    if change_percent >= 3.0 {
        " sc-hm-tile--u3"
    } else if change_percent >= 1.5 {
        " sc-hm-tile--u2"
    } else if change_percent > 0.0 {
        " sc-hm-tile--u1"
    } else if change_percent == 0.0 {
        " sc-hm-tile--flat"
    } else if change_percent > -1.5 {
        " sc-hm-tile--d1"
    } else if change_percent > -3.0 {
        " sc-hm-tile--d2"
    } else {
        " sc-hm-tile--d3"
    }
}

/// Tile size from market cap (labs thresholds).
fn size_class(market_cap: f64) -> &'static str {
    if market_cap > 500_000_000_000.0 {
        " sc-hm-tile--lg"
    } else if market_cap > 100_000_000_000.0 {
        " sc-hm-tile--md"
    } else {
        " sc-hm-tile--sm"
    }
}

/// Sector-group a slice of instruments: rows keep market-cap order inside
/// each group (stable), and the groups sort by combined market cap.
fn group_rows(rows: Vec<&'static Instrument>) -> Vec<(String, Vec<&'static Instrument>)> {
    let mut buckets: Vec<(String, Vec<&'static Instrument>)> = Vec::new();

    for instrument in rows {
        let sector = if instrument.sector.is_empty() {
            "Broad Market".to_string()
        } else {
            instrument.sector.clone()
        };
        if let Some((_, bucket)) = buckets.iter_mut().find(|(name, _)| *name == sector) {
            bucket.push(instrument);
        } else {
            buckets.push((sector, vec![instrument]));
        }
    }

    // Items inside a group: market cap descending.
    for (_, bucket) in buckets.iter_mut() {
        bucket.sort_by(|a, b| {
            b.market_cap
                .partial_cmp(&a.market_cap)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    // Groups by combined market cap descending.
    buckets.sort_by(|a, b| {
        let total =
            |list: &[&'static Instrument]| -> f64 { list.iter().map(|inst| inst.market_cap).sum() };
        total(&b.1)
            .partial_cmp(&total(&a.1))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    buckets
}

#[component]
pub fn ScreenerHeatmap() -> impl IntoView {
    let store = ScreenerStore::take();
    let rows = store.sorted();
    let group_by_sector = RwSignal::new(true);

    let groups = Memo::new(move |_| {
        if group_by_sector.get() {
            group_rows(rows.get())
        } else {
            let mut grouped = rows.get();
            grouped.sort_by(|a, b| {
                b.market_cap
                    .partial_cmp(&a.market_cap)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            vec![("All Markets".to_string(), grouped)]
        }
    });

    // Legend steps (`getHeatColor` swatch + label pairs).
    let legend: [(String, &'static str); 5] = [
        ("u3".into(), "≥ +3%"),
        ("u1".into(), "0 – +3%"),
        ("flat".into(), "0%"),
        ("d1".into(), "0 – -3%"),
        ("d3".into(), "< -3%"),
    ];

    view! {
        <div class="sc-heatmap">
            // Toolbar: grouping toggle + legend.
            <div class="sc-hm-toolbar">
                <button
                    class=move || {
                        if group_by_sector.get() {
                            "wb-pill wb-pill--on sc-hm-toggle"
                        } else {
                            "wb-pill sc-hm-toggle"
                        }
                    }
                    title="Group tiles by sector"
                    on:click=move |_| group_by_sector.update(|on| *on = !*on)
                >
                    {move || {
                        if group_by_sector.get() { "Grouped by sector" } else { "Single grid" }
                    }}
                </button>

                <div class="sc-hm-legend">
                    {legend
                        .iter()
                        .map(|(tier, label)| {
                            let tier = tier.clone();
                            let label = label.to_string();
                            view! {
                                <span class="sc-hm-legend__step">
                                    <span class=format!("sc-hm-legend__swatch sc-hm-legend__swatch--{tier}")></span>
                                    {label}
                                </span>
                            }
                        })
                        .collect_view()}
                </div>
            </div>

            // Tile matrix.
            <div class="sc-hm-groups">
                {move || {
                    groups.get().into_iter().map(|(sector, items)| {
                        let head = format!("{sector} · {}", items.len());
                        let views = items.into_iter().map(|inst| {
                            let symbol = inst.symbol.clone();
                            let symbol_for_click = symbol.clone();
                            let symbol_for_sel = symbol.clone();
                            let name = inst.name.clone();
                            let change = inst.change_percent;
                            let base = format!(
                                "sc-hm-tile{}{}",
                                heat_class(change),
                                size_class(inst.market_cap),
                            );
                            let title = format!(
                                "{} — {}\nPrice: {}\nChange: {}",
                                inst.symbol,
                                name,
                                currency(inst.price),
                                percent(change),
                            );
                            let short = symbol.clone();
                            view! {
                                <button
                                    class=move || {
                                        if store.active_symbol.get() == symbol_for_sel {
                                            format!("{base} sc-hm-tile--sel")
                                        } else {
                                            base.clone()
                                        }
                                    }
                                    title=title
                                    on:click=move |_| {
                                        store.open_detail(&symbol_for_click, DetailTab::Overview);
                                    }
                                >
                                    {short}
                                </button>
                            }
                        }).collect_view();

                        view! {
                            <div class="sc-hm-group">
                                <div class="sc-hm-group__head">{head}</div>
                                <div class="sc-hm-group__tiles">{views}</div>
                            </div>
                        }
                    }).collect_view()
                }}
            </div>
        </div>
    }
}
