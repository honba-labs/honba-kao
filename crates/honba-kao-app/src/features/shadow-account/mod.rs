//! Shadow Account Reconciler — automated reconciliation between internal OMS trade records and broker end-of-day statements, with violation reporting.
use leptos::prelude::*;

/// Shadow Account Reconciler page component.
#[component]
pub fn ShadowAccountPage() -> impl IntoView {
    view! {
        <section class="p-6">
            <h2 class="text-2xl font-bold text-primary mb-6">Shadow Account Reconciler</h2>
            <p class="text-muted mb-6">
                "Reconciliation between internal OMS trade ledger and broker EOD statements under construction."
                " Will detect phantom executions, unmatched positions, corporate action drift, and commission mismatches."
            </p>
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Broker Statement Import</h3>
                    <p class="text-sm">CSV/Zerodha Kite API sync for Groww, Zerodha, and generic formats</p>
                </div>
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Reconciliation Run</h3>
                    <p class="text-sm">Compare internal book vs broker positions, highlight discrepancies</p>
                </div>
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Violation Report</h3>
                    <p class="text-sm">Phantom executions, unmatched positions, commission mismatches, corporate action drift</p>
                </div>
                <div class="col-span-full p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Reconciliation Status</h3>
                    <p class="text-sm">Pending / Completed / With violations / Up to date</p>
                </div>
            </div>
        </section>
    }
}