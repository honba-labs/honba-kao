use honba_kao_ui_core::molecules::fno::StrategyLeg;
use honba_kao_ui_core::organisms::HfStrategyBuilder;
use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn StrategyBuilderPage() -> impl IntoView {
    let legs = RwSignal::new(vec![
        StrategyLeg {
            side: "BUY".to_string(),
            strike: 24500.0,
            expiry: "28-OCT".to_string(),
            lots: 1.0,
        },
        StrategyLeg {
            side: "SELL".to_string(),
            strike: 24700.0,
            expiry: "28-OCT".to_string(),
            lots: 1.0,
        },
    ]);

    view! {
        <HfPage title="Strategy Builder & Payoff Simulator" subtitle="Multi-leg options spreads with breakeven and probability of profit">
            <HfStrategyBuilder
                legs=legs
                max_profit=Signal::derive(|| Some(8500.0))
                max_loss=Signal::derive(|| Some(3200.0))
                breakeven_points=Signal::derive(|| vec![24564.0])
                on_execute_strategy=Callback::new(|_| {})
            />
        </HfPage>
    }
}
