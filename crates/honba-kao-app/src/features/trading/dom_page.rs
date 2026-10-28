use honba_kao_ui_core::organisms::{BookLevel, HfOrderBook};
use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn DomPage() -> impl IntoView {
    let bids = RwSignal::new(vec![
        BookLevel {
            price: 24498.0,
            size: 150.0,
            cumulative_size: 150.0,
            order_count: 5,
        },
        BookLevel {
            price: 24495.0,
            size: 300.0,
            cumulative_size: 450.0,
            order_count: 8,
        },
    ]);
    let asks = RwSignal::new(vec![
        BookLevel {
            price: 24502.0,
            size: 120.0,
            cumulative_size: 120.0,
            order_count: 4,
        },
        BookLevel {
            price: 24505.0,
            size: 280.0,
            cumulative_size: 400.0,
            order_count: 7,
        },
    ]);
    let spread = RwSignal::new(Some((4.0, 1.63)));

    view! {
        <HfPage title="Depth of Market (DOM)" subtitle="Level 2 / Level 3 order book ladder">
            <div class="max-w-md mx-auto h-[600px]">
                <HfOrderBook
                    bids=Signal::derive(move || bids.get())
                    asks=Signal::derive(move || asks.get())
                    spread=Signal::derive(move || spread.get())
                    on_level_click=Callback::new(|_| {})
                />
            </div>
        </HfPage>
    }
}
