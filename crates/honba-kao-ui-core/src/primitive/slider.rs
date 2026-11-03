//! `HfPercentSlider` — snapping track slider with 0%, 25%, 50%, 75%, 100% tick marks.
//! Ported from `secured-finance-app` atom `Slider`.

use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn HfPercentSlider(
    #[prop(into)] value: RwSignal<f64>, // 0.0 ..= 100.0
    #[prop(into, optional)] disabled: Option<bool>,
) -> impl IntoView {
    let disabled = disabled.unwrap_or(false);
    let ticks = [0.0, 25.0, 50.0, 75.0, 100.0];
    let pct_clamped = Signal::derive(move || value.get().clamp(0.0, 100.0));

    view! {
        <div class="hf-percent-slider flex flex-col gap-2">
            <input
                type="range"
                min="0"
                max="100"
                step="1"
                class="hf-slider-track w-full accent-emerald-600 h-1 bg-surface-subtle rounded-lg appearance-none cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                prop:value=pct_clamped
                disabled=disabled
                on:input=move |ev: web_sys::Event| {
                    let target: web_sys::HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
                    if let Ok(v) = target.value().parse::<f64>() {
                        value.set(v.clamp(0.0, 100.0));
                    }
                }
            />
            <div class="hf-tick-marks flex justify-between text-[10px] text-text-muted">
                {ticks.into_iter().map(|t| {
                    let active = Signal::derive(move || (pct_clamped.get() - t).abs() < 1.0);
                    view! {
                        <div class="hf-tick flex flex-col items-center gap-0.5">
                            <div class=move || format!("hf-tick-dot w-1.5 h-1.5 rounded-full {}",
                                if active.get() { "bg-emerald-600" } else { "bg-border-soft" }
                            )></div>
                            <span>{format!("{:.0}%", t)}</span>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
            <div class="text-[11px] font-mono text-text-high">
                {move || format!("{:.0}%", pct_clamped.get())}
            </div>
        </div>
    }
}
