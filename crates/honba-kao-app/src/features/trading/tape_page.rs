use honba_kao_ui_core::organisms::{HfMarketTape, TapeTrade};
use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn TapePage() -> impl IntoView {
    let trades = RwSignal::new(vec![
        TapeTrade {
            price: 24500.5,
            size: 50.0,
            time: "09:15:02".to_string(),
            is_buyer_maker: false,
        },
        TapeTrade {
            price: 24500.0,
            size: 100.0,
            time: "09:15:01".to_string(),
            is_buyer_maker: true,
        },
        TapeTrade {
            price: 24501.0,
            size: 25.0,
            time: "09:15:00".to_string(),
            is_buyer_maker: false,
        },
    ]);

    view! {
        <HfPage title="Market Tape" subtitle="Time & Sales real-time trade prints">
            <div class="max-w-lg mx-auto h-[600px]">
                <HfMarketTape trades=Signal::derive(move || trades.get()) />
            </div>
        </HfPage>
    }
}
