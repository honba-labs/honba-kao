use honba_kao_ui_core::molecules::wealth::RiskLevel;
use honba_kao_ui_core::organisms::HfMutualFundDossier;
use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn MutualFundsPage() -> impl IntoView {
    view! {
        <HfPage title="Mutual Funds & Wealth Management" subtitle="Institutional fund analytics, rolling alpha, and portfolio overlap">
            <HfMutualFundDossier
                fund_name="Parag Parikh Flexi Cap Fund - Direct (Growth)".to_string()
                category="Flexi Cap Equity".to_string()
                aum=74500.0
                expense_ratio=0.62
                nav=82.45
                return_1y=28.4
                return_3y=21.2
                return_5y=24.8
                risk_level=RiskLevel::VeryHigh
                equity_pct=84.5
                debt_pct=10.2
                cash_pct=5.3
                gold_pct=0.0
            />
        </HfPage>
    }
}
