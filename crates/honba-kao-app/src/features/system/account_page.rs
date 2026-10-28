use honba_kao_ui_core::primitive::{ButtonVariant, HfButton, HfInput};
use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn AccountPage() -> impl IntoView {
    let username = RwSignal::new("AlphaDesk_Prime".to_string());
    let broker_id = RwSignal::new("BRK-990214".to_string());
    let webhook_url =
        RwSignal::new("https://oms.institutional-fund.internal/hooks/fills".to_string());

    view! {
        <HfPage
            title="Institutional Account & Settings"
            subtitle="Trader identity, exchange gateway credentials, API keys, and notification webhooks"
        >
            <div class="max-w-3xl flex flex-col gap-6">
                <div class="p-6 rounded-lg border border-border-soft bg-surface-base">
                    <h3 class="text-sm font-semibold text-text-primary mb-4">"Profile & Broker Allocation"</h3>
                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label class="text-xs text-text-muted mb-1 block">"Trader Alias"</label>
                            <HfInput value=username />
                        </div>
                        <div>
                            <label class="text-xs text-text-muted mb-1 block">"Broker Account Reference"</label>
                            <HfInput value=broker_id />
                        </div>
                    </div>
                </div>

                <div class="p-6 rounded-lg border border-border-soft bg-surface-base">
                    <h3 class="text-sm font-semibold text-text-primary mb-4">"Real-time Execution Webhook"</h3>
                    <p class="text-xs text-text-muted mb-3">
                        "Fills and risk circuit violations are dispatched immediately to your internal ingestion endpoint."
                    </p>
                    <HfInput value=webhook_url />
                </div>

                <div class="flex justify-end gap-3">
                    <HfButton variant=ButtonVariant::Ghost>
                        "Reset Changes"
                    </HfButton>
                    <HfButton variant=ButtonVariant::Primary>
                        "Save Preferences"
                    </HfButton>
                </div>
            </div>
        </HfPage>
    }
}
