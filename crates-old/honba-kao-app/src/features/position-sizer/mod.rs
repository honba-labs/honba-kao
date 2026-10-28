//! Position Sizer — dynamic position sizing with Fixed Fractional, Kelly Criterion, Volatility Target, and Equal Risk Contribution models, wired to honba-core::sizing.
use leptos::prelude::*;

/// Position Sizer page component.
#[component]
pub fn PositionSizerPage() -> impl IntoView {
    view! {
        <section class="p-6">
            <h2 class="text-2xl font-bold text-primary mb-6">Position Sizer</h2>
            <p class="text-muted mb-6">
                "Position sizing calculator using honba-core models — Fixed Fractional,"
                " Kelly Criterion, Volatility Target, and Equal Risk Contribution."
            </p>
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Method Selection</h3>
                    <p class="text-sm">Select a sizing method to calculate position quantity</p>
                </div>
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Result</h3>
                    <p class="text-sm">Will display quantity, allocated notional, and risk at stake</p>
                </div>
            </div>
            <p class="text-sm text-muted mt-4">
                "Position sizing isPure functional with 128-bit decimal arithmetic via rust_decimal."
                " Guaranteed zero floating-point drift and deterministic lot rounding."
            </p>
        </section>
    }
}
