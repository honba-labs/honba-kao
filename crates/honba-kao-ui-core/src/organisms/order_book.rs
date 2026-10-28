use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OrderBookFilter {
    #[default]
    All,
    BidsOnly,
    AsksOnly,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BookLevel {
    pub price: f64,
    pub size: f64,
    pub cumulative_size: f64,
    pub order_count: u32,
}

#[component]
pub fn HfOrderBook(
    #[prop(into)] bids: Signal<Vec<BookLevel>>,
    #[prop(into)] asks: Signal<Vec<BookLevel>>,
    #[prop(into)] spread: Signal<Option<(f64, f64)>>, // (absolute, percentage)
    #[prop(into)] on_level_click: Callback<(f64, f64, bool)>, // (price, size, is_bid)
    #[prop(into, optional)] aggregation_options: Option<Vec<f64>>,
    #[prop(into, optional)] active_aggregation: Option<RwSignal<f64>>,
) -> impl IntoView {
    let agg_opts = aggregation_options.unwrap_or_else(|| vec![0.05, 0.10, 0.50, 1.00]);
    let active_agg = active_aggregation.unwrap_or_else(|| RwSignal::new(0.05));

    view! {
        <div class="hf-order-book flex flex-col h-full bg-surface-base border border-border-soft rounded overflow-hidden">
            <div class="hf-book-header flex justify-between items-center p-2 border-b border-border-soft text-xs">
                <span class="font-semibold text-text-high">"Order Book"</span>
                <div class="hf-book-controls flex items-center gap-1">
                    <span class="text-text-muted text-[10px]">"Agg:"</span>
                    <select
                        class="bg-surface-subtle border border-border-soft rounded text-[11px] px-1 py-0.5 text-text-high outline-none"
                        prop:value=move || active_agg.get().to_string()
                        on:change=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                active_agg.set(v);
                            }
                        }
                    >
                        {agg_opts.into_iter().map(|opt| {
                            view! { <option value=opt.to_string()>{format!("{opt:.2}")}</option> }
                        }).collect::<Vec<_>>()}
                    </select>
                </div>
            </div>

            // Asks (Red volume depth from right)
            <div class="hf-book-asks flex-grow overflow-y-auto flex flex-col-reverse text-xs font-mono">
                <For
                    each=move || asks.get()
                    key=|level| format!("{:.4}-ask", level.price)
                    children=move |level| {
                        let px = level.price;
                        let sz = level.size;
                        view! {
                            <div
                                class="hf-book-row relative flex justify-between px-2 py-0.5 hover:bg-rose-500/10 cursor-pointer"
                                on:click=move |_| on_level_click.run((px, sz, false))
                            >
                                <div
                                    class="hf-depth-fill absolute inset-y-0 right-0 bg-rose-500/15"
                                    style=format!("width: {:.1}%;", (level.cumulative_size / 1000.0 * 100.0).min(100.0))
                                />
                                <span class="relative text-rose-400">{format!("{:.2}", level.price)}</span>
                                <span class="relative text-text-muted">{format!("{:.2}", level.size)}</span>
                            </div>
                        }
                    }
                />
            </div>

            // Spread Row
            <div class="hf-book-spread py-1 px-2 bg-surface-subtle border-y border-border-soft flex justify-between text-xs text-text-muted">
                <span>"Spread"</span>
                <span>
                    {move || match spread.get() {
                        Some((abs, bps)) => format!("{:.2} ({:.2} bps)", abs, bps),
                        None => "-".to_string(),
                    }}
                </span>
            </div>

            // Bids (Green volume depth from right)
            <div class="hf-book-bids flex-grow overflow-y-auto flex flex-col text-xs font-mono">
                <For
                    each=move || bids.get()
                    key=|level| format!("{:.4}-bid", level.price)
                    children=move |level| {
                        let px = level.price;
                        let sz = level.size;
                        view! {
                            <div
                                class="hf-book-row relative flex justify-between px-2 py-0.5 hover:bg-emerald-500/10 cursor-pointer"
                                on:click=move |_| on_level_click.run((px, sz, true))
                            >
                                <div
                                    class="hf-depth-fill absolute inset-y-0 right-0 bg-emerald-500/15"
                                    style=format!("width: {:.1}%;", (level.cumulative_size / 1000.0 * 100.0).min(100.0))
                                />
                                <span class="relative text-emerald-400">{format!("{:.2}", level.price)}</span>
                                <span class="relative text-text-muted">{format!("{:.2}", level.size)}</span>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
