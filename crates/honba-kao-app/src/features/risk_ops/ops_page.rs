use honba_kao_ui_core::primitive::{ButtonVariant, HfButton};
use honba_kao_ui_core::templates::{HfPage, HfTwoColumnSplitLayout};
use leptos::prelude::*;

#[component]
pub fn OpsPage() -> impl IntoView {
    let emergency_active = RwSignal::new(false);
    let cancel_all = Callback::new(move |_| {
        // Emergency kill switch trigger
        emergency_active.set(true);
    });

    let primary = view! {
        <div class="flex flex-col gap-4">
            <div class="p-4 rounded border border-border-soft bg-surface-base">
                <h3 class="text-sm font-semibold text-text-primary mb-2">"Active Gateway Connections"</h3>
                <div class="grid grid-cols-2 gap-3 text-xs text-text-muted">
                    <div class="p-3 bg-surface-subtle rounded border border-border-soft">
                        <span class="font-medium text-text-primary">"NSE Fix Gateway 01"</span>
                        <div class="text-semantic-success mt-1">"Connected • 1.2ms latency"</div>
                    </div>
                    <div class="p-3 bg-surface-subtle rounded border border-border-soft">
                        <span class="font-medium text-text-primary">"BSE Fix Gateway 02"</span>
                        <div class="text-semantic-success mt-1">"Connected • 1.4ms latency"</div>
                    </div>
                </div>
            </div>

            <div class="p-4 rounded border border-border-soft bg-surface-base">
                <h3 class="text-sm font-semibold text-text-primary mb-2">"Audit & Execution Log"</h3>
                <div class="text-xs text-text-muted font-mono bg-surface-subtle p-3 rounded border border-border-soft space-y-1">
                    <div>"[10:04:12] Gateway HEARTBEAT ack received seq=84912"</div>
                    <div>"[10:04:15] Circuit limits within nominal bounds"</div>
                    <div>"[10:04:18] Margin health check OK (utilization: 24.9%)"</div>
                </div>
            </div>
        </div>
    }
    .into_any();

    let rail = view! {
        <div class="p-4 rounded border border-semantic-error/40 bg-semantic-error/5 flex flex-col gap-4">
            <h3 class="text-sm font-bold text-semantic-error">"Emergency Controls & Kill-Switches"</h3>
            <p class="text-xs text-text-muted">
                "Instantly cancel all open active orders across all connected exchange gateways and pause algorithm routing."
            </p>
            <HfButton
                variant=ButtonVariant::Danger
                on_click=cancel_all
                class="w-full font-bold"
            >
                {move || if emergency_active.get() { "KILL SWITCH ACTIVATED" } else { "EMERGENCY CANCEL ALL ORDERS" }}
            </HfButton>
        </div>
    }
    .into_any();

    view! {
        <HfPage
            title="Operations & Kill-Switch Console"
            subtitle="Exchange gateway status, circuit health, and fail-safe liquidation controls"
        >
            <HfTwoColumnSplitLayout primary=primary rail=rail />
        </HfPage>
    }
}
