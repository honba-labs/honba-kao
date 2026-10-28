//! Position Sizer — dynamic position sizing with Fixed Fractional, Kelly Criterion, Volatility Target, and Equal Risk Contribution models, wired to honba-core::sizing.
use leptos::prelude::*;
use rust_decimal::Decimal;
use crate::core::sizing::{SizingMethod, RiskBudget, PositionSizingRequest, PositionSizingResult, calculate_position_size};

/// Position Sizer page component.
#[component]
pub fn PositionSizerPage() -> impl IntoView {
    let (risk_pct, set_risk_pct) = leptos::use_signal(|| Decimal::from(1u32) / Decimal::from(100u32)); // 1% default
    let (method, set_method) = leptos::use_signal(|| "fixed_fractional".to_string());
    let (result, set_result) = leptos::use_signal(|| None::<PositionSizingResult>);
    let (loading, set_loading) = leptos::use_signal(|| false);

    let instrument = "RELIANCE".to_string();
    let side = "buy".to_string();
    let entry_price = Decimal::from(298250u32); // RELIANCE LTP approx
    let account_equity = crate::shared:: Money::new(Decimal::from(10_000_000u32), "INR"); // ₹1cr

    let handle_calculate = move |_| {
        let risk_pct = *risk_pct;
        let method_str = *method;
        let sizing_method = match method_str.as_str() {
            "fixed_fractional" => {
                SizingMethod::FixedFractional { risk_pct }
            }
            "kelly" => {
                SizingMethod::KellyCriterion {
                    fraction: Decimal::from(5u32) / Decimal::from(10u32), // half-Kelly
                    win_rate: Decimal::from(55u32) / Decimal::from(100u32),
                    payoff_ratio: Decimal::from(2u32) / Decimal::from(1u32),
                }
            }
            "volatility_target" => {
                SizingMethod::VolatilityTarget {
                    target_annual_vol: Decimal::from(15u32) / Decimal::from(100u32),
                    lookback_bars: 20,
                }
            }
            "equal_risk" => SizingMethod::EqualRiskContribution,
            _ => SizingMethod::FixedFractional { risk_pct },
        };

        set_loading(true);
        let request = PositionSizingRequest::new(
            "RELIANCE".into(),
            PositionSide::Long, // simplified - would need proper import
            entry_price,
            account_equity,
            RiskBudget::new(crate::shared::Money::new(Decimal::from(10_000_000u32), "INR"), risk_pct, 1u32.into()),
            sizing_method,
        );
        // Note: calculate_position_size is pure, we call it synchronously here
        match calculate_position_size(&request) {
            Ok(res) => {
                set_result(Some(res));
            }
            Err(e) => {
                // show error
                set_result(Some(PositionSizingResult {
                    instrument: "RELIANCE".into(),
                    quantity: crate::instrument::Quantity::ZERO,
                    allocated_notional: crate::shared::Money::new(Decimal::ZERO, "INR"),
                    risk_at_stake: crate::shared::Money::new(Decimal::ZERO, "INR"),
                    target_weight: None,
                    capped_by_limit: false,
                    reason: Some(e.to_string()),
                }));
            }
        }
        set_loading(false);
    };

    view! {
        <section class="p-6">
            <h2 class="text-2xl font-bold text-primary mb-6">Position Sizer</h2>
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Risk Percentage</h3>
                    <input
                        type="range"
                        min="0"
                        max="50"
                        step="1"
                        class="w-full p-2 border rounded"
                        prop:value=risk_pct.to_string()
                        on:input=move |ev| {
                            if let Ok(v) = ev.value().parse::<u32>() {
                                set_risk_pct(Decimal::from(v) / Decimal::from(100u32));
                            }
                        }
                    />
                    <p class="text-sm mt-2">{format!("{}%", risk_pct * Decimal::from(100u32))}</p>
                </div>
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Sizing Method</h3>
                    <select
                        class="w-full p-2 border rounded"
                        prop:value=method.clone()
                        on:input=move |ev| set_method(ev.target().value())
                    >
                        <option value="fixed_fractional">"Fixed Fractional"</option>
                        <option value="kelly">"Kelly Criterion (Half)"</option>
                        <option value="volatility_target">"Volatility Target"</option>
                        <option value="equal_risk">"Equal Risk Contribution"</option>
                    </select>
                </div>
            </div>
            <div class="mt-6 p-4 border rounded elevated" style="background: var(--surface-panel);">
                <h3 class="font-semibold text-muted mb-2">Sizing Result</h3>
                {move || {
                    let r = *result;
                    if r.risk_at_stake.amount > Decimal::ZERO {
                        view! {
                            <div>
                                <p class="text-sm">Quantity: {r.quantity}</p>
                                <p class="text-sm">Allocated Notional: {r.allocated_notional}</p>
                                <p class="text-sm">Risk at Stake: {r.risk_at_stake}</p>
                                <p class="text-sm">Target Weight: {r.target_weight.map(|w| format!("{}%", w * Decimal::from(100u32))).unwrap_or_default()}</p>
                                {r.reason.map(|reason| <p class="text-xs text-danger mt-1">{reason}</p>)}
                            </div>
                        }
                    } else {
                        view! { <p class="text-muted">Enter parameters above to calculate position size</p> }
                    }
                }}
            </div>
            <div class="mt-6">
                <button
                    class="py-2 px-6 border rounded elevated text-primary hover:bg-primary/10"
                    on:click=handle_calculate
                >
                    {move || if *loading { "Calculating..." } else { "Calculate Position Size" }}
                </button>
            </div>
        </section>
    }
}