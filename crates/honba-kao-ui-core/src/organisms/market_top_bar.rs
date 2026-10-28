use leptos::prelude::*;

#[component]
pub fn HfMarketTopBar(
    #[prop(into)] symbol: Signal<String>,
    #[prop(into)] ltp: Signal<f64>,
    #[prop(into)] change_abs: Signal<f64>,
    #[prop(into)] change_pct: Signal<f64>,
    #[prop(into)] high_24h: Signal<f64>,
    #[prop(into)] low_24h: Signal<f64>,
    #[prop(into)] volume_24h: Signal<f64>,
    #[prop(into, optional)] currency_prefix: Option<String>,
    #[prop(into, optional)] actions: Option<AnyView>,
) -> impl IntoView {
    let curr = currency_prefix.unwrap_or_else(|| "₹".to_string());
    let curr_c1 = curr.clone();
    let curr_c2 = curr.clone();
    let curr_c3 = curr;

    let is_pos = Signal::derive(move || change_pct.get() >= 0.0);
    let change_class = Signal::derive(move || {
        if is_pos.get() {
            "text-emerald-400 font-mono font-bold"
        } else {
            "text-rose-400 font-mono font-bold"
        }
    });

    view! {
        <div class="hf-market-top-bar flex items-center justify-between px-3 py-2 bg-surface-base border border-border-soft rounded">
            <div class="flex items-center gap-6">
                <div class="flex items-center gap-2">
                    <span class="text-base font-bold text-text-high font-mono tracking-tight">{move || symbol.get()}</span>
                    <span class="px-1.5 py-0.5 rounded text-[10px] font-semibold bg-surface-subtle text-text-muted">"SPOT"</span>
                </div>
                <div class="flex items-baseline gap-2">
                    <span class="text-lg font-bold font-mono text-text-high">{move || format!("{}{:.2}", curr_c1, ltp.get())}</span>
                    <span class=change_class>
                        {move || {
                            let abs = change_abs.get();
                            let pct = change_pct.get();
                            if abs >= 0.0 {
                                format!("+{:.2} (+{:.2}%)", abs, pct)
                            } else {
                                format!("{:.2} ({:.2}%)", abs, pct)
                            }
                        }}
                    </span>
                </div>
                <div class="hidden tablet:flex items-center gap-4 text-xs text-text-muted">
                    <div>
                        <span class="text-text-muted/60">"24h H: "</span>
                        <span class="font-mono text-text-high">{move || format!("{}{:.2}", curr_c2, high_24h.get())}</span>
                    </div>
                    <div>
                        <span class="text-text-muted/60">"24h L: "</span>
                        <span class="font-mono text-text-high">{move || format!("{}{:.2}", curr_c3, low_24h.get())}</span>
                    </div>
                    <div>
                        <span class="text-text-muted/60">"24h Vol: "</span>
                        <span class="font-mono text-text-high">{move || format!("{:.2}M", volume_24h.get() / 1_000_000.0)}</span>
                    </div>
                </div>
            </div>
            {actions.map(|act| view! { <div class="flex items-center gap-2">{act}</div> })}
        </div>
    }
}
