use leptos::prelude::*;

#[component]
pub fn HfSpreadIndicator(
    #[prop(into)] spread_abs: Signal<Option<f64>>,
    #[prop(into)] spread_bps: Signal<Option<f64>>,
) -> impl IntoView {
    view! {
        <div class="hf-spread-indicator flex items-center justify-between px-2 py-1 bg-surface-subtle border-y border-border-soft text-xs text-text-muted font-mono">
            <span>"Spread"</span>
            <span>
                {move || match (spread_abs.get(), spread_bps.get()) {
                    (Some(abs), Some(bps)) => format!("{:.2} ({:.2} bps)", abs, bps),
                    (Some(abs), None) => format!("{:.2}", abs),
                    _ => "-".to_string(),
                }}
            </span>
        </div>
    }
}
