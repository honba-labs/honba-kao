use honba_kao_ui_core::molecules::fno::OptionLegQuote;
use honba_kao_ui_core::organisms::{HfOptionChain, OptionStrikePair};
use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn OptionChainPage() -> impl IntoView {
    let spot = RwSignal::new(24500.0);
    let pairs = RwSignal::new(vec![
        OptionStrikePair {
            strike: 24400.0,
            call: OptionLegQuote {
                ltp: 165.0,
                change_pct: 12.5,
                volume: 50000,
                open_interest: 120000,
                oi_change: 15000,
                iv: 0.142,
                delta: 0.65,
                gamma: 0.0004,
            },
            put: OptionLegQuote {
                ltp: 42.0,
                change_pct: -18.2,
                volume: 32000,
                open_interest: 85000,
                oi_change: -8000,
                iv: 0.151,
                delta: -0.35,
                gamma: 0.0004,
            },
        },
        OptionStrikePair {
            strike: 24500.0,
            call: OptionLegQuote {
                ltp: 98.0,
                change_pct: 5.4,
                volume: 120000,
                open_interest: 250000,
                oi_change: 32000,
                iv: 0.138,
                delta: 0.50,
                gamma: 0.0005,
            },
            put: OptionLegQuote {
                ltp: 92.0,
                change_pct: -6.8,
                volume: 110000,
                open_interest: 240000,
                oi_change: 28000,
                iv: 0.140,
                delta: -0.50,
                gamma: 0.0005,
            },
        },
        OptionStrikePair {
            strike: 24600.0,
            call: OptionLegQuote {
                ltp: 48.0,
                change_pct: -15.0,
                volume: 45000,
                open_interest: 90000,
                oi_change: -12000,
                iv: 0.145,
                delta: 0.32,
                gamma: 0.0004,
            },
            put: OptionLegQuote {
                ltp: 172.0,
                change_pct: 14.1,
                volume: 62000,
                open_interest: 140000,
                oi_change: 19000,
                iv: 0.148,
                delta: -0.68,
                gamma: 0.0004,
            },
        },
    ]);

    view! {
        <HfPage title="Futures & Options Matrix" subtitle="Straddle ladder, Greeks analysis, and open interest buildup">
            <div class="h-[650px]">
                <HfOptionChain
                    spot_price=Signal::derive(move || spot.get())
                    pairs=Signal::derive(move || pairs.get())
                    on_select_leg=Callback::new(|_| {})
                />
            </div>
        </HfPage>
    }
}
