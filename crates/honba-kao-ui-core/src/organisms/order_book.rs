//! L2 depth ladder organism — ported from secured-finance `NewOrderBookWidget`.
//!
//! All price/size/money values are `Decimal` (zero-floats invariant, issue #33).

use leptos::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OrderBookFilter {
    #[default]
    All,
    BidsOnly,
    AsksOnly,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BookLevel {
    pub price: Decimal,
    pub size: Decimal,
    pub cumulative_size: Decimal,
    pub order_count: u32,
}

#[component]
pub fn HfOrderBook(
    #[prop(into)] bids: Signal<Vec<BookLevel>>,
    #[prop(into)] asks: Signal<Vec<BookLevel>>,
    #[prop(into)] spread: Signal<Option<(Decimal, Decimal)>>, // (absolute, percentage)
    #[prop(into)] on_level_click: Callback<(Decimal, Decimal, bool)>, // (price, size, is_bid)
    #[prop(into, optional)] filter: Option<Signal<OrderBookFilter>>,
    #[prop(into, optional)] aggregation_options: Option<Vec<f64>>,
    #[prop(into, optional)] active_aggregation: Option<RwSignal<f64>>,
) -> impl IntoView {
    let filter = filter.unwrap_or_else(|| Signal::derive(|| OrderBookFilter::default()));
    let agg_opts = aggregation_options.unwrap_or_else(|| vec![0.05, 0.10, 0.50, 1.00]);
    let active_agg = active_aggregation.unwrap_or_else(|| RwSignal::new(0.05));

    // Aggregate levels: bucket prices within `active_agg` of each other and sum
    // their sizes, so the aggregation control actually changes the book.
    let aggregate = move |levels: Vec<BookLevel>, side_filter: OrderBookFilter| {
        let active = active_agg.get();
        let active_dec = Decimal::from_f64_retain(active).unwrap_or_default();
        let mut buckets: Vec<(Decimal, Decimal, u32)> = Vec::new(); // (price, size, order_count)
        for level in levels {
            let key = if active > 0.0 {
                (level.price / active_dec).round() * active_dec
            } else {
                level.price
            };
            if let Some(pos) = buckets
                .iter()
                .position(|(p, _, _)| (*p - key).abs() < dec!(0.000001))
            {
                buckets[pos].1 += level.size;
                buckets[pos].2 = buckets[pos].2.saturating_add(level.order_count);
            } else {
                buckets.push((key, level.size, level.order_count));
            }
        }
        let mut out: Vec<BookLevel> = buckets
            .into_iter()
            .map(|(price, size, order_count)| BookLevel {
                price,
                size,
                cumulative_size: Decimal::ZERO,
                order_count,
            })
            .collect();
        match side_filter {
            OrderBookFilter::BidsOnly | OrderBookFilter::All => {
                out.sort_by(|a, b| b.price.cmp(&a.price));
            }
            OrderBookFilter::AsksOnly => {
                out.sort_by(|a, b| a.price.cmp(&b.price));
            }
        }
        let mut run = Decimal::ZERO;
        for level in out.iter_mut() {
            run += level.size;
            level.cumulative_size = run;
        }
        out
    };

    // Max cumulative size per side, used to scale depth bars relative to the
    // widest level on that side (no magic constant).
    let max_cum = move |levels: &[BookLevel]| {
        levels
            .iter()
            .map(|l| l.cumulative_size)
            .fold(Decimal::ZERO, Decimal::max)
    };

    let asks_view = move || -> AnyView {
        let raw = asks.get();
        let levels = if filter.get() == OrderBookFilter::BidsOnly {
            Vec::new()
        } else {
            aggregate(raw, OrderBookFilter::AsksOnly)
        };
        let max = max_cum(&levels);
        view! {
            <div class="hf-book-asks flex-grow overflow-y-auto flex flex-col-reverse text-xs font-mono">
                {levels.into_iter().map(|level| {
                    let px = level.price;
                    let sz = level.size;
                    let width = if max > Decimal::ZERO {
                        (level.cumulative_size / max * dec!(100)).round()
                    } else {
                        dec!(0)
                    };
                    view! {
                        <div
                            class="hf-book-row relative flex justify-between px-2 py-0.5 hover:bg-rose-500/10 cursor-pointer"
                            on:click=move |_| on_level_click.run((px, sz, false))
                        >
                            <div
                                class="hf-depth-fill absolute inset-y-0 right-0 bg-rose-500/15"
                                style=format!("width: {}%;", width)
                            />
                            <span class="relative text-rose-400">{format!("{}", px.round_dp(2))}</span>
                            <span class="relative text-text-muted">{format!("{}", sz.round_dp(2))}</span>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        }
        .into_any()
    };

    let bids_view = move || -> AnyView {
        let raw = bids.get();
        let levels = if filter.get() == OrderBookFilter::AsksOnly {
            Vec::new()
        } else {
            aggregate(raw, OrderBookFilter::BidsOnly)
        };
        let max = max_cum(&levels);
        view! {
            <div class="hf-book-bids flex-grow overflow-y-auto flex flex-col text-xs font-mono">
                {levels.into_iter().map(|level| {
                    let px = level.price;
                    let sz = level.size;
                    let width = if max > Decimal::ZERO {
                        (level.cumulative_size / max * dec!(100)).round()
                    } else {
                        dec!(0)
                    };
                    view! {
                        <div
                            class="hf-book-row relative flex justify-between px-2 py-0.5 hover:bg-emerald-500/10 cursor-pointer"
                            on:click=move |_| on_level_click.run((px, sz, true))
                        >
                            <div
                                class="hf-depth-fill absolute inset-y-0 right-0 bg-emerald-500/15"
                                style=format!("width: {}%;", width)
                            />
                            <span class="relative text-emerald-400">{format!("{}", px.round_dp(2))}</span>
                            <span class="relative text-text-muted">{format!("{}", sz.round_dp(2))}</span>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        }
        .into_any()
    };

    view! {
        <div class="hf-order-book flex flex-col h-full bg-surface-base border border-border-soft rounded overflow-hidden">
            <div class="hf-book-header flex justify-between items-center p-2 border-b border-border-soft text-xs">
                <span class="font-semibold text-text-high">"Order Book"</span>
                <div class="hf-book-controls flex items-center gap-1">
                    <span class="text-text-muted text-[10px]">"Agg:"</span>
                    <select
                        class="bg-surface-subtle border border-border-soft rounded text-[11px] px-1 py-0.5 text-text-high outline-none"
                        prop:value=move || active_agg.get().to_string()
                        on:change=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                active_agg.set(v);
                            }
                        }
                    >
                        {agg_opts.into_iter().map(|opt| {
                            let selected = active_agg.get() == opt;
                            view! {
                                <option value=opt.to_string() selected=selected>{format!("{opt:.2}")}</option>
                            }
                        }).collect::<Vec<_>>()}
                    </select>
                </div>
            </div>

            {move || match filter.get() {
                OrderBookFilter::BidsOnly => bids_view(),
                OrderBookFilter::AsksOnly => asks_view(),
                OrderBookFilter::All => view! {
                    {asks_view()}
                    <div class="hf-book-spread py-1 px-2 bg-surface-subtle border-y border-border-soft flex justify-between text-xs text-text-muted">
                        <span>"Spread"</span>
                        <span>
                            {move || match spread.get() {
                                Some((abs, bps)) => {
                                    format!("{} ({} bps)", abs.round_dp(2), bps.round_dp(2))
                                }
                                None => "-".to_string(),
                            }}
                        </span>
                    </div>
                    {bids_view()}
                }.into_any(),
            }}
        </div>
    }
}
