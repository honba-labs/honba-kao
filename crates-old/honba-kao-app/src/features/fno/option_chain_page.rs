//! Option Chain (Phase 3.1) and Strike Summary Bar (Phase 3.2) at `/fno`.
//!
//! Everything here is fabricated mock data: no F&O feed or backend is wired.
//! Prices are whole paise (`i64`), so the mock carries no floating-point money.
//! Greeks and IV are display-only `f64` values derived from simple formulas of
//! distance from spot. They are not Black-Scholes outputs.

use honba_kao_ui_core::{
    HfKpi, HfPageHeader, HfPanel, HfPanelHeader, HfTable, HfTableBody, HfTableCell, HfTableHeader,
    HfTableHeaderCell, HfTableRow, HfToggleGroup, HfToggleGroupItem,
};
use leptos::prelude::*;

/// Underlying spot in whole rupees.
const SPOT: i64 = 24_850;
/// Strike spacing in whole rupees.
const STRIKE_STEP: i64 = 50;
/// Strikes run from ATM - HALF_SPAN to ATM + HALF_SPAN.
const HALF_SPAN: i64 = 7;
const TOTAL_STRIKES: usize = (2 * HALF_SPAN + 1) as usize;

/// Range filter options: (toggle value, label, strikes around ATM).
/// `usize::MAX` means "all strikes".
const RANGE_OPTIONS: [(&str, &str, usize); 4] = [
    ("10", "10", 10),
    ("15", "15", 15),
    ("20", "20", 20),
    ("all", "All", usize::MAX),
];
const DEFAULT_RANGE: &str = "15";

/// One side (call or put) of a strike. Prices are paise.
#[derive(Clone, Copy, Debug)]
struct Leg {
    ltp: i64,
    bid: i64,
    ask: i64,
    iv: f64,
    delta: f64,
    gamma: f64,
    theta: f64,
    vega: f64,
    oi: i64,
    volume: i64,
}

#[derive(Clone, Copy, Debug)]
struct StrikeRow {
    strike: i64,
    /// Strike index relative to ATM (0 = ATM).
    offset: i64,
    call: Leg,
    put: Leg,
}

/// Mock leg for one side at `strike`. `offset` is the strike's index from ATM.
fn leg(strike: i64, offset: i64, is_call: bool) -> Leg {
    let a = offset.abs();
    let af = a as f64;

    let intrinsic_rupees = if is_call {
        (SPOT - strike).max(0)
    } else {
        (strike - SPOT).max(0)
    };
    let time_value_paise = (14_000 - a * 1_200).max(200);
    let ltp = intrinsic_rupees * 100 + time_value_paise;

    let iv_call = 16.0 + 0.6 * af;
    let iv = if is_call {
        iv_call
    } else if offset < 0 {
        iv_call + 0.4
    } else {
        iv_call
    };

    let call_delta = 1.0 / (1.0 + (0.45 * offset as f64).exp());
    let delta = if is_call {
        call_delta
    } else {
        call_delta - 1.0
    };

    let (oi, volume) = if is_call {
        (
            100_000 - 5_000 * a + 3_000 * offset.max(0),
            40_000 - 2_000 * a + 1_000 * offset.max(0),
        )
    } else {
        (
            100_000 - 5_000 * a + 3_600 * (-offset).max(0),
            34_000 - 1_500 * a + 1_500 * (-offset).max(0),
        )
    };

    Leg {
        ltp,
        bid: ltp * 995 / 1000,
        ask: ltp * 1005 / 1000,
        iv,
        delta,
        gamma: 0.0016 * (1.0 - af / 10.0),
        theta: -(0.4 + 0.15 * (HALF_SPAN - a) as f64),
        vega: 12.0 - 0.8 * af,
        oi,
        volume,
    }
}

fn build_chain() -> Vec<StrikeRow> {
    (-HALF_SPAN..=HALF_SPAN)
        .map(|offset| {
            let strike = SPOT + offset * STRIKE_STEP;
            StrikeRow {
                strike,
                offset,
                call: leg(strike, offset, true),
                put: leg(strike, offset, false),
            }
        })
        .collect()
}

/// Whether a strike at `offset` from ATM falls inside a window of `n` strikes.
/// Windows are centred on ATM; even sizes take one extra strike on the lower side. Any `n` at or above the mock size shows the whole chain.
fn in_window(offset: i64, n: usize) -> bool {
    if n >= TOTAL_STRIKES {
        return true;
    }
    let half = (n / 2) as i64;
    offset >= -half && offset < n as i64 - half
}

/// Aggregates shown in the Strike Summary Bar.
#[derive(Clone, Copy, Debug)]
struct Summary {
    pcr_oi: f64,
    pcr_volume: f64,
    max_pain: i64,
    atm_iv: f64,
}

impl Summary {
    fn compute(rows: &[StrikeRow]) -> Self {
        let (call_oi, put_oi, call_vol, put_vol) =
            rows.iter()
                .fold((0_i64, 0_i64, 0_i64, 0_i64), |(co, po, cv, pv), r| {
                    (
                        co + r.call.oi,
                        po + r.put.oi,
                        cv + r.call.volume,
                        pv + r.put.volume,
                    )
                });
        let ratio = |put: i64, call: i64| {
            if call == 0 {
                0.0
            } else {
                put as f64 / call as f64
            }
        };
        // Max pain is approximated as the strike nearest spot.
        let max_pain = rows
            .iter()
            .min_by_key(|r| (r.strike - SPOT).abs())
            .map(|r| r.strike)
            .unwrap_or(SPOT);
        let atm_iv = rows
            .iter()
            .find(|r| r.offset == 0)
            .map(|r| (r.call.iv + r.put.iv) / 2.0)
            .unwrap_or(0.0);

        Self {
            pcr_oi: ratio(put_oi, call_oi),
            pcr_volume: ratio(put_vol, call_vol),
            max_pain,
            atm_iv,
        }
    }
}

/// Group an integer with thousands separators, e.g. `24850` -> `24,850`.
fn group_int(value: i64) -> String {
    let digits = value.unsigned_abs().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if value < 0 { format!("-{out}") } else { out }
}

/// Format paise as rupees with two decimals.
fn fmt_paise(paise: i64) -> String {
    format!("{}.{:02}", group_int(paise / 100), paise.rem_euclid(100))
}

#[component]
fn StrikeSummaryBar(summary: Summary) -> impl IntoView {
    view! {
        <HfPanel class="fno__summary">
            <HfPanelHeader title="Strike Summary" />
            <div class="fno__kpis">
                <HfKpi label="Spot">{format!("{}.00", group_int(SPOT))}</HfKpi>
                <HfKpi label="PCR (OI)">{format!("{:.2}", summary.pcr_oi)}</HfKpi>
                <HfKpi label="PCR (Volume)">{format!("{:.2}", summary.pcr_volume)}</HfKpi>
                <HfKpi label="Max Pain">{group_int(summary.max_pain)}</HfKpi>
                <HfKpi label="ATM IV">{format!("{:.2}%", summary.atm_iv)}</HfKpi>
            </div>
        </HfPanel>
    }
}

/// Nine cells per side, in order: OI, LTP, Bid, Ask, IV, Delta, Gamma, Theta, Vega.
fn leg_cells(leg: Leg) -> impl IntoView {
    view! {
        <HfTableCell class="tnum">{group_int(leg.oi)}</HfTableCell>
        <HfTableCell class="tnum">{fmt_paise(leg.ltp)}</HfTableCell>
        <HfTableCell class="tnum">{fmt_paise(leg.bid)}</HfTableCell>
        <HfTableCell class="tnum">{fmt_paise(leg.ask)}</HfTableCell>
        <HfTableCell class="tnum">{format!("{:.2}%", leg.iv)}</HfTableCell>
        <HfTableCell class="tnum">{format!("{:.2}", leg.delta)}</HfTableCell>
        <HfTableCell class="tnum">{format!("{:.4}", leg.gamma)}</HfTableCell>
        <HfTableCell class="tnum">{format!("{:.2}", leg.theta)}</HfTableCell>
        <HfTableCell class="tnum">{format!("{:.2}", leg.vega)}</HfTableCell>
    }
}

#[component]
pub fn OptionChainPage() -> impl IntoView {
    let chain = StoredValue::new(build_chain());
    let summary = chain.with_value(|rows| Summary::compute(rows));
    let range = RwSignal::new(DEFAULT_RANGE.parse::<usize>().unwrap_or(15));

    view! {
        <div class="console fno">
            <HfPageHeader title="Option Chain" subtitle="NIFTY · Derivatives · mock data" />

            <StrikeSummaryBar summary=summary />

            <HfPanel class="fno__chain">
                <div class="fno__toolbar">
                    <HfToggleGroup default_value=DEFAULT_RANGE>
                        {RANGE_OPTIONS
                            .iter()
                            .map(|&(value, label, n)| {
                                view! {
                                    <HfToggleGroupItem
                                        value=value
                                        on_select=Callback::new(move |_| range.set(n))
                                    >
                                        {label}
                                    </HfToggleGroupItem>
                                }
                            })
                            .collect_view()}
                    </HfToggleGroup>
                    <span class="placeholder__note">"Strikes around ATM · ₹ · mock data"</span>
                </div>

                <div class="fno__scroll">
                    <HfTable class="fno__table">
                        <HfTableHeader>
                            <HfTableRow>
                                <HfTableHeaderCell colspan=9>"Calls"</HfTableHeaderCell>
                                <HfTableHeaderCell class="tnum">"Strike"</HfTableHeaderCell>
                                <HfTableHeaderCell colspan=9>"Puts"</HfTableHeaderCell>
                            </HfTableRow>
                            <HfTableRow>
                                {["OI", "LTP", "Bid", "Ask", "IV", "Delta", "Gamma", "Theta", "Vega"]
                                    .into_iter()
                                    .map(|label| view! { <HfTableHeaderCell class="tnum">{label}</HfTableHeaderCell> })
                                    .collect_view()}
                                <HfTableHeaderCell class="tnum">""</HfTableHeaderCell>
                                {["OI", "LTP", "Bid", "Ask", "IV", "Delta", "Gamma", "Theta", "Vega"]
                                    .into_iter()
                                    .map(|label| view! { <HfTableHeaderCell class="tnum">{label}</HfTableHeaderCell> })
                                    .collect_view()}
                            </HfTableRow>
                        </HfTableHeader>
                        <HfTableBody>
                            {move || {
                                let n = range.get();
                                let visible: Vec<StrikeRow> = chain.with_value(|rows| {
                                    rows.iter().copied().filter(|r| in_window(r.offset, n)).collect()
                                });
                                visible
                                    .into_iter()
                                    .map(|r| {
                                        let class = if r.offset == 0 { "fno__row--atm" } else { "" };
                                        view! {
                                            <HfTableRow class=class>
                                                {leg_cells(r.call)}
                                                <HfTableCell class="tnum fno__strike">
                                                    {group_int(r.strike)}
                                                </HfTableCell>
                                                {leg_cells(r.put)}
                                            </HfTableRow>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </HfTableBody>
                    </HfTable>
                </div>
            </HfPanel>
        </div>
    }
}
