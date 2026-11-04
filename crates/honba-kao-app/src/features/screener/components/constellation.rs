//! `ScreenerConstellation` — interactive 2D Alpha Factor Constellation SVG view
//! (`X: RSI-14 Momentum`, `Y: RVOL Surge`, `Radius: AST Scanner Score`).
//!
//! All node coordinates are computed in exact 128-bit `Decimal` via
//! `telemetry::compute_constellation`.

use crate::features::screener::scanner::{
    SETUP_BB_SQUEEZE_BREAKOUT, SETUP_BREAKOUT_WATCH, SETUP_TREND_RETEST,
};
use crate::features::screener::telemetry::compute_constellation;
use crate::features::screener::types::ScreenerEquityRecord;
use leptos::prelude::*;
use rust_decimal_macros::dec;

fn setup_stroke_fill(setup: &str) -> (&'static str, &'static str) {
    if setup == SETUP_BB_SQUEEZE_BREAKOUT {
        ("#10b981", "rgba(16, 185, 129, 0.28)")
    } else if setup == SETUP_TREND_RETEST {
        ("#06b6d4", "rgba(6, 182, 212, 0.28)")
    } else if setup == SETUP_BREAKOUT_WATCH {
        ("#f59e0b", "rgba(245, 158, 11, 0.28)")
    } else {
        ("#f43f5e", "rgba(244, 63, 94, 0.20)")
    }
}

#[component]
pub fn ScreenerConstellation(
    records: Signal<Vec<ScreenerEquityRecord>>,
    active_symbol: RwSignal<String>,
    drawer_open: RwSignal<bool>,
) -> impl IntoView {
    let nodes = move || {
        let pts = compute_constellation(&records.get());
        pts.into_iter()
            .map(|pt| {
                let sym = pt.symbol.clone();
                let sym_click = sym.clone();
                let sym_active = sym.clone();
                let (stroke, fill) = setup_stroke_fill(&pt.setup_type);
                // Map 0..100 percentages into an 800x440 SVG coordinate space
                let cx = (pt.x_pct * dec!(8)).round_dp(1).to_string();
                let cy = (pt.y_pct * dec!(4.4)).round_dp(1).to_string();
                let cx_halo = cx.clone();
                let cy_halo = cy.clone();
                let cx_circle = cx.clone();
                let cy_circle = cy.clone();
                let r = pt.radius_px.to_string();
                let halo_r = (pt.radius_px + dec!(6)).to_string();
                let label_y = (pt.y_pct * dec!(4.4) - pt.radius_px - dec!(6))
                    .round_dp(1)
                    .to_string();
                let tooltip = format!(
                    "{} · {} · Score {}% · RSI {} · RVOL {}x",
                    pt.symbol,
                    pt.setup_type,
                    pt.score.round_dp(0),
                    pt.rsi_14.round_dp(1),
                    pt.rvol.round_dp(2)
                );

                view! {
                    <g
                        class="cursor-pointer transition-opacity hover:opacity-100"
                        on:click=move |_| {
                            active_symbol.set(sym_click.clone());
                            drawer_open.set(true);
                        }
                    >
                        <title>{tooltip}</title>
                        {move || {
                            (active_symbol.get() == sym_active).then(|| {
                                view! {
                                    <circle
                                        cx=cx_halo.clone()
                                        cy=cy_halo.clone()
                                        r=halo_r.clone()
                                        fill="none"
                                        stroke="#ffffff"
                                        stroke-width="2"
                                        stroke-dasharray="3 2"
                                    />
                                }
                            })
                        }}
                        <circle
                            cx=cx_circle
                            cy=cy_circle
                            r=r
                            fill=fill
                            stroke=stroke
                            stroke-width="2"
                        />
                        <text
                            x=cx
                            y=label_y
                            text-anchor="middle"
                            fill="currentColor"
                            class="font-mono text-[10px] font-bold select-none"
                        >
                            {sym}
                        </text>
                    </g>
                }
                .into_any()
            })
            .collect::<Vec<AnyView>>()
    };

    view! {
        <div class="hf-screener-constellation flex flex-col gap-3 p-4">
            <div class="flex flex-wrap items-center justify-between gap-2">
                <div>
                    <div class="text-xs font-mono font-bold text-text-high">
                        "🌌 ALPHA FACTOR CONSTELLATION // X: RSI(14) MOMENTUM · Y: RVOL SURGE · RADIUS: AST SCORE"
                    </div>
                    <div class="text-[11px] text-text-muted">
                        "Click any orbital instrument node to lock target reticle and inspect deterministic AST proof."
                    </div>
                </div>
                <div class="flex items-center gap-3 text-[11px] font-mono">
                    <span class="flex items-center gap-1 text-emerald-400">
                        <span class="h-2.5 w-2.5 rounded-full bg-emerald-400 inline-block"></span>
                        "BB_SQUEEZE"
                    </span>
                    <span class="flex items-center gap-1 text-cyan-400">
                        <span class="h-2.5 w-2.5 rounded-full bg-cyan-400 inline-block"></span>
                        "TREND_RETEST"
                    </span>
                    <span class="flex items-center gap-1 text-amber-400">
                        <span class="h-2.5 w-2.5 rounded-full bg-amber-400 inline-block"></span>
                        "BREAKOUT_WATCH"
                    </span>
                </div>
            </div>

            <div class="relative w-full overflow-hidden rounded-2xl border border-border-soft bg-surface-subtle/50 p-2">
                <svg
                    viewBox="0 0 800 440"
                    class="w-full h-[420px] text-text-high select-none"
                >
                    // RSI(14) 55..70 Sweet Spot vertical zone (mapped to X=400..568)
                    <rect
                        x="400"
                        y="20"
                        width="168"
                        height="390"
                        fill="rgba(16, 185, 129, 0.06)"
                        stroke="rgba(16, 185, 129, 0.30)"
                        stroke-dasharray="4 4"
                    />
                    <text
                        x="484"
                        y="38"
                        text-anchor="middle"
                        fill="rgba(52, 211, 153, 0.8)"
                        class="font-mono text-[10px]"
                    >
                        "RSI(14) SWEET SPOT (55–70)"
                    </text>

                    // RVOL >= 1.50x horizontal surge threshold line (Y = 276)
                    <line
                        x1="40"
                        y1="276"
                        x2="770"
                        y2="276"
                        stroke="rgba(6, 182, 212, 0.40)"
                        stroke-width="1"
                        stroke-dasharray="5 4"
                    />
                    <text
                        x="52"
                        y="268"
                        fill="rgba(34, 211, 238, 0.8)"
                        class="font-mono text-[10px]"
                    >
                        "▲ RVOL ≥ 1.50x SURGE THRESHOLD"
                    </text>

                    // Axis captions
                    <text
                        x="760"
                        y="428"
                        text-anchor="end"
                        fill="currentColor"
                        class="font-mono text-[10px] opacity-60"
                    >
                        "X: RSI(14) Momentum (25 → 85) →"
                    </text>
                    <text
                        x="52"
                        y="38"
                        fill="currentColor"
                        class="font-mono text-[10px] opacity-60"
                    >
                        "↑ Y: RVOL Surge (0.5x → 3.5x)"
                    </text>

                    {nodes}
                </svg>
            </div>
        </div>
    }
}
