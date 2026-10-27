//! Operations Console — multi-desk dashboard with Trader, Quant, PM, Risk, and System desks, plus emergency kill switches.
use leptos::prelude::*;

use honba_kao_ui_core::{HfButton, HfButtonVariant};

use crate::shared::contexts::use_confirm_dialog;
use crate::shared::stores::{AuditSeverity, use_ops_audit};

/// Operations Console page component.
#[component]
pub fn OpsPage() -> impl IntoView {
    let confirm = use_confirm_dialog();
    let audit = use_ops_audit();

    let engine_halted = RwSignal::new(false);
    let safe_mode = RwSignal::new(false);

    let panic_sell_all = move |_: web_sys::MouseEvent| {
        confirm.request(
            "Panic Sell All",
            "This immediately liquidates every active market position at best bid. This cannot be undone. Continue?",
            Callback::new(move |()| {
                audit.log(
                    "Panic Sell All",
                    "Liquidation order dispatched for all open positions",
                    AuditSeverity::Critical,
                );
            }),
        );
    };

    let cancel_all_pending = move |_: web_sys::MouseEvent| {
        confirm.request(
            "Cancel All Pending",
            "This cancels every unexecuted limit and stop order across all desks. Continue?",
            Callback::new(move |()| {
                audit.log(
                    "Cancel All Pending",
                    "Cancel request dispatched for all unexecuted orders",
                    AuditSeverity::Warning,
                );
            }),
        );
    };

    let toggle_engine_halt = move |_: web_sys::MouseEvent| {
        let halting = !engine_halted.get();
        let (title, message) = if halting {
            (
                "Halt Trading Engine",
                "This freezes all algo trading loops and rejects new incoming signals. Continue?",
            )
        } else {
            (
                "Resume Trading Engine",
                "This resumes algo trading loops and accepts new incoming signals. Continue?",
            )
        };
        confirm.request(
            title,
            message,
            Callback::new(move |()| {
                engine_halted.set(halting);
                audit.log(
                    if halting {
                        "Halt Trading Engine"
                    } else {
                        "Resume Trading Engine"
                    },
                    if halting {
                        "All algo trading loops frozen; new signals rejected"
                    } else {
                        "Algo trading loops resumed; signals accepted"
                    },
                    AuditSeverity::Critical,
                );
            }),
        );
    };

    let toggle_safe_mode = move |_: web_sys::MouseEvent| {
        let enabling = !safe_mode.get();
        let (title, message) = if enabling {
            (
                "Enable Safe Mode",
                "This restricts operations to risk-reducing orders only (closes/cancels; no new risk-adding orders). Continue?",
            )
        } else {
            (
                "Disable Safe Mode",
                "This lifts the risk-reducing-only restriction and allows new risk-adding orders. Continue?",
            )
        };
        confirm.request(
            title,
            message,
            Callback::new(move |()| {
                safe_mode.set(enabling);
                audit.log(
                    "Safe Mode Toggle",
                    if enabling {
                        "Restricted to risk-reducing orders only"
                    } else {
                        "Risk-reducing-only restriction lifted"
                    },
                    AuditSeverity::Warning,
                );
            }),
        );
    };

    view! {
        <section class="p-6">
            <h2 class="text-2xl font-bold text-primary mb-6">Operations Console</h2>
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Trader Desk</h3>
                    <p class="text-sm">Open orders, executed trades, position monitoring</p>
                </div>
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Quant Desk</h3>
                    <p class="text-sm">Model runs, backtest status, alpha factor performance</p>
                </div>
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">PM Desk</h3>
                    <p class="text-sm">Portfolio snapshot, PnL, risk alerts</p>
                </div>
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Risk Desk</h3>
                    <p class="text-sm">VaR, exposure heatmap, stress test results</p>
                </div>
                <div class="col-span-full p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">System Desk</h3>
                    <p class="text-sm">Feed health, WebSocket latency, data quality scoring</p>
                </div>
            </div>

            {move || (engine_halted.get() || safe_mode.get()).then(|| view! {
                <div class="mt-6 flex gap-2">
                    {engine_halted.get().then(|| view! {
                        <span class="px-3 py-1 rounded text-xs font-semibold" style="background: var(--risk-critical); color: var(--text-inverse);">
                            "⛔ TRADING ENGINE HALTED"
                        </span>
                    })}
                    {safe_mode.get().then(|| view! {
                        <span class="px-3 py-1 rounded text-xs font-semibold" style="background: var(--risk-medium); color: var(--text-inverse);">
                            "🛟 SAFE MODE ACTIVE"
                        </span>
                    })}
                </div>
            })}

            <div class="mt-6 p-4 border rounded elevated" style="background: var(--risk-critical); color: var(--text-inverse);">
                <h3 class="font-semibold text-warning mb-2">Emergency Kill Switches</h3>
                <div class="grid grid-cols-1 gap-2 md:grid-cols-4">
                    <HfButton variant=HfButtonVariant::Red class="w-full".to_string() on_click=Callback::new(panic_sell_all)>
                        "Panic Sell All"
                    </HfButton>
                    <HfButton variant=HfButtonVariant::Red class="w-full".to_string() on_click=Callback::new(cancel_all_pending)>
                        "Cancel All Pending"
                    </HfButton>
                    <HfButton
                        variant=HfButtonVariant::Red
                        class="w-full".to_string()
                        active=Signal::derive(move || engine_halted.get())
                        on_click=Callback::new(toggle_engine_halt)
                    >
                        {move || if engine_halted.get() { "Resume Trading Engine" } else { "Halt Trading Engine" }}
                    </HfButton>
                    <HfButton
                        variant=HfButtonVariant::Red
                        class="w-full".to_string()
                        active=Signal::derive(move || safe_mode.get())
                        on_click=Callback::new(toggle_safe_mode)
                    >
                        {move || if safe_mode.get() { "Disable Safe Mode" } else { "Safe Mode Toggle" }}
                    </HfButton>
                </div>
            </div>

            <div class="mt-6 p-4 border rounded elevated">
                <h3 class="font-semibold text-muted mb-2">Kill-Switch Audit Trail</h3>
                {move || {
                    let entries = audit.entries().get();
                    if entries.is_empty() {
                        view! { <p class="text-sm text-muted">No kill-switch actions logged this session.</p> }.into_any()
                    } else {
                        view! {
                            <ul class="text-sm space-y-1">
                                {entries.into_iter().map(|entry| view! {
                                    <li class="flex gap-2">
                                        <span class="text-muted font-mono">{entry.timestamp}</span>
                                        <span class="font-semibold" class:text-danger=move || entry.severity == AuditSeverity::Critical>
                                            {entry.severity.label()}
                                        </span>
                                        <span class="font-medium">{entry.action}</span>
                                        <span class="text-muted">{entry.detail}</span>
                                    </li>
                                }).collect_view()}
                            </ul>
                        }.into_any()
                    }
                }}
            </div>
        </section>
    }
}
