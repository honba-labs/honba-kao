//! Portfolio page: every linked broker in one book.
//!
//! The page owns the broker connection list; the aggregate is a memo over it,
//! so linking or unlinking an account immediately re-derives holdings, funds
//! and totals (see [`crate::data::demo_data::build_portfolio`]).
//!
//! Nothing here is broker-specific — adding a brokerage is a row in the demo
//! universe plus an OAuth handler behind [`crate::api::brokers::connect`].

use leptos::prelude::*;

use crate::api;
use crate::components::charts::EquityChart;
use honba_kao_ui_core::{HfPageHeader, HfPanel, HfPanelHeader};
use crate::types::{BrokerStatus, Holding};
use crate::utils::{compact_currency, currency, percent, quantity};

#[component]
pub fn PortfolioPage() -> impl IntoView {
    let brokers = RwSignal::new(api::brokers::list());
    let snapshot = Memo::new(move |_| api::portfolio::snapshot_for(&brokers.get()));
    // Any mutation to `brokers` re-renders; the memo keeps totals consistent.

    let toggle = move |id: &'static str, connected: bool| {
        if connected {
            let _ = api::brokers::disconnect(id);
            brokers.update(|list| {
                if let Some(broker) = list.iter_mut().find(|broker| broker.id == id) {
                    broker.status = BrokerStatus::Disconnected;
                    broker.client_id = "";
                    broker.last_sync = "—";
                }
            });
        } else if let Ok(updated) = api::brokers::connect(id) {
            brokers.update(|list| {
                if let Some(broker) = list.iter_mut().find(|broker| broker.id == id) {
                    *broker = updated;
                }
            });
        }
    };

    view! {
        <div class="console portfolio">
            <HfPageHeader
                title="Portfolio"
                subtitle="Every broker account, one aggregated book"
            >
                <span class="portfolio-sync">
                    <span class="portfolio-sync__dot" />
                    {move || {
                        format!("{} of {} brokers linked", snapshot.get().connected_brokers(), brokers.get().len())
                    }}
                </span>
            </HfPageHeader>

            <div class="portfolio-kpis">
                <div class="kpi">
                    <span class="kpi__label">"Total value"</span>
                    <span class="kpi__value tnum">
                        {move || currency(snapshot.get().summary.total_value())}
                    </span>
                    <span class="kpi__delta muted">
                        {move || {
                            let summary = snapshot.get().summary;
                            format!(
                                "{} held · {} cash",
                                compact_currency(summary.holdings_value),
                                compact_currency(summary.free_cash()),
                            )
                        }}
                    </span>
                </div>

                <div class="kpi">
                    <span class="kpi__label">"Day P&L"</span>
                    <span class="kpi__value tnum">
                        {move || pnl_text(snapshot.get().summary.day_pnl)}
                    </span>
                    <span
                        class=move || {
                            if snapshot.get().summary.day_pnl >= 0.0 {
                                "kpi__delta up"
                            } else {
                                "kpi__delta down"
                            }
                        }
                    >
                        {move || {
                            let summary = snapshot.get().summary;
                            percent(if summary.holdings_value == 0.0 {
                                0.0
                            } else {
                                summary.day_pnl / summary.holdings_value * 100.0
                            })
                        }}
                    </span>
                </div>

                <div class="kpi">
                    <span class="kpi__label">"Unrealised P&L"</span>
                    <span class="kpi__value tnum">
                        {move || pnl_text(snapshot.get().summary.unrealised_pnl)}
                    </span>
                    <span
                        class=move || {
                            if snapshot.get().summary.unrealised_pnl >= 0.0 {
                                "kpi__delta up"
                            } else {
                                "kpi__delta down"
                            }
                        }
                    >
                        {move || percent(snapshot.get().summary.unrealised_pct)}
                    </span>
                </div>

                <div class="kpi">
                    <span class="kpi__label">"Margin used"</span>
                    <span class="kpi__value tnum">
                        {move || currency(snapshot.get().summary.used_margin)}
                    </span>
                    <span class="kpi__delta muted">
                        {move || {
                            let summary = snapshot.get().summary;
                            format!(
                                "{} collateral · {} free",
                                compact_currency(summary.collateral),
                                compact_currency(summary.free_cash()),
                            )
                        }}
                    </span>
                </div>
            </div>

            <HfPanel>
                <HfPanelHeader title="Linked brokers" counter=brokers.get_untracked().len()>
                    <span class="muted small">"Connect once — holdings sync into the book"</span>
                </HfPanelHeader>

                <div class="hfui-panel__content">
                    <div class="broker-grid">
                        {move || {
                            brokers
                                .get()
                                .into_iter()
                                .map(|broker| {
                                    let connected = broker.status.is_connected();
                                    let value = snapshot.get().value_at(broker.id);
                                    let id = broker.id;
                                    view! {
                                        <article
                                            class=if connected {
                                                "broker-card broker-card--connected"
                                            } else {
                                                "broker-card"
                                            }
                                            style=format!("--tone: var({});", broker.tone)
                                        >
                                            <header class="broker-card__head">
                                                <span class="broker-card__monogram">
                                                    {broker.monogram}
                                                </span>
                                                <div class="broker-card__identity">
                                                    <span class="broker-card__name">{broker.name}</span>
                                                    <span class="broker-card__legal">{broker.legal_name}</span>
                                                </div>
                                                <span class=format!(
                                                    "broker-card__status {}",
                                                    if connected { "ok" } else { "idle" },
                                                )>{broker.status.label()}</span>
                                            </header>

                                            <div class="broker-card__segments">
                                                {broker
                                                    .segments
                                                    .iter()
                                                    .map(|segment| view! { <span>{*segment}</span> })
                                                    .collect_view()}
                                            </div>

                                            <footer class="broker-card__footer">
                                                <div class="broker-card__book">
                                                    <span class="broker-card__value tnum">
                                                        {if connected {
                                                            currency(value)
                                                        } else {
                                                            "—".to_string()
                                                        }}
                                                    </span>
                                                    <span class="broker-card__sync">
                                                        {if connected {
                                                            format!("Client {} · sync {}", broker.client_id, broker.last_sync)
                                                        } else {
                                                            "No credentials on file".to_string()
                                                        }}
                                                    </span>
                                                </div>
                                                <button
                                                    class=if connected {
                                                        "hfui-button ghost"
                                                    } else {
                                                        "hfui-button green"
                                                    }
                                                    on:click=move |_| toggle(id, connected)
                                                >
                                                    {if connected { "Disconnect" } else { "Connect" }}
                                                </button>
                                            </footer>
                                        </article>
                                    }
                                })
                                .collect_view()
                        }}
                    </div>
                </div>
            </HfPanel>

            <div class="portfolio-split">
                <HfPanel class="portfolio-holdings">
                    <HfPanelHeader title="Aggregated holdings">
                        <span class="muted small">"sorted by value"</span>
                    </HfPanelHeader>

                    <div class="hfui-panel__content flush">
                        <table class="hfui-table portfolio-table">
                            <thead>
                                <tr>
                                    <th>"Instrument"</th>
                                    <th>"Broker"</th>
                                    <th class="num">"Qty"</th>
                                    <th class="num">"Avg"</th>
                                    <th class="num">"Last"</th>
                                    <th class="num">"Day"</th>
                                    <th class="num">"Value"</th>
                                    <th class="num">"P&L"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {move || {
                                    let view = snapshot.get();
                                    let mut rows = view.holdings.clone();
                                    rows.sort_by(|a, b| {
                                        b.value()
                                            .partial_cmp(&a.value())
                                            .unwrap_or(std::cmp::Ordering::Equal)
                                    });

                                    if rows.is_empty() {
                                        return view! {
                                            <tr>
                                                <td colspan="8" class="portfolio-empty">
                                                    "No linked accounts yet — connect a broker above."
                                                </td>
                                            </tr>
                                        }
                                            .into_any();
                                    }

                                    rows.into_iter()
                                        .map(|holding| {
                                            let owner = view
                                                .brokers
                                                .iter()
                                                .find(|broker| broker.id == holding.broker);
                                            let broker = owner.map_or("—", |broker| broker.name);
                                            let tone = owner.map_or("--shade-5", |broker| broker.tone);
                                            holding_row(broker, tone, holding)
                                        })
                                        .collect_view()
                                        .into_any()
                                }}
                            </tbody>
                        </table>
                    </div>
                </HfPanel>

                <div class="portfolio-side">
                    <HfPanel>
                        <HfPanelHeader title="Allocation" />
                        <div class="hfui-panel__content">
                            <div class="alloc-list">
                                {move || {
                                    let view = snapshot.get();
                                    let total = view.summary.total_value().max(1.0);
                                    view.brokers
                                        .iter()
                                        .filter(|broker| broker.status.is_connected())
                                        .map(|broker| {
                                            let value = view.value_at(broker.id);
                                            let share = value / total * 100.0;
                                            view! {
                                                <div class="alloc-row">
                                                    <div class="alloc-row__head">
                                                        <span class="alloc-row__name">
                                                            <span
                                                                class="alloc-row__dot"
                                                                style=format!("background: var({});", broker.tone)
                                                            />
                                                            {broker.name}
                                                        </span>
                                                        <span class="alloc-row__value tnum">
                                                            {format!("{:.1}%", share)}
                                                        </span>
                                                    </div>
                                                    <div class="alloc-row__track">
                                                        <span
                                                            class="alloc-row__bar"
                                                            style=format!(
                                                                "width: {:.1}%; background: var({});",
                                                                share.clamp(0.0, 100.0),
                                                                broker.tone,
                                                            )
                                                        />
                                                    </div>
                                                    <span class="alloc-row__sub tnum">
                                                        {compact_currency(value)}
                                                    </span>
                                                </div>
                                            }
                                        })
                                        .collect_view()
                                }}
                            </div>
                        </div>
                    </HfPanel>

                    <HfPanel>
                        <HfPanelHeader title="Funds & margin" />
                        <div class="hfui-panel__content flush">
                            <table class="hfui-table portfolio-funds">
                                <thead>
                                    <tr>
                                        <th>"Broker"</th>
                                        <th class="num">"Cash"</th>
                                        <th class="num">"Collateral"</th>
                                        <th class="num">"Used"</th>
                                        <th class="num">"Free"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || {
                                        let view = snapshot.get();
                                        if view.funds.is_empty() {
                                            return view! {
                                                <tr>
                                                    <td colspan="5" class="portfolio-empty">
                                                        "Connect a broker to see funds."
                                                    </td>
                                                </tr>
                                            }
                                                .into_any();
                                        }

                                        view.funds
                                            .iter()
                                            .map(|funds| {
                                                let name = view
                                                    .brokers
                                                    .iter()
                                                    .find(|broker| broker.id == funds.broker)
                                                    .map(|broker| broker.name)
                                                    .unwrap_or("—");
                                                view! {
                                                    <tr>
                                                        <td><strong>{name}</strong></td>
                                                        <td class="num tnum">{currency(funds.cash)}</td>
                                                        <td class="num tnum">{currency(funds.collateral)}</td>
                                                        <td class="num tnum">{currency(funds.used_margin)}</td>
                                                        <td class="num tnum"><strong>{currency(funds.free())}</strong></td>
                                                    </tr>
                                                }
                                            })
                                            .collect_view()
                                            .into_any()
                                    }}
                                </tbody>
                            </table>
                        </div>
                    </HfPanel>
                </div>
            </div>

            <EquityChart />
        </div>
    }
}

fn holding_row(broker: &'static str, tone: &'static str, holding: Holding) -> impl IntoView {
    // Derived figures first: the strings below move out of `holding`.
    let value = holding.value();
    let pnl = holding.pnl();
    let pnl_pct = holding.pnl_pct();
    let day = holding.day_pnl();
    let day_pct = holding.day_change_pct;
    let qty = holding.qty;
    let avg_price = holding.avg_price;
    let last_price = holding.last_price;

    let symbol = holding.symbol;
    let name = holding.name;
    let sector = holding.sector;

    let day_class = if day >= 0.0 { "positive" } else { "negative" };
    let pnl_class = if pnl >= 0.0 { "positive" } else { "negative" };

    view! {
        <tr>
            <td>
                <div class="portfolio-symbol">
                    <strong>{symbol}</strong>
                    <span class="muted small">{name}</span>
                </div>
                <span class="portfolio-sector">{sector}</span>
            </td>
            <td>
                <span class="portfolio-broker">
                    <span class="alloc-row__dot" style=format!("background: var({});", tone) />
                    {broker}
                </span>
            </td>
            <td class="num tnum">{quantity(qty)}</td>
            <td class="num tnum">{format!("{:.2}", avg_price)}</td>
            <td class="num tnum">{format!("{:.2}", last_price)}</td>
            <td class=format!("num tnum {}", day_class)>
                {format!("{} ({})", pnl_text(day), percent(day_pct))}
            </td>
            <td class="num tnum">{currency(value)}</td>
            <td class=format!("num tnum {}", pnl_class)>
                {format!("{} ({})", pnl_text(pnl), percent(pnl_pct))}
            </td>
        </tr>
    }
}

/// Signed rupee text for a P&L figure (`+₹1,240.00` / `-₹310.50`).
fn pnl_text(value: f64) -> String {
    if value >= 0.0 {
        format!("+{}", currency(value))
    } else {
        currency(value)
    }
}
