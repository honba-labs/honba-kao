//! Time & Sales print tape. Mock data only: no trade feed is wired. Prices
//! are whole paise (`i64`). Each print carries the quote in force when it
//! traded, and its aggressor side is derived from that quote.

use honba_kao_ui_core::{
    HfInput, HfPageHeader, HfPanel, HfPanelHeader, HfTable, HfTableBody, HfTableCell,
    HfTableHeader, HfTableHeaderCell, HfTableRow,
};
use leptos::prelude::*;

const SYMBOL: &str = "RELIANCE";
/// Last traded price in paise (2982.50).
const SPOT: i64 = 298_250;
/// Price tick in paise (0.05).
const TICK: i64 = 5;
/// Seconds since midnight for the 09:15:00 open.
const OPEN_SECS: i64 = 9 * 3600 + 15 * 60;
const PRINT_COUNT: i64 = 20;

/// Which side crossed the spread for a print.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Aggressor {
    /// Lifted the offer (traded at or above the ask).
    Buy,
    /// Hit the bid (traded at or below the bid).
    Sell,
    /// Traded inside the spread.
    Neutral,
}

impl Aggressor {
    /// Classify a print against the quote in force when it traded.
    fn derive(price: i64, bid: i64, ask: i64) -> Self {
        if price >= ask {
            Aggressor::Buy
        } else if price <= bid {
            Aggressor::Sell
        } else {
            Aggressor::Neutral
        }
    }

    fn label(self) -> &'static str {
        match self {
            Aggressor::Buy => "Buy",
            Aggressor::Sell => "Sell",
            Aggressor::Neutral => "Mid",
        }
    }

    fn class(self) -> &'static str {
        match self {
            Aggressor::Buy => "tape-side tape-side--buy",
            Aggressor::Sell => "tape-side tape-side--sell",
            Aggressor::Neutral => "tape-side tape-side--mid",
        }
    }
}

/// One print on the tape, oldest first in the mock list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Print {
    /// Seconds since midnight.
    time_secs: i64,
    /// Trade price in paise.
    price: i64,
    qty: i64,
    side: Aggressor,
}

/// Deterministic mock prints across the session, in chronological order.
fn build_prints() -> Vec<Print> {
    (0..PRINT_COUNT)
        .map(|i| {
            let time_secs = OPEN_SECS + i * 11 + (i * i) % 7;
            let walk = (i * 7 + 3) % 5 - 2;
            let mid = SPOT + walk * TICK;
            let bid = mid - TICK;
            let ask = mid + TICK;
            let price = match (i * 5) % 4 {
                0 | 2 => ask,
                1 => bid,
                _ => mid,
            };
            Print {
                time_secs,
                price,
                qty: 1 + ((i * 73) % 40) * 25,
                side: Aggressor::derive(price, bid, ask),
            }
        })
        .collect()
}

/// Format seconds since midnight as `HH:MM:SS`.
fn fmt_time(secs: i64) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

/// Format paise as rupees with two decimals, e.g. `298250` -> `2982.50`.
fn fmt_paise(paise: i64) -> String {
    format!("{}.{:02}", paise / 100, paise.rem_euclid(100))
}

#[component]
pub fn TapePage() -> impl IntoView {
    let prints = StoredValue::new(build_prints());
    let min_qty = RwSignal::new(0_i64);

    // Newest first, with prints below the minimum size hidden.
    let visible = Memo::new(move |_| {
        let min = min_qty.get();
        prints.with_value(|all| {
            all.iter()
                .rev()
                .filter(|p| p.qty >= min)
                .copied()
                .collect::<Vec<Print>>()
        })
    });

    view! {
        <div class="console tape">
            <HfPageHeader
                title="Time & Sales"
                subtitle="RELIANCE · Print tape · mock data"
            />

            <HfPanel class="tape__toolbar">
                <div class="tape__filter">
                    <span class="tape__filter-label">"Min qty"</span>
                    <HfInput
                        r#type="number"
                        value="0".to_string()
                        class="tape__filter-input"
                        on_input=Callback::new(move |ev: web_sys::Event| {
                            let parsed = event_target_value(&ev).trim().parse::<i64>().unwrap_or(0);
                            min_qty.set(parsed.max(0));
                        })
                    />
                </div>
                <span class="placeholder__note">
                    {move || format!("{} of {} prints · {SYMBOL} · mock data", visible.get().len(), PRINT_COUNT)}
                </span>
            </HfPanel>

            <HfPanel class="tape__panel">
                <HfPanelHeader title="Prints" />
                <HfTable class="tape-table">
                    <HfTableHeader>
                        <HfTableRow>
                            <HfTableHeaderCell>"Time"</HfTableHeaderCell>
                            <HfTableHeaderCell class="tnum">"Price"</HfTableHeaderCell>
                            <HfTableHeaderCell class="tnum">"Qty"</HfTableHeaderCell>
                            <HfTableHeaderCell>"Side"</HfTableHeaderCell>
                        </HfTableRow>
                    </HfTableHeader>
                    <HfTableBody>
                        {move || {
                            visible
                                .get()
                                .into_iter()
                                .map(|p| {
                                    view! {
                                        <HfTableRow>
                                            <HfTableCell class="tnum">{fmt_time(p.time_secs)}</HfTableCell>
                                            <HfTableCell class="tnum">{fmt_paise(p.price)}</HfTableCell>
                                            <HfTableCell class="tnum">{p.qty.to_string()}</HfTableCell>
                                            <HfTableCell class=p.side.class()>{p.side.label()}</HfTableCell>
                                        </HfTableRow>
                                    }
                                })
                                .collect_view()
                        }}
                    </HfTableBody>
                </HfTable>
            </HfPanel>
        </div>
    }
}
