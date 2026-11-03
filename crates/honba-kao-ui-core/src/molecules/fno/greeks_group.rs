//! `HfGreeksGroup` — monospace display of Delta, Gamma, Theta, Vega, Rho.

use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Greeks {
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub rho: f64,
}

#[component]
pub fn HfGreeksGroup(
    greeks: Signal<Greeks>,
) -> impl IntoView {
    let theta_class = Signal::derive(move || {
        if greeks.get().theta >= 0.0 { "text-emerald-400" } else { "text-rose-400" }
    });
    view! {
        <div class="hf-greeks-grid grid grid-cols-2 gap-2 text-[10px] font-mono">
            <div class="hf-greek flex justify-between">
                <span class="text-text-muted">"Δ"</span>
                <span class="text-text-high">{move || format!("{:.3}", greeks.get().delta)}</span>
            </div>
            <div class="hf-greek flex justify-between">
                <span class="text-text-muted">"Γ"</span>
                <span class="text-text-high">{move || format!("{:.3}", greeks.get().gamma)}</span>
            </div>
            <div class="hf-greek flex justify-between">
                <span class="text-text-muted">"Θ"</span>
                <span class=theta_class>{move || format!("{:.3}", greeks.get().theta)}</span>
            </div>
            <div class="hf-greek flex justify-between">
                <span class="text-text-muted">"ν"</span>
                <span class="text-text-high">{move || format!("{:.3}", greeks.get().vega)}</span>
            </div>
            <div class="hf-greek flex justify-between">
                <span class="text-text-muted">"ρ"</span>
                <span class="text-text-high">{move || format!("{:.3}", greeks.get().rho)}</span>
            </div>
        </div>
    }
}
