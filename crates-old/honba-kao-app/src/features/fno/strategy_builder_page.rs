//! Multi-Leg Options Strategy Builder (Phase 3.3) at `/fno/strategy`.
//!
//! Everything here is fabricated mock data: no F&O feed or backend is wired.
//! Prices are whole paise (`i64`). The payoff curve is evaluated at expiry
//! only (no T+0 Greeks-adjusted curve). The POP figure is a placeholder
//! display value, not a model output.

use honba_kao_ui_core::{
    HfButton, HfButtonSize, HfButtonVariant, HfInput, HfKpi, HfPageHeader, HfPanel, HfPanelHeader,
    HfTable, HfTableBody, HfTableCell, HfTableHeader, HfTableHeaderCell, HfTableRow, HfToggleGroup,
    HfToggleGroupItem,
};
use leptos::prelude::*;

/// Underlying spot in whole rupees.
const SPOT: i64 = 24_850;
/// Strike spacing in whole rupees.
const STRIKE_STEP: i64 = 50;
/// Payoff chart spans +/- this many percent of spot.
const SPAN_PCT: i64 = 10;
/// Payoff sample count is `CHART_STEPS + 1` (21 points).
const CHART_STEPS: i64 = 20;

const VB_W: f64 = 600.0;
const VB_H: f64 = 240.0;
const PAD: f64 = 16.0;

const DEFAULT_PRESET: &str = "bull_call";
/// Preset toggle options: (toggle value, label).
const PRESETS: [(&str, &str); 6] = [
    ("bull_call", "Bull Call Spread"),
    ("bear_put", "Bear Put Spread"),
    ("long_straddle", "Long Straddle"),
    ("short_strangle", "Short Strangle"),
    ("iron_condor", "Iron Condor"),
    ("iron_butterfly", "Iron Butterfly"),
];

/// One option leg. Prices are paise; `qty` is a unit multiplier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Leg {
    is_call: bool,
    is_long: bool,
    strike: i64,
    premium: i64,
    qty: i64,
}

impl Leg {
    /// Sign applied to the per-unit P&L: `+1` for a long leg, `-1` for a short.
    fn sign(&self) -> i64 {
        if self.is_long { 1 } else { -1 }
    }

    /// Expiry intrinsic value at `spot`, in paise.
    fn intrinsic_paise(&self, spot: i64) -> i64 {
        let rupees = if self.is_call {
            (spot - self.strike).max(0)
        } else {
            (self.strike - spot).max(0)
        };
        rupees * 100
    }
}

/// Mock premium in paise: intrinsic value plus a time-value term that decays
/// with distance from spot. Display-only; not a pricing model.
fn mock_premium(strike: i64, spot: i64, is_call: bool) -> i64 {
    let distance = (spot - strike).abs();
    let intrinsic_rupees = if is_call {
        (spot - strike).max(0)
    } else {
        (strike - spot).max(0)
    };
    let time_value_paise = (14_000 - distance * 24).max(200);
    intrinsic_rupees * 100 + time_value_paise
}

/// Build a leg `offset` strike-steps from ATM with a unit quantity.
fn build_leg(is_call: bool, is_long: bool, offset: i64) -> Leg {
    let strike = SPOT + offset * STRIKE_STEP;
    Leg {
        is_call,
        is_long,
        strike,
        premium: mock_premium(strike, SPOT, is_call),
        qty: 1,
    }
}

/// Legs for a preset id, centred on ATM. Unknown ids fall back to the default.
fn preset_legs(id: &str) -> Vec<Leg> {
    match id {
        "bear_put" => vec![build_leg(false, true, 0), build_leg(false, false, -2)],
        "long_straddle" => vec![build_leg(true, true, 0), build_leg(false, true, 0)],
        "short_strangle" => vec![build_leg(true, false, 2), build_leg(false, false, -2)],
        "iron_condor" => vec![
            build_leg(false, true, -4),
            build_leg(false, false, -2),
            build_leg(true, false, 2),
            build_leg(true, true, 4),
        ],
        "iron_butterfly" => vec![
            build_leg(true, false, 0),
            build_leg(false, false, 0),
            build_leg(true, true, 4),
            build_leg(false, true, -4),
        ],
        _ => vec![build_leg(true, true, 0), build_leg(true, false, 2)],
    }
}

/// Expiry payoff sampled across +/- `SPAN_PCT` of spot.
#[derive(Clone, Debug, PartialEq)]
struct Payoff {
    /// (spot in whole rupees, net P&L in paise).
    points: Vec<(i64, i64)>,
    /// `None` when the curve is still rising at the right edge.
    max_profit: Option<i64>,
    /// `None` when the curve is still falling at either edge.
    max_loss: Option<i64>,
    /// Spot levels (whole rupees) where the sampled curve changes sign.
    breakevens: Vec<i64>,
    /// Sum of signed premiums × qty, in paise. Positive is a net debit.
    net_premium: i64,
}

impl Payoff {
    fn compute(legs: &[Leg]) -> Self {
        let lo = SPOT * (100 - SPAN_PCT) / 100;
        let hi = SPOT * (100 + SPAN_PCT) / 100;

        let points: Vec<(i64, i64)> = (0..=CHART_STEPS)
            .map(|i| {
                let spot = lo + (hi - lo) * i / CHART_STEPS;
                let pnl: i64 = legs
                    .iter()
                    .map(|l| (l.intrinsic_paise(spot) - l.premium) * l.sign() * l.qty)
                    .sum();
                (spot, pnl)
            })
            .collect();

        let n = points.len();
        let (first, second) = (points[0].1, points[1].1);
        let (penult, last) = (points[n - 2].1, points[n - 1].1);
        let right_slope = last - penult;
        let left_slope = second - first;
        let profit_unbounded = right_slope > 0 || left_slope < 0;
        let loss_unbounded = right_slope < 0 || left_slope > 0;

        let max_pnl = points.iter().map(|p| p.1).max().unwrap_or(0);
        let min_pnl = points.iter().map(|p| p.1).min().unwrap_or(0);

        let breakevens = points
            .windows(2)
            .filter(|w| (w[0].1 < 0) != (w[1].1 < 0))
            .map(|w| {
                let ((s0, p0), (s1, p1)) = (w[0], w[1]);
                s0 + (s1 - s0) * (-p0) / (p1 - p0)
            })
            .collect();

        let net_premium = legs.iter().map(|l| l.sign() * l.premium * l.qty).sum();

        Self {
            points,
            max_profit: (!profit_unbounded).then_some(max_pnl),
            max_loss: (!loss_unbounded).then_some(min_pnl),
            breakevens,
            net_premium,
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

/// Format a non-negative paise amount as rupees with two decimals.
fn fmt_paise(paise: i64) -> String {
    format!("{}.{:02}", group_int(paise / 100), paise % 100)
}

/// Signed rupee display for P&L values, e.g. `-₹1,250.00`.
fn fmt_pnl(paise: i64) -> String {
    if paise < 0 {
        format!("-₹{}", fmt_paise(-paise))
    } else {
        format!("₹{}", fmt_paise(paise))
    }
}

fn fmt_opt_pnl(value: Option<i64>) -> String {
    value
        .map(fmt_pnl)
        .unwrap_or_else(|| "Unlimited".to_string())
}

/// Payoff polyline in the fixed viewBox, split at the zero line by clipping
/// the same polyline twice: green above zero, red below.
fn payoff_chart(points: &[(i64, i64)]) -> impl IntoView + use<> {
    let steps = (points.len().saturating_sub(1)).max(1) as f64;
    let hi = points.iter().map(|p| p.1).max().unwrap_or(0).max(0);
    let lo = points.iter().map(|p| p.1).min().unwrap_or(0).min(0);
    let range = (hi - lo).max(1) as f64;
    let plot_h = VB_H - 2.0 * PAD;

    let y_of = |pnl: i64| PAD + (hi - pnl) as f64 / range * plot_h;
    let x_of = |i: usize| PAD + i as f64 / steps * (VB_W - 2.0 * PAD);

    let y0 = y_of(0);
    let loss_h = VB_H - y0;
    let zero_end = VB_W - PAD;
    let line_profit = points
        .iter()
        .enumerate()
        .map(|(i, &(_, pnl))| format!("{:.1},{:.1}", x_of(i), y_of(pnl)))
        .collect::<Vec<_>>()
        .join(" ");
    let line_loss = line_profit.clone();

    view! {
        <svg class="fno__chart" viewBox=format!("0 0 {VB_W} {VB_H}") role="img" aria-label="Net payoff at expiry">
            <defs>
                <clipPath id="fno-clip-profit">
                    <rect x="0" y="0" width=VB_W height=y0 />
                </clipPath>
                <clipPath id="fno-clip-loss">
                    <rect x="0" y=y0 width=VB_W height=loss_h />
                </clipPath>
            </defs>
            <line class="fno__zero" x1=PAD y1=y0 x2=zero_end y2=y0 />
            <polyline class="fno__line fno__line--profit" fill="none" clip-path="url(#fno-clip-profit)" points=line_profit />
            <polyline class="fno__line fno__line--loss" fill="none" clip-path="url(#fno-clip-loss)" points=line_loss />
        </svg>
    }
}

/// One editable row in the legs table. Side and type are read-only badges;
/// quantity edits write straight back to `legs` without rebuilding the row.
#[component]
fn LegRow(
    index: usize,
    leg: Leg,
    legs: RwSignal<Vec<Leg>>,
    table_key: RwSignal<u32>,
) -> impl IntoView {
    let side_class = if leg.is_long {
        "fno__badge fno__badge--buy"
    } else {
        "fno__badge fno__badge--sell"
    };
    let side_label = if leg.is_long { "BUY" } else { "SELL" };
    let type_label = if leg.is_call { "CE" } else { "PE" };

    view! {
        <HfTableRow>
            <HfTableCell>
                <span class=side_class>{side_label}</span>
            </HfTableCell>
            <HfTableCell>
                <span class="fno__badge">{type_label}</span>
            </HfTableCell>
            <HfTableCell class="tnum">{group_int(leg.strike)}</HfTableCell>
            <HfTableCell class="tnum">{fmt_paise(leg.premium)}</HfTableCell>
            <HfTableCell class="tnum">
                <HfInput
                    r#type="number"
                    value=leg.qty.to_string()
                    class="fno__qty"
                    on_input=Callback::new(move |ev: web_sys::Event| {
                        if let Ok(q) = event_target_value(&ev).parse::<i64>() {
                            legs.update(|l| {
                                if let Some(x) = l.get_mut(index) {
                                    x.qty = q.max(0);
                                }
                            });
                        }
                    })
                />
            </HfTableCell>
            <HfTableCell>
                <HfButton
                    size=HfButtonSize::Small
                    variant=HfButtonVariant::Red
                    on_click=Callback::new(move |_| {
                        legs.update(|l| {
                            if index < l.len() {
                                l.remove(index);
                            }
                        });
                        table_key.update(|k| *k += 1);
                    })
                >
                    "Remove"
                </HfButton>
            </HfTableCell>
        </HfTableRow>
    }
}

#[component]
pub fn StrategyBuilderPage() -> impl IntoView {
    let legs = RwSignal::new(preset_legs(DEFAULT_PRESET));
    // Bumped when the leg set changes shape (preset pick or removal) so the
    // table rows rebuild. Quantity edits do not bump it, keeping focus intact.
    let table_key = RwSignal::new(0_u32);
    let payoff = Memo::new(move |_| legs.with(|l| Payoff::compute(l)));

    let lo = SPOT * (100 - SPAN_PCT) / 100;
    let hi = SPOT * (100 + SPAN_PCT) / 100;

    view! {
        <div class="console fno">
            <HfPageHeader title="Strategy Builder" subtitle="NIFTY · Multi-leg options · mock data" />

            <HfPanel class="fno__chain">
                <div class="fno__toolbar">
                    <HfToggleGroup default_value=DEFAULT_PRESET>
                        {PRESETS
                            .iter()
                            .map(|&(value, label)| {
                                view! {
                                    <HfToggleGroupItem
                                        value=value
                                        on_select=Callback::new(move |_| {
                                            legs.set(preset_legs(value));
                                            table_key.update(|k| *k += 1);
                                        })
                                    >
                                        {label}
                                    </HfToggleGroupItem>
                                }
                            })
                            .collect_view()}
                    </HfToggleGroup>
                    <span class="placeholder__note">"Spot ₹24,850.00 · expiry payoff · mock data"</span>
                </div>
            </HfPanel>

            <HfPanel class="fno__legs">
                <HfPanelHeader title="Legs" />
                <div class="fno__scroll">
                    <HfTable class="fno__table">
                        <HfTableHeader>
                            <HfTableRow>
                                <HfTableHeaderCell>"Side"</HfTableHeaderCell>
                                <HfTableHeaderCell>"Type"</HfTableHeaderCell>
                                <HfTableHeaderCell class="tnum">"Strike"</HfTableHeaderCell>
                                <HfTableHeaderCell class="tnum">"Premium"</HfTableHeaderCell>
                                <HfTableHeaderCell class="tnum">"Qty"</HfTableHeaderCell>
                                <HfTableHeaderCell>""</HfTableHeaderCell>
                            </HfTableRow>
                        </HfTableHeader>
                        <HfTableBody>
                            {move || {
                                table_key.get();
                                legs.get_untracked()
                                    .into_iter()
                                    .enumerate()
                                    .map(|(i, leg)| {
                                        view! { <LegRow index=i leg=leg legs=legs table_key=table_key /> }
                                    })
                                    .collect_view()
                            }}
                        </HfTableBody>
                    </HfTable>
                </div>
            </HfPanel>

            <HfPanel class="fno__chartpanel">
                <HfPanelHeader title="Payoff at Expiry" />
                <div class="fno__chart-wrap">
                    {move || payoff_chart(&payoff.get().points)}
                </div>
                <div class="fno__axis">
                    <span class="tnum">{group_int(lo)}</span>
                    <span class="tnum">{format!("{} (spot)", group_int(SPOT))}</span>
                    <span class="tnum">{group_int(hi)}</span>
                </div>
            </HfPanel>

            <HfPanel class="fno__metrics">
                <HfPanelHeader title="Metrics" />
                {move || {
                    let p = payoff.get();
                    if legs.with(Vec::is_empty) {
                        return view! { <span class="placeholder__note">"Add or keep at least one leg to see metrics."</span> }.into_any();
                    }
                    let breakevens = if p.breakevens.is_empty() {
                        "None in range".to_string()
                    } else {
                        p.breakevens.iter().map(|s| group_int(*s)).collect::<Vec<_>>().join(", ")
                    };
                    let (net_label, net_value) = match p.net_premium {
                        n if n > 0 => ("Net Debit", fmt_pnl(n)),
                        n if n < 0 => ("Net Credit", fmt_pnl(-n)),
                        _ => ("Net Premium", fmt_pnl(0)),
                    };
                    view! {
                        <div class="fno__kpis">
                            <HfKpi label="Max Profit">{fmt_opt_pnl(p.max_profit)}</HfKpi>
                            <HfKpi label="Max Loss">{fmt_opt_pnl(p.max_loss)}</HfKpi>
                            <HfKpi label="Breakevens">{breakevens}</HfKpi>
                            <HfKpi label=net_label>{net_value}</HfKpi>
                            <span class="fno__pop">"Est. POP: ~55%"</span>
                        </div>
                    }
                    .into_any()
                }}
            </HfPanel>
        </div>
    }
}
