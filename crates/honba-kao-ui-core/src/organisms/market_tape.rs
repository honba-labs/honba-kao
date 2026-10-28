use crate::molecules::core::HfTapeRow;
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct TapeTrade {
    pub price: f64,
    pub size: f64,
    pub time: String,
    pub is_buyer_maker: bool,
}

#[component]
pub fn HfMarketTape(
    #[prop(into)] trades: Signal<Vec<TapeTrade>>,
    #[prop(into, optional)] on_trade_click: Option<Callback<TapeTrade>>,
) -> impl IntoView {
    view! {
        <div class="hf-market-tape flex flex-col h-full bg-surface-base border border-border-soft rounded overflow-hidden">
            <div class="hf-tape-header flex justify-between items-center p-2 border-b border-border-soft text-xs">
                <span class="font-semibold text-text-high">"Market Tape (Time & Sales)"</span>
                <span class="text-[10px] text-text-muted">"Real-time"</span>
            </div>
            <div class="flex justify-between px-2 py-1 bg-surface-subtle border-b border-border-soft text-[10px] text-text-muted font-mono">
                <span>"Price"</span>
                <span>"Size"</span>
                <span>"Time"</span>
            </div>
            <div class="hf-tape-stream flex-grow overflow-y-auto">
                <For
                    each=move || {
                        trades
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, tr)| (i, tr))
                            .collect::<Vec<_>>()
                    }
                    key=|(i, _tr)| format!("tape-{i}")
                    children=move |(_i, tr)| {
                        let tr_c = tr.clone();
                        let click_cb = on_trade_click.map(|cb| Callback::new(move |_| cb.run(tr_c.clone())));
                        match click_cb {
                            Some(cb) => view! {
                                <HfTapeRow
                                    price=tr.price
                                    size=tr.size
                                    time=tr.time.clone()
                                    is_buyer_maker=tr.is_buyer_maker
                                    on_click=cb
                                />
                            }.into_any(),
                            None => view! {
                                <HfTapeRow
                                    price=tr.price
                                    size=tr.size
                                    time=tr.time.clone()
                                    is_buyer_maker=tr.is_buyer_maker
                                />
                            }.into_any(),
                        }
                    }
                />
            </div>
        </div>
    }
}
