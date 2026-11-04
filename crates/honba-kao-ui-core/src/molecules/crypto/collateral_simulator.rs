//! `HfCollateralSimulator` — before / after impact simulation for collateral
//! deposits and withdrawals. Ported from `secured-finance-app` molecule
//! `CollateralSimulationSection`.

use leptos::prelude::*;

/// Before / after impact simulation for collateral deposits / withdrawals.
#[component]
pub fn HfCollateralSimulator(
    #[prop(into)] current_collateral: f64,
    #[prop(into)] proposed_delta: RwSignal<f64>, // positive = deposit, negative = withdraw
) -> impl IntoView {
    let simulated = Signal::derive(move || current_collateral + proposed_delta.get());
    let impact = Signal::derive(move || simulated.get() - current_collateral);

    let on_deposit = move |_| proposed_delta.set(proposed_delta.get() + 10_000.0);
    let on_withdraw = move |_| proposed_delta.set(proposed_delta.get() - 10_000.0);

    view! {
        <div class="hf-collateral-simulator rounded-lg border border-border-soft bg-surface-base p-3 text-xs shadow-sm">
            <span class="text-[10px] font-semibold text-text-muted uppercase tracking-wide block mb-2">"Collateral Simulator"</span>
            <div class="hf-sim-controls flex items-center gap-2 mb-3">
                <button
                    type="button"
                    class="hf-btn px-2 py-1 rounded bg-emerald-600/10 text-emerald-400 hover:bg-emerald-600/20 border border-emerald-500/30 text-xs"
                    on:click=on_deposit
                >
                    "Deposit ₹10k"
                </button>
                <button
                    type="button"
                    class="hf-btn px-2 py-1 rounded bg-rose-600/10 text-rose-400 hover:bg-rose-600/20 border border-rose-500/30 text-xs"
                    on:click=on_withdraw
                >
                    "Withdraw ₹10k"
                </button>
                <span class="ml-auto font-mono text-text-muted">
                    {move || format!("Δ: ₹{:+.2}", proposed_delta.get())}
                </span>
            </div>
            <div class="hf-sim-result flex justify-between items-center p-2 rounded bg-surface-subtle border border-border-soft/50">
                <span class="text-text-muted">"Simulated Collateral"</span>
                <span class="font-mono text-text-high">{move || format!("₹{:.2}", simulated.get())}</span>
                <span class=format!("font-mono {}", if impact.get() >= 0.0 { "text-emerald-400" } else { "text-rose-400" })>
                    {move || format!("{:+.2}", impact.get())}
                </span>
            </div>
        </div>
    }
}
