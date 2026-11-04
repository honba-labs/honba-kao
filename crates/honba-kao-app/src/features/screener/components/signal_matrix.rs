//! `ScreenerSignalMatrix` — futuristic Bento telemetry card grid over the
//! filtered `ScreenerRecord` set, featuring inline 5-detector AST gate badges,
//! `LaserGauge` (`Invalidation -> Price -> Trigger`) bars, and score telemetry.

use crate::features::screener::filters::{format_change_pct, format_mcap_rs_cr, format_price};
use crate::features::screener::scanner::{
    SETUP_BB_SQUEEZE_BREAKOUT, SETUP_BREAKOUT_WATCH, SETUP_TREND_RETEST, scan_row,
};
use crate::features::screener::telemetry::compute_laser_gauge;
use crate::features::screener::types::ScreenerRecord;
use leptos::prelude::*;
use rust_decimal::Decimal;
use std::collections::HashSet;

fn setup_badge_class(setup: &str) -> &'static str {
    if setup == SETUP_BB_SQUEEZE_BREAKOUT {
        "bg-emerald-500/20 text-emerald-300 border-emerald-500/40"
    } else if setup == SETUP_TREND_RETEST {
        "bg-cyan-500/20 text-cyan-300 border-cyan-500/40"
    } else if setup == SETUP_BREAKOUT_WATCH {
        "bg-amber-500/20 text-amber-300 border-amber-500/40"
    } else {
        "bg-rose-500/15 text-rose-300 border-rose-500/30"
    }
}

fn gate_code(condition: &str) -> &'static str {
    if condition.contains("Bollinger") {
        "SQZ"
    } else if condition.contains("SMA 200") {
        "TRD"
    } else if condition.contains("RVOL") {
        "RVOL"
    } else if condition.contains("RSI") {
        "RSI"
    } else {
        "52W"
    }
}

#[component]
pub fn ScreenerSignalMatrix(
    records: Signal<Vec<ScreenerRecord>>,
    active_symbol: RwSignal<String>,
    watchlist: RwSignal<HashSet<String>>,
    drawer_open: RwSignal<bool>,
) -> impl IntoView {
    let cards = move || {
        let rows = records.get();
        if rows.is_empty() {
            return vec![
                view! {
                    <div class="col-span-full flex items-center justify-center p-8 text-xs font-mono text-text-muted">
                        "No instruments match the active AST pipeline filters."
                    </div>
                }
                .into_any(),
            ];
        }

        rows.into_iter()
            .map(|rec| match rec {
                ScreenerRecord::Stock(r) => {
                    let sym = r.symbol.clone();
                    let sym_click = sym.clone();
                    let sym_active = sym.clone();
                    let sym_star = sym.clone();
                    let sym_star_check = sym.clone();
                    let name = r.name.clone();
                    let sector = r.sector.clone();
                    let is_fno = r.is_fno;
                    let price_txt = format_price(r.price);
                    let chg_txt = format_change_pct(r.change_pct);
                    let chg_tone = if r.change_pct >= Decimal::ZERO {
                        "text-emerald-400"
                    } else {
                        "text-rose-400"
                    };

                    let scan = scan_row(&r);
                    let gauge =
                        compute_laser_gauge(r.price, scan.invalidation_level, scan.trigger_level);
                    let setup_cls = setup_badge_class(&scan.setup_type);
                    let setup_label = scan.setup_type.clone();
                    let score_label = format!("{}%", scan.score.round_dp(0));
                    let inv_txt = format_price(scan.invalidation_level);
                    let trig_txt = format_price(scan.trigger_level);
                    let rvol_txt = format!("{}x", r.rvol.round_dp(2));
                    let rsi_txt = format!("{}", r.rsi_14.round_dp(1));
                    let mcap_txt = format_mcap_rs_cr(r.market_cap_cr);
                    let laser_left = format!("left: {}%", gauge.position_pct.round_dp(0));

                    let gate_chips = scan
                        .explain_steps
                        .into_iter()
                        .map(|step| {
                            let code = gate_code(&step.condition);
                            let cond_title = step.condition;
                            let cls = if step.passed {
                                "px-1.5 py-0.5 rounded text-[10px] font-mono border bg-emerald-500/20 border-emerald-500/50 text-emerald-300"
                            } else {
                                "px-1.5 py-0.5 rounded text-[10px] font-mono border bg-surface-subtle border-border-soft text-text-muted opacity-55"
                            };
                            view! {
                                <span title=cond_title class=cls>
                                    {code}
                                </span>
                            }
                            .into_any()
                        })
                        .collect::<Vec<AnyView>>();

                    view! {
                        <div
                            class=move || {
                                let selected = active_symbol.get() == sym_active;
                                format!(
                                    "hf-signal-card cursor-pointer rounded-2xl p-3.5 border transition-all duration-150 bg-surface-base flex flex-col gap-2.5 {}",
                                    if selected {
                                        "border-emerald-500 ring-1 ring-emerald-500/40 shadow-md"
                                    } else {
                                        "border-border-soft hover:border-emerald-500/40 hover:bg-surface-subtle/40"
                                    }
                                )
                            }
                            on:click=move |_| {
                                active_symbol.set(sym_click.clone());
                                drawer_open.set(true);
                            }
                        >
                            <div class="flex items-start justify-between gap-2">
                                <div class="min-w-0">
                                    <div class="flex items-center gap-1.5">
                                        <button
                                            type="button"
                                            class="text-xs leading-none hover:scale-125 transition-transform"
                                            on:click=move |ev: web_sys::MouseEvent| {
                                                ev.stop_propagation();
                                                let s = sym_star.clone();
                                                watchlist.update(move |w| {
                                                    if w.contains(&s) {
                                                        w.remove(&s);
                                                    } else {
                                                        w.insert(s);
                                                    }
                                                });
                                            }
                                        >
                                            {move || {
                                                if watchlist.get().contains(&sym_star_check) {
                                                    view! { <span class="text-amber-400">"★"</span> }.into_any()
                                                } else {
                                                    view! { <span class="text-text-muted">"☆"</span> }.into_any()
                                                }
                                            }}
                                        </button>
                                        <span class="font-mono text-sm font-bold text-text-high truncate">
                                            {sym}
                                        </span>
                                        {is_fno.then(|| {
                                            view! {
                                                <span class="px-1.5 py-0.5 rounded bg-purple-500/15 border border-purple-500/30 text-purple-300 text-[10px] font-mono">
                                                    "F&O"
                                                </span>
                                            }
                                        })}
                                    </div>
                                    <div class="text-[11px] text-text-muted truncate">
                                        {name} " · " {sector}
                                    </div>
                                </div>
                                <div class="text-right font-mono shrink-0">
                                    <div class="text-sm font-bold text-text-high">{price_txt}</div>
                                    <div class=format!("text-xs {}", chg_tone)>{chg_txt}</div>
                                </div>
                            </div>

                            <div class="flex items-center justify-between gap-2">
                                <span class=format!("px-2 py-0.5 rounded-md text-[10px] font-mono font-semibold border {}", setup_cls)>
                                    {setup_label}
                                </span>
                                <div class="flex items-center gap-1 flex-wrap">
                                    {gate_chips}
                                </div>
                                <div class="text-right font-mono shrink-0">
                                    <span class="text-[10px] text-text-muted block leading-none">"AST"</span>
                                    <span class="text-xs font-bold text-emerald-400">{score_label}</span>
                                </div>
                            </div>

                            <div class="flex flex-col gap-1 pt-1">
                                <div class="flex items-center justify-between text-[10px] font-mono">
                                    <span class="text-rose-400">"INV " {inv_txt}</span>
                                    <span class="text-text-muted">
                                        "RVOL " <strong class="text-text-high">{rvol_txt}</strong>
                                        " · RSI " <strong class="text-cyan-400">{rsi_txt}</strong>
                                    </span>
                                    <span class="text-emerald-400">"TRIG " {trig_txt}</span>
                                </div>
                                <div class="relative h-1.5 w-full rounded-full bg-surface-subtle border border-border-soft overflow-hidden">
                                    <div
                                        class="absolute top-0 bottom-0 w-2 rounded-full bg-emerald-400 shadow"
                                        style=laser_left
                                    ></div>
                                </div>
                            </div>

                            <div class="flex items-center justify-between text-[10px] font-mono text-text-muted pt-1 border-t border-border-soft">
                                <span>"MCAP " <strong class="text-text-high">{mcap_txt}</strong></span>
                                <span class="text-cyan-400">
                                    {format!("R:R 1 : {}", gauge.reward_risk_ratio.round_dp(1))}
                                </span>
                            </div>
                        </div>
                    }
                    .into_any()
                }
                other => {
                    let sym = other.symbol().to_string();
                    let sym_click = sym.clone();
                    let sym_active = sym.clone();
                    let nm = other.name().to_string();
                    let pr = format_price(other.price());
                    let chg = format_change_pct(other.change_pct());
                    view! {
                        <div
                            class=move || {
                                let selected = active_symbol.get() == sym_active;
                                format!(
                                    "hf-signal-card cursor-pointer rounded-2xl p-3.5 border transition-all bg-surface-base flex items-center justify-between {}",
                                    if selected {
                                        "border-emerald-500 ring-1 ring-emerald-500/40"
                                    } else {
                                        "border-border-soft hover:border-emerald-500/40"
                                    }
                                )
                            }
                            on:click=move |_| {
                                active_symbol.set(sym_click.clone());
                                drawer_open.set(true);
                            }
                        >
                            <div class="min-w-0">
                                <div class="font-mono text-sm font-bold text-text-high truncate">{sym}</div>
                                <div class="text-xs text-text-muted truncate">{nm}</div>
                            </div>
                            <div class="text-right font-mono shrink-0">
                                <div class="text-sm font-bold text-text-high">{pr}</div>
                                <div class="text-xs text-emerald-400">{chg}</div>
                            </div>
                        </div>
                    }
                    .into_any()
                }
            })
            .collect::<Vec<AnyView>>()
    };

    view! {
        <div class="hf-screener-signal-matrix p-3 grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
            {cards}
        </div>
    }
}
