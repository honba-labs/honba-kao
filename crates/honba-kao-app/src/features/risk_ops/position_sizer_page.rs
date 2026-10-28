use honba_api_wasm::margin::{
    calculate_per_share_risk, calculate_position_sizing, calculate_risk_amount,
    calculate_suggested_shares,
};
use honba_kao_ui_core::templates::{HfPage, HfTwoColumnSplitLayout};
use honba_kao_ui_core::{Decimal, primitive::HfNumberInput};
use leptos::prelude::*;
use rust_decimal_macros::dec;

#[component]
pub fn PositionSizerPage() -> impl IntoView {
    let account_equity = RwSignal::new(Decimal::from(1_000_000));
    let risk_pct = RwSignal::new(Decimal::from(1)); // 1%
    let entry_price = RwSignal::new(Decimal::from(2500));
    let stop_loss = RwSignal::new(Decimal::from(2450));

    // All numerical algorithms evicted to honba-kao-api-wasm::margin (Zero Floats Policy).
    let risk_amount = move || calculate_risk_amount(account_equity.get(), risk_pct.get());
    let per_share_risk = move || calculate_per_share_risk(entry_price.get(), stop_loss.get());
    let suggested_shares = move || calculate_suggested_shares(risk_amount(), per_share_risk());
    let total_notional = move || suggested_shares() * entry_price.get();

    // Kelly / volatility-target sizing via the compute engine.
    let position_sizing = Signal::derive(move || {
        calculate_position_sizing(
            account_equity.get(),
            dec!(0.55), // assumed win rate
            dec!(1.5),  // assumed win/loss ratio
            dec!(0.10), // 10% annual vol target
            dec!(0.20), // 20% annual asset vol
        )
    });

    let kelly_allocation = move || {
        let ps = position_sizing.get();
        let pct = if account_equity.get().is_zero() {
            Decimal::ZERO
        } else {
            (ps.recommended_notional / account_equity.get()) * dec!(100)
        };
        format!("{:.2}% of total portfolio equity", pct)
    };

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
                    {move || format!("Optimal Half-Kelly Allocation: {}", kelly_allocation())}
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
