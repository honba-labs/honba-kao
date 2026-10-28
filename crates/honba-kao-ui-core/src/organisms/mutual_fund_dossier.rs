use crate::molecules::wealth::{Allocation, HfAssetAllocationBar, HfRiskOMeter, RiskLevel};
use leptos::prelude::*;

#[component]
pub fn HfMutualFundDossier(
    #[prop(into)] fund_name: String,
    #[prop(into)] category: String,
    #[prop(into)] aum: f64,
    #[prop(into)] expense_ratio: f64,
    #[prop(into)] nav: f64,
    #[prop(into)] return_1y: f64,
    #[prop(into)] return_3y: f64,
    #[prop(into)] return_5y: f64,
    #[prop(into)] risk_level: RiskLevel,
    #[prop(into)] equity_pct: f64,
    #[prop(into)] debt_pct: f64,
    #[prop(into)] cash_pct: f64,
    #[prop(into)] gold_pct: f64,
) -> impl IntoView {
    view! {
        <div class="hf-mutual-fund-dossier flex flex-col gap-4 p-4 bg-surface-base border border-border-soft rounded">
            <div class="flex justify-between items-start border-b border-border-soft pb-3">
                <div>
                    <h2 class="text-base font-bold text-text-high">{fund_name}</h2>
                    <p class="text-xs text-text-muted">{category}</p>
                </div>
                <div class="text-right font-mono">
                    <div class="text-base font-bold text-text-high">{format!("NAV: ₹{:.2}", nav)}</div>
                    <div class="text-xs text-text-muted">{format!("AUM: ₹{:.0} Cr | TER: {:.2}%", aum, expense_ratio)}</div>
                </div>
            </div>

            <div class="grid grid-cols-1 laptop:grid-cols-3 gap-4">
                // Returns Matrix
                <div class="flex flex-col gap-2 p-3 bg-surface-subtle border border-border-soft rounded">
                    <span class="text-xs font-semibold text-text-muted uppercase">"Trailing Compounded Returns"</span>
                    <div class="grid grid-cols-3 gap-2 text-center font-mono">
                        <div>
                            <div class="text-[10px] text-text-muted">"1 Year"</div>
                            <div class="text-xs font-bold text-emerald-400">{format!("{:.2}%", return_1y)}</div>
                        </div>
                        <div>
                            <div class="text-[10px] text-text-muted">"3 Year"</div>
                            <div class="text-xs font-bold text-emerald-400">{format!("{:.2}%", return_3y)}</div>
                        </div>
                        <div>
                            <div class="text-[10px] text-text-muted">"5 Year"</div>
                            <div class="text-xs font-bold text-emerald-400">{format!("{:.2}%", return_5y)}</div>
                        </div>
                    </div>
                </div>

                // SEBI Riskometer
                <div class="flex justify-center items-center">
                    <HfRiskOMeter level=risk_level />
                </div>

                // Asset Allocation
                <div class="flex flex-col justify-center">
                    <HfAssetAllocationBar
                        alloc=Signal::derive(move || Allocation {
                            equity: equity_pct,
                            debt: debt_pct,
                            cash: cash_pct,
                            gold: gold_pct,
                        })
                    />
                </div>
            </div>
        </div>
    }
}
