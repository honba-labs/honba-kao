//! DOM Depth Ladder page. Mock data only: no depth feed or order routing is
//! wired. Prices are whole paise (`i64`), so the mock carries no
//! floating-point money. Buy/Sell only update an inline mock confirmation.

use honba_kao_ui_core::{
    HfButton, HfButtonSize, HfButtonVariant, HfKpi, HfPageHeader, HfPanel, HfPanelHeader, HfTable,
    HfTableBody, HfTableCell, HfTableHeader, HfTableHeaderCell, HfTableRow,
};
use leptos::prelude::*;

const SYMBOL: &str = "RELIANCE";
/// Last traded price in paise (2982.50).
const SPOT: i64 = 298_250;
/// Price tick in paise (0.05).
const TICK: i64 = 5;
/// Levels shown on each side of spot.
const LEVELS: i64 = 10;
/// Mock lot size for the Buy/Sell confirmation.
const MOCK_ORDER_QTY: i64 = 1;

/// One price level of the ladder. Prices are paise.
#[derive(Clone, Copy, Debug)]
struct Level {
    price: i64,
    qty: i64,
    is_bid: bool,
}

/// Deterministic mock quantity for a level `k` ticks from the touch.
fn mock_qty(k: i64, is_bid: bool) -> i64 {
    let seed = k * 37 + if is_bid { 11 } else { 29 };
    100 * (seed % 47 + 5)
}

/// Ladder rows, highest price first: 10 ask levels above spot, then 10 bid
/// levels below spot.
fn build_ladder() -> Vec<Level> {
    let best_bid = SPOT - TICK;
    let best_ask = SPOT;
    let asks = (0..LEVELS).rev().map(|k| Level {
        price: best_ask + k * TICK,
        qty: mock_qty(k, false),
        is_bid: false,
    });
    let bids = (0..LEVELS).map(|k| Level {
        price: best_bid - k * TICK,
        qty: mock_qty(k, true),
        is_bid: true,
    });
    asks.chain(bids).collect()
}

/// Format paise as rupees with two decimals, e.g. `298250` -> `2982.50`.
fn fmt_paise(paise: i64) -> String {
    format!("{}.{:02}", paise / 100, paise.rem_euclid(100))
}

#[component]
fn LadderRow(level: Level, confirmation: RwSignal<String>) -> impl IntoView {
    let price = fmt_paise(level.price);
    let row_class = if level.is_bid {
        "dom-ladder__row dom-ladder__row--bid"
    } else {
        "dom-ladder__row dom-ladder__row--ask"
    };
    let qty = level.qty.to_string();
    let none = String::from("—");
    let (bid_qty, bid_px, ask_px, ask_qty) = if level.is_bid {
        (qty, price.clone(), none, String::new())
    } else {
        (String::new(), none, price.clone(), qty)
    };
    let buy_text = format!("Mock order: BUY {MOCK_ORDER_QTY} @ {price}");
    let sell_text = format!("Mock order: SELL {MOCK_ORDER_QTY} @ {price}");

    view! {
        <HfTableRow class=row_class>
            <HfTableCell class="tnum dom-ladder__qty">{bid_qty}</HfTableCell>
            <HfTableCell class="tnum dom-ladder__bid-px">{bid_px}</HfTableCell>
            <HfTableCell class="tnum dom-ladder__ask-px">{ask_px}</HfTableCell>
            <HfTableCell class="tnum dom-ladder__qty">{ask_qty}</HfTableCell>
            <HfTableCell class="dom-ladder__actions">
                <HfButton
                    size=HfButtonSize::Small
                    variant=HfButtonVariant::Green
                    on_click=Callback::new(move |_| confirmation.set(buy_text.clone()))
                >
                    "Buy"
                </HfButton>
                <HfButton
                    size=HfButtonSize::Small
                    variant=HfButtonVariant::Red
                    on_click=Callback::new(move |_| confirmation.set(sell_text.clone()))
                >
                    "Sell"
                </HfButton>
            </HfTableCell>
        </HfTableRow>
    }
}

#[component]
pub fn DomPage() -> impl IntoView {
    let ladder = StoredValue::new(build_ladder());
    let confirmation = RwSignal::new(String::new());
    let best_bid = fmt_paise(SPOT - TICK);
    let best_ask = fmt_paise(SPOT);
    let spread = fmt_paise(TICK);

    view! {
        <div class="console dom">
            <HfPageHeader
                title="DOM Depth Ladder"
                subtitle="RELIANCE · Market depth · mock data"
            />

            <HfPanel class="dom__summary">
                <HfPanelHeader title="Top of Book" />
                <div class="dom__kpis">
                    <HfKpi label="Symbol">{SYMBOL}</HfKpi>
                    <HfKpi label="Spot">{fmt_paise(SPOT)}</HfKpi>
                    <HfKpi label="Best Bid">{best_bid}</HfKpi>
                    <HfKpi label="Best Ask">{best_ask}</HfKpi>
                    <HfKpi label="Spread">{spread}</HfKpi>
                </div>
            </HfPanel>

            <HfPanel class="dom__ladder">
                <HfPanelHeader title="Depth Ladder" />
                <p class="placeholder__note">
                    "Tick 0.05 · 10 levels each side of spot · mock data"
                </p>
                <div class="dom__confirm" role="status" aria-live="polite">
                    {move || confirmation.get()}
                </div>
                <HfTable class="dom-ladder">
                    <HfTableHeader>
                        <HfTableRow>
                            <HfTableHeaderCell class="tnum">"Bid Qty"</HfTableHeaderCell>
                            <HfTableHeaderCell class="tnum">"Bid Price"</HfTableHeaderCell>
                            <HfTableHeaderCell class="tnum">"Ask Price"</HfTableHeaderCell>
                            <HfTableHeaderCell class="tnum">"Ask Qty"</HfTableHeaderCell>
                            <HfTableHeaderCell>"Order"</HfTableHeaderCell>
                        </HfTableRow>
                    </HfTableHeader>
                    <HfTableBody>
                        {ladder
                            .get_value()
                            .into_iter()
                            .map(|level| view! { <LadderRow level=level confirmation=confirmation /> })
                            .collect_view()}
                    </HfTableBody>
                </HfTable>
            </HfPanel>
        </div>
    }
}
