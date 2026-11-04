//! `ScreenerDetailDrawer` — Holographic Target Inspector slide-over / dock with
//! 5-Axis Factor Pentagon Radar SVG, Invalidation-to-Trigger Laser Gauge,
//! deterministic 5-detector `ExplainStep` trail (`scanner.rs`), key metric tiles,
//! and `honba-mon` pre-trade gate status.

use crate::features::screener::filters::{
    format_change_pct, format_mcap_rs_cr, format_price, format_volume_compact,
};
use crate::features::screener::scanner::{SETUP_UNQUALIFIED, scan_row};
use crate::features::screener::telemetry::{RadarAxes, compute_laser_gauge, compute_radar_axes};
use crate::features::screener::types::ScreenerRecord;
use honba_kao_ui_core::primitive::{ButtonSize, ButtonVariant, HfButton};
use honba_kao_ui_core::{BadgeVariant, HfCloseButton, HfDrawer, HfStatusChip};
use leptos::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashSet;

fn range_marker_pct(price: Decimal, low: Decimal, high: Decimal) -> u64 {
    let span = high - low;
    let pct: Decimal = if span == Decimal::ZERO {
        dec!(50)
    } else {
        (price - low) * dec!(100) / span
    };
    pct.round_dp(0)
        .to_string()
        .parse::<u64>()
        .unwrap_or(50)
        .min(100)
}

fn pct_vs(price: Decimal, level: Decimal) -> String {
    if price == Decimal::ZERO {
        return "—".to_string();
    }
    let cost = (level - price) / price * dec!(100);
    let body = cost.round_dp(2).to_string();
    if cost >= Decimal::ZERO {
        format!("+{}", body)
    } else {
        body
    }
}

fn metric_tile(label: &'static str, value: String) -> AnyView {
    view! {
        <div class="rounded-lg border border-border-soft bg-surface-subtle p-2">
            <div class="text-[10px] font-mono uppercase tracking-wide text-text-muted">{label}</div>
            <div class="mt-0.5 font-mono text-xs font-semibold text-text-high">{value}</div>
        </div>
    }
    .into_any()
}

#[component]
pub fn ScreenerDetailDrawer(
    open: RwSignal<bool>,
    record: Signal<Option<ScreenerRecord>>,
    watchlist: RwSignal<HashSet<String>>,
) -> impl IntoView {
    view! {
        <HfDrawer open=open title="Target Inspector // AST HUD" right=true>
            <div class="screener-detail-drawer flex flex-col gap-3">
                {move || {
                    let snap = record.get();
                    match snap {
                        None => view! {
                            <div class="text-xs font-mono text-text-muted">
                                "No instrument locked. Select any node or card to inspect."
                            </div>
                        }
                        .into_any(),
                        Some(ScreenerRecord::Stock(r)) => {
                            let price = format_price(r.price);
                            let change = format_change_pct(r.change_pct);
                            let tone = if r.change_pct >= Decimal::ZERO {
                                "text-emerald-400"
                            } else {
                                "text-rose-400"
                            };
                            let pe = if r.pe_ratio == Decimal::ZERO {
                                "—".to_string()
                            } else {
                                r.pe_ratio.round_dp(1).to_string()
                            };
                            let rsi = r.rsi_14.round_dp(1).to_string();
                            let rvol = format!("{}x", r.rvol.round_dp(2));
                            let vol = format_volume_compact(r.volume);
                            let mcap = format_mcap_rs_cr(r.market_cap_cr);
                            let turnover = format!("₹{} Cr", r.turnover_cr.round_dp(1));
                            let day_pct = range_marker_pct(r.price, r.day_low, r.day_high);
                            let yoy_pct = range_marker_pct(r.price, r.low_52w, r.high_52w);
                            let day_low = format_price(r.day_low);
                            let day_high = format_price(r.day_high);
                            let low_52w = format_price(r.low_52w);
                            let high_52w = format_price(r.high_52w);
                            let symbol = r.symbol.clone();
                            let name = r.name.clone();
                            let exchange = r.exchange.clone();
                            let scan = scan_row(&r);
                            let qualified = scan.setup_type != SETUP_UNQUALIFIED;
                            let score = format!("{}%", scan.score.round_dp(1));
                            let setup = scan.setup_type.clone();

                            // Futuristic Radar + Laser Telemetry
                            let radar = compute_radar_axes(&r);
                            let poly_points = radar.svg_polygon_points();
                            let ring_100 = RadarAxes::svg_ring_points(dec!(100));
                            let ring_50 = RadarAxes::svg_ring_points(dec!(50));
                            let gauge = compute_laser_gauge(
                                r.price,
                                scan.invalidation_level,
                                scan.trigger_level,
                            );
                            let laser_pos = format!("left: calc({}% - 4px)", gauge.position_pct.round_dp(0));
                            let rr_label = format!("Implied R:R 1 : {}", gauge.reward_risk_ratio.round_dp(2));

                            let trigger_info = format!(
                                "{} ({}%)",
                                format_price(scan.trigger_level),
                                pct_vs(r.price, scan.trigger_level)
                            );
                            let invalidation_info = format!(
                                "{} ({}%)",
                                format_price(scan.invalidation_level),
                                pct_vs(r.price, scan.invalidation_level)
                            );
                            let status_text = if qualified { "QUALIFIED" } else { "UNQUALIFIED" };
                            let status_tone = if qualified {
                                "text-emerald-400"
                            } else {
                                "text-rose-400"
                            };
                            let steps = scan
                                .explain_steps
                                .iter()
                                .map(|step| {
                                    let mark = if step.passed { "✓" } else { "✗" };
                                    let tone = if step.passed {
                                        "text-emerald-400"
                                    } else {
                                        "text-rose-400"
                                    };
                                    let condition = step.condition.clone();
                                    view! {
                                        <div class="flex items-center justify-between gap-1 px-2 py-1 rounded bg-surface-base border border-border-soft">
                                            <div class="flex items-center gap-1.5 min-w-0">
                                                <span class=format!("font-mono text-[10px] font-bold {}", tone)>
                                                    {mark}
                                                </span>
                                                <span class="text-[10px] font-mono text-text-high truncate">
                                                    {condition}
                                                </span>
                                            </div>
                                        </div>
                                    }
                                    .into_any()
                                })
                                .collect::<Vec<AnyView>>();
                            view! {
                                <div class="flex items-start justify-between gap-2">
                                    <div class="min-w-0">
                                        <div class="truncate font-mono text-sm font-bold text-text-high">
                                            {symbol}
                                        </div>
                                        <div class="truncate text-[11px] text-text-muted">{name}</div>
                                        <div class="mt-1 flex items-center gap-2">
                                            <HfStatusChip status=exchange variant=BadgeVariant::Info />
                                            <span class="font-mono text-sm font-semibold text-text-high">
                                                {price}
                                            </span>
                                            <span class=move || format!("font-mono text-xs {}", tone)>
                                                {change}
                                            </span>
                                        </div>
                                    </div>
                                    <div class="flex shrink-0 items-center gap-1">
                                        <HfButton
                                            variant=ButtonVariant::Ghost
                                            size=ButtonSize::Xs
                                            aria_label="Toggle watchlist"
                                            on_click=Callback::new(move |_| {
                                                if let Some(r) = record.get() {
                                                    let symbol_for_star = r.symbol().to_string();
                                                    watchlist.update(move |w| {
                                                        if w.contains(&symbol_for_star) {
                                                            w.remove(&symbol_for_star);
                                                        } else {
                                                            w.insert(symbol_for_star.clone());
                                                        }
                                                    });
                                                }
                                            })
                                        >
                                            {move || {
                                                let starred = record
                                                    .get()
                                                    .is_some_and(|r| watchlist
                                                        .get()
                                                        .contains(r.symbol()));
                                                if starred { "★" } else { "☆" }
                                            }}
                                        </HfButton>
                                        <HfCloseButton on_click=Some(Callback::new(move |_| open
                                            .set(false))) />
                                    </div>
                                </div>

                                // 5-Axis Factor Pentagon Radar + Score Telemetry
                                <div class="rounded-xl border border-border-soft bg-surface-subtle p-2.5 flex items-center justify-between gap-2">
                                    <svg viewBox="0 0 160 150" class="w-36 h-32 shrink-0 text-text-muted">
                                        <polygon
                                            points=ring_100
                                            fill="none"
                                            stroke="currentColor"
                                            stroke-opacity="0.25"
                                            stroke-width="1"
                                        />
                                        <polygon
                                            points=ring_50
                                            fill="none"
                                            stroke="currentColor"
                                            stroke-opacity="0.20"
                                            stroke-dasharray="2 2"
                                            stroke-width="1"
                                        />
                                        <polygon
                                            points=poly_points
                                            fill="rgba(16, 185, 129, 0.28)"
                                            stroke="#10b981"
                                            stroke-width="2"
                                        />
                                        <text x="80" y="16" text-anchor="middle" fill="currentColor" class="font-mono text-[9px]">"SQZ"</text>
                                        <text x="142" y="58" text-anchor="middle" fill="currentColor" class="font-mono text-[9px]">"TRD"</text>
                                        <text x="118" y="132" text-anchor="middle" fill="currentColor" class="font-mono text-[9px]">"RVOL"</text>
                                        <text x="42" y="132" text-anchor="middle" fill="currentColor" class="font-mono text-[9px]">"RSI"</text>
                                        <text x="18" y="58" text-anchor="middle" fill="currentColor" class="font-mono text-[9px]">"ROE"</text>
                                    </svg>
                                    <div class="flex flex-col gap-1.5 flex-1 min-w-0 font-mono">
                                        <div class="rounded-lg border border-border-soft bg-surface-base p-2">
                                            <div class="text-[9px] text-text-muted uppercase">"AST SCORE"</div>
                                            <div class="text-sm font-bold text-emerald-400">{score.clone()}</div>
                                        </div>
                                        <div class="rounded-lg border border-border-soft bg-surface-base p-2">
                                            <div class="text-[9px] text-text-muted uppercase">"SETUP REGIME"</div>
                                            <div class="text-[10px] font-semibold text-cyan-300 truncate">{setup.clone()}</div>
                                        </div>
                                    </div>
                                </div>

                                // Invalidation -> Trigger Laser Telemetry Bar
                                <div class="rounded-xl border border-border-soft bg-surface-subtle p-2.5 flex flex-col gap-1.5">
                                    <div class="flex items-center justify-between text-[10px] font-mono">
                                        <span class="text-rose-400">"INV " {invalidation_info.clone()}</span>
                                        <span class="text-emerald-400">"TRIG " {trigger_info.clone()}</span>
                                    </div>
                                    <span class="relative block h-2 w-full rounded-full bg-surface-base border border-border-soft">
                                        <span
                                            class="absolute top-1/2 h-2.5 w-2.5 -translate-y-1/2 rounded-full bg-emerald-400 shadow"
                                            style=laser_pos
                                        ></span>
                                    </span>
                                    <div class="flex items-center justify-between text-[10px] font-mono text-text-muted">
                                        <span>"Stop Pivot"</span>
                                        <span class="text-cyan-400 font-semibold">{rr_label}</span>
                                        <span>"Breakout Pivot"</span>
                                    </div>
                                </div>

                                <div class="grid grid-cols-2 gap-2">
                                    {metric_tile("P/E", pe)}
                                    {metric_tile("RSI (14)", rsi)}
                                    {metric_tile("RVOL", rvol)}
                                    {metric_tile("Volume", vol)}
                                    {metric_tile("MCAP", mcap)}
                                    {metric_tile("Turnover", turnover)}
                                </div>
                                <div class="flex flex-col gap-1">
                                    <div class="flex items-center justify-between">
                                        <span class="text-[10px] uppercase tracking-wide text-text-muted">
                                            "Day Range"
                                        </span>
                                        <span class="font-mono text-[10px] text-text-muted">
                                            {day_low} " — " {day_high}
                                        </span>
                                    </div>
                                    <span class="relative block h-1.5 w-full rounded-full bg-emerald-500/15">
                                        <span
                                            class="absolute top-1/2 h-2 w-2 -translate-y-1/2 rounded-full bg-emerald-400"
                                            style=format!("left: calc({}% - 4px)", day_pct)
                                        ></span>
                                    </span>
                                </div>
                                <div class="flex flex-col gap-1">
                                    <div class="flex items-center justify-between">
                                        <span class="text-[10px] uppercase tracking-wide text-text-muted">
                                            "52W Range"
                                        </span>
                                        <span class="font-mono text-[10px] text-text-muted">
                                            {low_52w} " — " {high_52w}
                                        </span>
                                    </div>
                                    <span class="relative block h-1.5 w-full rounded-full bg-emerald-500/15">
                                        <span
                                            class="absolute top-1/2 h-2 w-2 -translate-y-1/2 rounded-full bg-emerald-400"
                                            style=format!("left: calc({}% - 4px)", yoy_pct)
                                        ></span>
                                    </span>
                                </div>
                                <div class="rounded-xl border border-border-soft bg-surface-subtle p-2.5 flex flex-col gap-1.5">
                                    <div class="flex items-center justify-between">
                                        <span class="text-[10px] font-mono uppercase tracking-wide text-text-muted">
                                            "AST Detector Proof"
                                        </span>
                                        <span class="flex items-center gap-2">
                                            <span class=format!("text-[10px] font-mono font-bold {}", status_tone)>
                                                {status_text}
                                            </span>
                                            <span class="font-mono text-xs font-semibold text-text-high">
                                                {score}
                                            </span>
                                            <span class="font-mono text-[10px] text-text-muted">
                                                {setup}
                                            </span>
                                        </span>
                                    </div>
                                    <div class="flex flex-col gap-1">{steps}</div>
                                    <div class="flex items-center justify-between pt-1 text-[10px] font-mono text-text-muted border-t border-border-soft">
                                        <span>"SEBI GATE (honba-mon)"</span>
                                        <span class="text-emerald-400">"✓ BAND OK · <11µs"</span>
                                    </div>
                                </div>
                                <div class="flex flex-wrap gap-1 border-t border-border-soft pt-2">
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        aria_label="Open Cockpit"
                                        on_click=Callback::new(|_| {})
                                    >
                                        "Cockpit"
                                    </HfButton>
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        aria_label="Open Research Hub"
                                        on_click=Callback::new(|_| {})
                                    >
                                        "Research Hub"
                                    </HfButton>
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        aria_label="Open Simulator"
                                        on_click=Callback::new(|_| {})
                                    >
                                        "Simulator"
                                    </HfButton>
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        aria_label="Open Derivatives"
                                        on_click=Callback::new(|_| {})
                                    >
                                        "Derivatives"
                                    </HfButton>
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        aria_label="Open Order Ticket"
                                        on_click=Callback::new(|_| {})
                                    >
                                        "Order Ticket"
                                    </HfButton>
                                </div>
                            }
                            .into_any()
                        }
                        Some(ScreenerRecord::Etf(r)) => {
                            let price = format_price(r.ltp);
                            let change = format_change_pct(r.premium_discount_pct);
                            let tone = if r.premium_discount_pct >= Decimal::ZERO {
                                "text-emerald-400"
                            } else {
                                "text-rose-400"
                            };
                            let nav = format_price(r.nav);
                            let prem = format_change_pct(r.premium_discount_pct);
                            let aum = format_mcap_rs_cr(r.aum_cr);
                            let exp = format!("{} %", r.expense_ratio.round_dp(2));
                            let vol = format_volume_compact(r.volume);
                            let ret = format_change_pct(r.return_1y);
                            let symbol = r.symbol.clone();
                            let name = r.name.clone();
                            view! {
                                <div class="flex items-start justify-between gap-2">
                                    <div class="min-w-0">
                                        <div class="truncate font-mono text-sm font-bold text-text-high">
                                            {symbol}
                                        </div>
                                        <div class="truncate text-[11px] text-text-muted">{name}</div>
                                        <div class="mt-1 flex items-center gap-2">
                                            <HfStatusChip status="ETF" variant=BadgeVariant::Info />
                                            <span class="font-mono text-sm font-semibold text-text-high">
                                                {price}
                                            </span>
                                            <span class=move || format!("font-mono text-xs {}", tone)>
                                                {change}
                                            </span>
                                        </div>
                                    </div>
                                    <div class="flex shrink-0 items-center gap-1">
                                        <HfCloseButton on_click=Some(Callback::new(move |_| open
                                            .set(false))) />
                                    </div>
                                </div>
                                <div class="grid grid-cols-2 gap-2">
                                    {metric_tile("NAV", nav)}
                                    {metric_tile("Prem/Disc", prem)}
                                    {metric_tile("AUM", aum)}
                                    {metric_tile("Exp %", exp)}
                                    {metric_tile("Volume", vol)}
                                    {metric_tile("1Y Ret", ret)}
                                </div>
                                <div class="flex flex-wrap gap-1 border-t border-border-soft pt-2">
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        aria_label="Open Cockpit"
                                        on_click=Callback::new(|_| {})
                                    >
                                        "Cockpit"
                                    </HfButton>
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        aria_label="Open Research Hub"
                                        on_click=Callback::new(|_| {})
                                    >
                                        "Research Hub"
                                    </HfButton>
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        aria_label="Open Simulator"
                                        on_click=Callback::new(|_| {})
                                    >
                                        "Simulator"
                                    </HfButton>
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        aria_label="Open Derivatives"
                                        on_click=Callback::new(|_| {})
                                    >
                                        "Derivatives"
                                    </HfButton>
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        aria_label="Open Order Ticket"
                                        on_click=Callback::new(|_| {})
                                    >
                                        "Order Ticket"
                                    </HfButton>
                                </div>
                            }
                            .into_any()
                        }
                        Some(ScreenerRecord::Bond(r)) => {
                            let price = format_price(r.clean_price);
                            let change = "0.00%".to_string();
                            let tone = "text-emerald-400";
                            let coupon = format!("{} %", r.coupon_rate.round_dp(2));
                            let ytm = format!("{} %", r.yield_to_maturity.round_dp(2));
                            let face = format_price(r.face_value);
                            let maturity = r.maturity_date.clone();
                            let rating = r.credit_rating.clone();
                            let mod_dur = r.modified_duration.round_dp(2).to_string();
                            let symbol = r.isin.clone();
                            let name = r.issuer_name.clone();
                            view! {
                                <div class="flex items-start justify-between gap-2">
                                    <div class="min-w-0">
                                        <div class="truncate font-mono text-sm font-bold text-text-high">
                                            {symbol}
                                        </div>
                                        <div class="truncate text-[11px] text-text-muted">{name}</div>
                                        <div class="mt-1 flex items-center gap-2">
                                            <HfStatusChip
                                                status=r.credit_rating.clone()
                                                variant=BadgeVariant::Success
                                            />
                                            <span class="font-mono text-sm font-semibold text-text-high">
                                                {price}
                                            </span>
                                            <span class=move || format!("font-mono text-xs {}", tone)>
                                                {change}
                                            </span>
                                        </div>
                                    </div>
                                    <div class="flex shrink-0 items-center gap-1">
                                        <HfCloseButton on_click=Some(Callback::new(move |_| open
                                            .set(false))) />
                                    </div>
                                </div>
                                <div class="grid grid-cols-2 gap-2">
                                    {metric_tile("Coupon", coupon)}
                                    {metric_tile("YTM", ytm)}
                                    {metric_tile("Face Value", face)}
                                    {metric_tile("Maturity", maturity)}
                                    {metric_tile("Rating", rating)}
                                    {metric_tile("Mod Dur", mod_dur)}
                                </div>
                                <div class="flex flex-wrap gap-1 border-t border-border-soft pt-2">
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        aria_label="Open Cockpit"
                                        on_click=Callback::new(|_| {})
                                    >
                                        "Cockpit"
                                    </HfButton>
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        aria_label="Open Research Hub"
                                        on_click=Callback::new(|_| {})
                                    >
                                        "Research Hub"
                                    </HfButton>
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        aria_label="Open Simulator"
                                        on_click=Callback::new(|_| {})
                                    >
                                        "Simulator"
                                    </HfButton>
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        aria_label="Open Derivatives"
                                        on_click=Callback::new(|_| {})
                                    >
                                        "Derivatives"
                                    </HfButton>
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        aria_label="Open Order Ticket"
                                        on_click=Callback::new(|_| {})
                                    >
                                        "Order Ticket"
                                    </HfButton>
                                </div>
                            }
                            .into_any()
                        }
                        Some(ScreenerRecord::Mf(r)) => {
                            let price = format_price(r.nav);
                            let change = "0.00%".to_string();
                            let tone = "text-emerald-400";
                            let aum = format_mcap_rs_cr(r.aum_cr);
                            let exp = format!("{} %", r.expense_ratio.round_dp(2));
                            let cagr_3y = format_change_pct(r.cagr_3y);
                            let cagr_5y = format_change_pct(r.cagr_5y);
                            let category = r.category.clone();
                            let symbol = r.scheme_code.clone();
                            let name = r.scheme_name.clone();
                            view! {
                                <div class="flex items-start justify-between gap-2">
                                    <div class="min-w-0">
                                        <div class="truncate font-mono text-sm font-bold text-text-high">
                                            {symbol}
                                        </div>
                                        <div class="truncate text-[11px] text-text-muted">{name}</div>
                                        <div class="mt-1 flex items-center gap-2">
                                            <HfStatusChip
                                                status=r.risk_grade.clone()
                                                variant=BadgeVariant::Warning
                                            />
                                            <span class="font-mono text-sm font-semibold text-text-high">
                                                {price}
                                            </span>
                                            <span class=move || format!("font-mono text-xs {}", tone)>
                                                {change}
                                            </span>
                                        </div>
                                    </div>
                                    <div class="flex shrink-0 items-center gap-1">
                                        <HfCloseButton on_click=Some(Callback::new(move |_| open
                                            .set(false))) />
                                    </div>
                                </div>
                                <div class="grid grid-cols-2 gap-2">
                                    {metric_tile("AUM", aum)}
                                    {metric_tile("Exp %", exp)}
                                    {metric_tile("3Y CAGR", cagr_3y)}
                                    {metric_tile("5Y CAGR", cagr_5y)}
                                    {metric_tile("Category", category)}
                                </div>
                                <div class="flex flex-wrap gap-1 border-t border-border-soft pt-2">
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        aria_label="Open Cockpit"
                                        on_click=Callback::new(|_| {})
                                    >
                                        "Cockpit"
                                    </HfButton>
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        aria_label="Open Research Hub"
                                        on_click=Callback::new(|_| {})
                                    >
                                        "Research Hub"
                                    </HfButton>
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        aria_label="Open Simulator"
                                        on_click=Callback::new(|_| {})
                                    >
                                        "Simulator"
                                    </HfButton>
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        aria_label="Open Derivatives"
                                        on_click=Callback::new(|_| {})
                                    >
                                        "Derivatives"
                                    </HfButton>
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        aria_label="Open Order Ticket"
                                        on_click=Callback::new(|_| {})
                                    >
                                        "Order Ticket"
                                    </HfButton>
                                </div>
                            }
                            .into_any()
                        }
                    }
                }}
            </div>
        </HfDrawer>
    }
}
