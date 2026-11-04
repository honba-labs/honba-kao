//! `HfColorBar` — cumulative depth background bar for L2 books
//! (Emerald green / Crimson red). Ported from `secured-finance-app` atom `ColorBar`.

use leptos::prelude::*;

/// A single color-bar segment representing depth at a price level.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorBarSegment {
    pub price: f64,
    pub size: f64,
    pub is_bid: bool,
}

/// Cumulative depth background bar for L2 books (Emerald green / Crimson red).
#[component]
pub fn HfColorBar(
    segments: Signal<Vec<ColorBarSegment>>,
    #[prop(into, optional)] max_size: Option<f64>,
    #[prop(into, optional)] height: Option<&'static str>,
) -> impl IntoView {
    let max_size = max_size.unwrap_or(1000.0);
    let height = height.unwrap_or("h-1");

    view! {
        <div class="hf-color-bar flex w-full gap-0.5 overflow-hidden rounded-sm">
            <For
                each=move || segments.get()
                key=|seg| format!("{}-{}", seg.price, if seg.is_bid { "bid" } else { "ask" })
                children=move |seg| {
                    let pct = if max_size > 0.0 {
                        (seg.size / max_size * 100.0).min(100.0)
                    } else {
                        0.0
                    };
                    let color = if seg.is_bid {
                        "bg-emerald-500/60"
                    } else {
                        "bg-rose-500/60"
                    };
                    view! {
                        <div
                            class=format!("hf-color-segment {} {}", height, color)
                            style=format!("width: {:.1}%;", pct)
                            title=format!("{:.2} @ {:.2}", seg.size, seg.price)
                        />
                    }
                }
            />
        </div>
    }
}

/// Open Interest bar with optional change indicator
#[component]
pub fn HfOiBar(
    #[prop(into)] oi: Signal<u64>,
    #[prop(into, optional)] oi_change: Option<Signal<i64>>,
    #[prop(into, optional)] max_oi: Option<u64>,
) -> impl IntoView {
    let max = max_oi.unwrap_or(100_000).max(1) as f64;
    let pct = Signal::derive(move || (oi.get() as f64 / max * 100.0).min(100.0));
    view! {
        <div class="hf-oi-bar flex items-center gap-1.5 w-full">
            <div class="h-2 flex-grow bg-surface-subtle rounded-sm overflow-hidden relative">
                <div
                    class="h-full bg-accent-primary/60 rounded-sm transition-all"
                    style=move || format!("width: {:.1}%;", pct.get())
                />
            </div>
            <span class="text-[10px] font-mono text-text-muted shrink-0">
                {move || format!("{}", oi.get())}
            </span>
            {oi_change.map(|chg| {
                view! {
                    <span class=move || {
                        let c = chg.get();
                        if c >= 0 { "text-[9px] font-mono text-emerald-400 shrink-0" } else { "text-[9px] font-mono text-rose-400 shrink-0" }
                    }>
                        {move || {
                            let c = chg.get();
                            if c >= 0 { format!("+{c}") } else { format!("{c}") }
                        }}
                    </span>
                }
            })}
        </div>
    }
}
