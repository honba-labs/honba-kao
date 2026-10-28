use leptos::prelude::*;

#[component]
pub fn HfEstimationFields(
    #[prop(into)] notional: Signal<f64>,
    #[prop(into)] est_fee: Signal<f64>,
    #[prop(into)] margin_req: Signal<f64>,
    #[prop(into, optional)] currency_prefix: Option<String>,
) -> impl IntoView {
    let curr = currency_prefix.unwrap_or_else(|| "₹".to_string());
    let curr_c1 = curr.clone();
    let curr_c2 = curr.clone();
    let curr_c3 = curr;
    view! {
        <div class="hf-estimation-fields flex flex-col gap-1 p-2 rounded bg-surface-subtle text-xs border border-border-soft">
            <div class="flex justify-between text-text-muted">
                <span>"Order Value"</span>
                <span class="font-mono text-text-high">{move || format!("{}{:.2}", curr_c1, notional.get())}</span>
            </div>
            <div class="flex justify-between text-text-muted">
                <span>"Est. Fee (1 bp)"</span>
                <span class="font-mono text-text-high">{move || format!("{}{:.2}", curr_c2, est_fee.get())}</span>
            </div>
            <div class="flex justify-between text-text-muted">
                <span>"Margin Required"</span>
                <span class="font-mono text-text-high">{move || format!("{}{:.2}", curr_c3, margin_req.get())}</span>
            </div>
        </div>
    }
}
