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
