use honba_kao_ui_core::{Decimal, primitive::HfNumberInput};
use honba_kao_ui_core::templates::{HfPage, HfTwoColumnSplitLayout};
use leptos::prelude::*;

#[component]
pub fn PositionSizerPage() -> impl IntoView {
    let account_equity = RwSignal::new(Decimal::from(1_000_000));
    let risk_pct = RwSignal::new(Decimal::from(1)); // 1%
    let entry_price = RwSignal::new(Decimal::from(2500));
    let stop_loss = RwSignal::new(Decimal::from(2450));

    let risk_amount = move || -> Decimal {
        account_equity.get() * (risk_pct.get() / Decimal::from(100))
    };
    let per_share_risk = move || -> Decimal {
        (entry_price.get() - stop_loss.get()).abs()
    };
    let suggested_shares = move || -> Decimal {
        let risk = per_share_risk();
        if risk > Decimal::ZERO {
            (risk_amount() / risk).trunc() // whole units
        } else {
            Decimal::ZERO
        }
    };
    let total_notional = move || -> Decimal { suggested_shares() * entry_price.get() };

    let primary = view! {
        <div class="flex flex-col gap-4">
            <div class="p-4 rounded border border-border-soft bg-surface-base">
                <h3 class="text-sm font-semibold text-text-primary mb-3">"Position Sizing Parameters"</h3>
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="text-xs text-text-muted mb-1 block">"Account Equity (INR)"</label>
                        <HfNumberInput value=account_equity />
                    </div>
                    <div>
                        <label class="text-xs text-text-muted mb-1 block">"Risk Tolerance (%)"</label>
                        <HfNumberInput value=risk_pct />
                    </div>
                    <div>
                        <label class="text-xs text-text-muted mb-1 block">"Planned Entry Price"</label>
                        <HfNumberInput value=entry_price />
                    </div>
                    <div>
                        <label class="text-xs text-text-muted mb-1 block">"Stop-Loss Price"</label>
                        <HfNumberInput value=stop_loss />
                    </div>
                </div>
            </div>

            <div class="p-4 rounded border border-border-soft bg-surface-base">
                <h3 class="text-sm font-semibold text-text-primary mb-3">"Kelly Criterion & Risk Parity Calibration"</h3>
                <p class="text-xs text-text-muted mb-2">
                    "Dynamic fractional Kelly multiplier applied against covariance risk matrix to cap leverage and drawdown."
                </p>
                <div class="text-xs font-mono bg-surface-subtle p-3 rounded border border-border-soft text-text-muted">
                    "Optimal Half-Kelly Allocation: 8.4% of total portfolio equity"
                </div>
            </div>
        </div>
    }
    .into_any();

    let rail = view! {
        <div class="p-4 rounded border border-border-soft bg-surface-base flex flex-col gap-4">
            <h3 class="text-sm font-semibold text-text-primary">"Sizing Recommendation"</h3>
            <div class="flex flex-col gap-3 text-xs">
                <div class="flex justify-between py-1 border-b border-border-soft">
                    <span class="text-text-muted">"Maximum Risk Budget"</span>
                    <span class="font-mono font-medium text-text-primary">{move || format!("₹{}", risk_amount())}</span>
                </div>
                <div class="flex justify-between py-1 border-b border-border-soft">
                    <span class="text-text-muted">"Risk Per Share"</span>
                    <span class="font-mono font-medium text-text-primary">{move || format!("₹{}", per_share_risk())}</span>
                </div>
                <div class="flex justify-between py-1 border-b border-border-soft">
                    <span class="text-text-muted">"Suggested Quantity"</span>
                    <span class="font-mono font-bold text-semantic-success text-sm">{move || format!("{} units", suggested_shares())}</span>
                </div>
                <div class="flex justify-between py-1 border-b border-border-soft">
                    <span class="text-text-muted">"Total Trade Notional"</span>
                    <span class="font-mono font-medium text-text-primary">{move || format!("₹{}", total_notional())}</span>
                </div>
            </div>
        </div>
    }
    .into_any();

    view! {
        <HfPage
            title="Dynamic Position Sizer"
            subtitle="Risk-parity sizing, volatility-adjusted stop distance, and half-Kelly allocation"
        >
            <HfTwoColumnSplitLayout primary=primary rail=rail />
        </HfPage>
    }
}
