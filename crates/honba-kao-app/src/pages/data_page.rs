//! Market data page. Mirrors `web/src/pages/DataPage.tsx`.

use leptos::prelude::*;

use crate::api;
use crate::components::ui::{HfPanel, HfPanelHeader, PageHeader};
use crate::hooks::use_interval;
use crate::utils::{currency, percent};

#[component]
pub fn DataPage() -> impl IntoView {
    let tick = use_interval(10_000);
    let feeds = api::data::get_feed_health();
    let quotes = api::data::get_market_quotes();
    let feed_count = feeds.len();
    let quote_count = quotes.len();

    view! {
        <div class="console">
            <PageHeader title="Market data" subtitle="Feeds, freshness and instrument snapshots">
                <span class="muted small">{move || format!("tick {}", tick.get())}</span>
            </PageHeader>

            <HfPanel>
                <HfPanelHeader title="Feed health" counter=feed_count />
                <div class="hfui-panel__content flush">
                    <table class="hfui-table">
                        <thead>
                            <tr>
                                <th>"Provider"</th>
                                <th>"Segment"</th>
                                <th>"Latency"</th>
                                <th>"Status"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {feeds
                                .into_iter()
                                .map(|feed| {
                                    let healthy = feed.healthy;
                                    view! {
                                        <tr>
                                            <td><strong>{feed.provider}</strong></td>
                                            <td class="muted small">{feed.segment}</td>
                                            <td class="tnum">{format!("{} ms", feed.latency_ms)}</td>
                                            <td>
                                                <span class=if healthy {
                                                    "status-chip ok"
                                                } else {
                                                    "status-chip warn"
                                                }>{if healthy { "HEALTHY" } else { "DEGRADED" }}</span>
                                            </td>
                                        </tr>
                                    }
                                })
                                .collect_view()}
                        </tbody>
                    </table>
                </div>
            </HfPanel>

            <HfPanel>
                <HfPanelHeader title="Watchlist snapshot" counter=quote_count />
                <div class="hfui-panel__content flush">
                    <table class="hfui-table">
                        <thead>
                            <tr>
                                <th>"Symbol"</th>
                                <th>"Name"</th>
                                <th>"Last"</th>
                                <th>"Change"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {quotes
                                .into_iter()
                                .map(|quote| {
                                    let up = quote.change_pct >= 0.0;
                                    view! {
                                        <tr>
                                            <td><strong class="sym">{quote.symbol}</strong></td>
                                            <td class="muted small">{quote.name}</td>
                                            <td class="tnum">{currency(quote.last)}</td>
                                            <td>
                                                <span class=if up { "pnl-up" } else { "pnl-down" }>
                                                    {percent(quote.change_pct)}
                                                </span>
                                            </td>
                                        </tr>
                                    }
                                })
                                .collect_view()}
                        </tbody>
                    </table>
                </div>
            </HfPanel>
        </div>
    }
}
