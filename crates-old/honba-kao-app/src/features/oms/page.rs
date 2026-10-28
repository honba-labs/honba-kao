//! OMS & Pre-Trade Compliance page component.
use leptos::prelude::*;

/// OMS & Pre-Trade Compliance page component.
#[component]
pub fn OmsPage() -> impl IntoView {
    view! {
        <section class="p-6">
            <h2 class="text-2xl font-bold text-primary mb-4">OMS & Pre-Trade Compliance</h2>
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Order Throttle Rate (OTR)</h3>
                    <p class="text-sm">Maximum orders per time window per symbol</p>
                </div>
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Circuit Filter Checks</h3>
                    <p class="text-sm">Price band distance validation for limit orders</p>
                </div>
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Restricted Symbols</h3>
                    <p class="text-sm">Block list validation for prohibited instruments</p>
                </div>
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Audit Log</h3>
                    <p class="text-sm">Immutable record of all order requests, approvals, and refusals</p>
                </div>
            </div>
        </section>
    }
}