//! Bottom trading drawer, ported from the labs `trading-drawer.tsx`.
//!
//! Collapsed: a 28px strip with the tab shortcuts and the demo-account equity
//! + day P&L. Expanded: a fixed-height panel with tabbed Holdings /
//! Positions / Orders / Trades / Analytics / Depth. The portfolio tables read
//! the demo universe like the rest of the app; orders and trades are the
//! workbench store's local book.

use leptos::prelude::*;

use super::market_depth::MarketDepth;
use crate::data::demo_data;
use crate::stores::layout_store::DrawerTab;
use crate::stores::workbench_store::WorkbenchStore;
use crate::utils::{currency, percent, quantity};

/// The six drawer tabs in display order.
const TABS: [(DrawerTab, &'static str); 6] = [
    (DrawerTab::Holdings, "Holdings"),
    (DrawerTab::Positions, "Positions"),
    (DrawerTab::Orders, "Open Orders"),
    (DrawerTab::Trades, "Trade Book"),
    (DrawerTab::Analytics, "Analytics"),
    (DrawerTab::Depth, "Market Depth"),
];

/// Row count for a tab's badge (static demo universe + store book).
fn tab_count(store: &WorkbenchStore, tab: DrawerTab) -> usize {
    match tab {
        DrawerTab::Holdings => demo_data::demo_holdings().len(),
        DrawerTab::Positions => store.book.positions.get_untracked().len(),
        DrawerTab::Orders => store.book.orders.get_untracked().len(),
        DrawerTab::Trades => store.book.trades.get_untracked().len(),
        _ => 0,
    }
}

#[component]
pub fn TradingDrawer() -> impl IntoView {
    let store = WorkbenchStore::take();

    view! {
        <Show when=move || store.layout.drawer_open.get() fallback=move || view! { <DrawerStrip store=store /> }>
            <section class="wb-drawer">
                <header class="wb-drawer__tabs">
                    <div class="wb-drawer__tab-list">
                        {TABS
                            .iter()
                            .map(|(tab, label)| {
                                let count = tab_count(&store, *tab);
                                view! { <DrawerTabPill store=store tab=*tab label=*label count=count /> }
                            })
                            .collect_view()}
                    </div>
                    <button
                        class="wb-chip wb-chip--ghost"
                        title="Minimize Drawer"
                        on:click=move |_| store.layout.toggle_drawer()
                    >
                        "▾"
                    </button>
                </header>

                <div class="wb-drawer__body">
                    {move || match store.layout.drawer_tab.get() {
                        DrawerTab::Holdings => view! { <HoldingsTable /> }.into_any(),
                        DrawerTab::Positions => view! { <PositionsTable /> }.into_any(),
                        DrawerTab::Orders => view! { <OrdersTable /> }.into_any(),
                        DrawerTab::Trades => view! { <TradesTable /> }.into_any(),
                        DrawerTab::Analytics => view! { <AnalyticsView /> }.into_any(),
                        DrawerTab::Depth => view! { <DepthTab /> }.into_any(),
                    }}
                </div>
            </section>
        </Show>
    }
}

// ---------------------------------------------------------------------------
// Collapsed strip
// ---------------------------------------------------------------------------

/// The 28px strip: tab shortcuts + the aggregated demo book's equity / P&L.
#[component]
fn DrawerStrip(store: WorkbenchStore) -> impl IntoView {
    let summary = demo_data::demo_portfolio().summary;
    let strip_pnl_class = if summary.day_pnl >= 0.0 {
        "wb-strip__pnl wb-strip__pnl--up"
    } else {
        "wb-strip__pnl wb-strip__pnl--down"
    };

    view! {
        <div class="wb-strip">
            <div class="wb-strip__tabs">
                {TABS
                    .iter()
                    .map(|(tab, label)| {
                        let count = tab_count(&store, *tab);
                        view! {
                            <DrawerTabPill store=store tab=*tab label=*label count=count opens=true />
                        }
                    })
                    .collect_view()}
            </div>

            <div class="wb-strip__account">
                <span class="wb-strip__name">"Aggregated book:"</span>
                <span class="wb-strip__equity tnum">{currency(summary.total_value())}</span>
                <span class=strip_pnl_class>{currency(summary.day_pnl)}</span>
                <button
                    class="wb-strip__open"
                    title="Open Trading Drawer"
                    on:click=move |_| store.layout.toggle_drawer()
                >
                    "Portfolio Drawer ▴"
                </button>
            </div>
        </div>
    }
}

/// A tab button with its count badge. Strip items both select the tab and
/// expand the drawer.
#[component]
fn DrawerTabPill(
    store: WorkbenchStore,
    tab: DrawerTab,
    label: &'static str,
    count: usize,
    #[prop(default = false)] opens: bool,
) -> impl IntoView {
    let selected = move || store.layout.drawer_tab.get() == tab;
    view! {
        <button
            class=move || {
                if selected() { "wb-drawer-tab wb-drawer-tab--on" } else { "wb-drawer-tab" }
            }
            title=label
            on:click=move |_| {
                store.layout.set_drawer_tab(tab);
                if opens && !store.layout.drawer_open.get_untracked() {
                    store.layout.toggle_drawer();
                }
            }
        >
            {label}
            {view! { <TabCount count=count /> }}
        </button>
    }
}

#[component]
fn TabCount(count: usize) -> impl IntoView {
    let count_class = if count > 0 {
        "wb-drawer-tab__count wb-drawer-tab__count--on"
    } else {
        "wb-drawer-tab__count"
    };
    view! {
        <span class=count_class>{count}</span>
    }
}

// ---------------------------------------------------------------------------
// Tab bodies
// ---------------------------------------------------------------------------

/// Holdings, straight from the demo universe.
#[component]
fn HoldingsTable() -> impl IntoView {
    let holdings = demo_data::demo_holdings();

    view! {
        <table class="hfui-table wb-drawer-table">
            <thead>
                <tr>
                    <th>"Instrument"</th>
                    <th>"Broker"</th>
                    <th class="num">"Qty"</th>
                    <th class="num">"Avg"</th>
                    <th class="num">"Last"</th>
                    <th class="num">"Value"</th>
                    <th class="num">"P&L"</th>
                </tr>
            </thead>
            <tbody>
                {holdings
                    .into_iter()
                    .map(|holding| {
                        let pnl = holding.pnl();
                        view! {
                            <tr>
                                <td><strong>{holding.symbol.clone()}</strong></td>
                                <td class="wb-drawer-table__muted">{holding.broker}</td>
                                <td class="num tnum">{quantity(holding.qty)}</td>
                                <td class="num tnum">{format!("{:.2}", holding.avg_price)}</td>
                                <td class="num tnum">{format!("{:.2}", holding.last_price)}</td>
                                <td class="num tnum">{currency(holding.value())}</td>
                                <td class=num_tn_pnl(pnl)>{pnl_text(pnl)}</td>
                            </tr>
                        }
                    })
                    .collect_view()}
            </tbody>
        </table>
    }
}

/// Open positions from the workbench book, with inline square-off.
#[component]
fn PositionsTable() -> impl IntoView {
    let store = WorkbenchStore::take();
    view! {
        {move || {
            let positions = store.book.positions.get();
            if positions.is_empty() {
                return view! {
                    <div class="wb-drawer__empty">"No open positions in this session."</div>
                }
                    .into_any();
            }
            positions
                .into_iter()
                .map(|position| {
                    let symbol = position.symbol.clone();
                    let pnl = position.unrealized_pnl;
                    view! {
                        <tr>
                            <td><strong>{position.symbol.clone()}</strong></td>
                            <td>{position.side.position_label()}</td>
                            <td class="num tnum">{quantity(position.quantity)}</td>
                            <td class="num tnum">{format!("{:.2}", position.avg_price)}</td>
                            <td class="num tnum">{format!("{:.2}", position.ltp)}</td>
                            <td class=num_tn_pnl(pnl)>{pnl_text(pnl)}</td>
                            <td class="wb-drawer-table__action">
                                <button
                                    class="wb-chip wb-chip--danger"
                                    on:click=move |_| store.book.close_position(&symbol)
                                >
                                    "✕ Close"
                                </button>
                            </td>
                        </tr>
                    }
                })
                .collect_view()
                .into_any()
        }}
    }
}

/// Working orders with cancel.
#[component]
fn OrdersTable() -> impl IntoView {
    let store = WorkbenchStore::take();
    view! {
        <table class="hfui-table wb-drawer-table">
            <thead>
                <tr>
                    <th>"Order ID"</th>
                    <th>"Symbol"</th>
                    <th>"Side"</th>
                    <th>"Type"</th>
                    <th class="num">"Price"</th>
                    <th class="num">"Qty"</th>
                    <th>"Status"</th>
                    <th></th>
                </tr>
            </thead>
            <tbody>
        {move || {
            let orders = store.book.orders.get();
            if orders.is_empty() {
                return view! {
                    <tr><td class="wb-drawer__empty" colspan=8>"No open orders."</td></tr>
                }
                    .into_any();
            }
            orders
                .into_iter()
                .map(|order| {
                    let order_id = order.order_id.clone();
                    let status = order.status.clone();
                    view! {
                        <tr>
                            <td class="wb-drawer-table__muted">{order.order_id.clone()}</td>
                            <td><strong>{order.symbol.clone()}</strong></td>
                            <td>{order.side.label()}</td>
                            <td class="wb-drawer-table__muted">{order.order_type.as_id()}</td>
                            <td class="num tnum">{format!("{:.2}", order.price)}</td>
                            <td class="num tnum">{quantity(order.quantity)}</td>
                            <td class="wb-status-tag">{status}</td>
                            <td class="wb-drawer-table__action">
                                <button
                                    class="wb-chip wb-chip--danger"
                                    title="Cancel Order"
                                    on:click=move |_| store.book.cancel_order(&order_id)
                                >
                                    "✕ Cancel"
                                </button>
                            </td>
                        </tr>
                    }
                })
                .collect_view()
                .into_any()
        }}
            </tbody>
        </table>
    }
}

/// This session's fills.
#[component]
fn TradesTable() -> impl IntoView {
    let store = WorkbenchStore::take();
    view! {
        <table class="hfui-table wb-drawer-table">
            <thead>
                <tr>
                    <th>"Order ID"</th>
                    <th>"Symbol"</th>
                    <th>"Side"</th>
                    <th class="num">"Exec Price"</th>
                    <th class="num">"Qty"</th>
                    <th class="num">"Costs"</th>
                </tr>
            </thead>
            <tbody>
        {move || {
            let trades = store.book.trades.get();
            if trades.is_empty() {
                return view! {
                    <tr><td class="wb-drawer__empty" colspan=6>"No executions in this session."</td></tr>
                }
                    .into_any();
            }
            trades
                .into_iter()
                .map(|trade| {
                    view! {
                        <tr>
                            <td class="wb-drawer-table__muted">{trade.order_id.clone()}</td>
                            <td><strong>{trade.symbol.clone()}</strong></td>
                            <td>{trade.side.label()}</td>
                            <td class="num tnum">{format!("{:.2}", trade.price)}</td>
                            <td class="num tnum">{quantity(trade.quantity)}</td>
                            <td class="num tnum wb-drawer-table__muted">{format!("{:.2}", trade.costs)}</td>
                        </tr>
                    }
                })
                .collect_view()
                .into_any()
        }}
            </tbody>
        </table>
    }
}

/// KPI chips over the aggregated demo book.
#[component]
fn AnalyticsView() -> impl IntoView {
    let summary = demo_data::demo_portfolio().summary;

    view! {
        <div class="wb-analytics">
            <div class="kpi">
                <span class="kpi__label">"Total value"</span>
                <span class="kpi__value tnum">{currency(summary.total_value())}</span>
            </div>
            <div class="kpi">
                <span class="kpi__label">"Unrealised P&L"</span>
                <span class="kpi__value tnum">{pnl_text(summary.unrealised_pnl)}</span>
                <span class="kpi__delta muted">{percent(summary.unrealised_pct)}</span>
            </div>
            <div class="kpi">
                <span class="kpi__label">"Day P&L"</span>
                <span class="kpi__value tnum">{pnl_text(summary.day_pnl)}</span>
            </div>
            <div class="kpi">
                <span class="kpi__label">"Margin used"</span>
                <span class="kpi__value tnum">{currency(summary.used_margin)}</span>
                <span class="kpi__delta muted">{format!("{} free", currency(summary.free_cash()))}</span>
            </div>
        </div>
    }
}

/// Market depth for the ticket's current symbol.
#[component]
fn DepthTab() -> impl IntoView {
    let store = WorkbenchStore::take();
    view! {
        <div class="wb-depth-tab">
            {move || {
                let symbol = store.draft.ticket.get().symbol;
                view! { <MarketDepth symbol=symbol /> }
            }}
        </div>
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Signed rupee text for a P&L figure.
fn pnl_text(value: f64) -> String {
    if value >= 0.0 {
        format!("+{}", currency(value))
    } else {
        currency(value)
    }
}

/// A `td` class attribute helper: `num tnum` plus gain/loss tint.
fn num_tn_pnl(value: f64) -> &'static str {
    if value >= 0.0 {
        "num tnum wb-cell--gain"
    } else {
        "num tnum wb-cell--loss"
    }
}
