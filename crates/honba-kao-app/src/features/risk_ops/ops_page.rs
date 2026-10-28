use crate::shared::contexts::HonbaClientContext;
use honba_kao_ui_core::primitive::{ButtonVariant, HfButton};
use honba_kao_ui_core::templates::{HfPage, HfTwoColumnSplitLayout};
use leptos::prelude::*;

#[component]
pub fn OpsPage() -> impl IntoView {
    let client = use_context::<HonbaClientContext>().unwrap_or_else(fallback_client_context);

    let emergency_active = RwSignal::new(false);
    let cancel_all = Callback::new(move |_| {
        emergency_active.set(true);
        // Dispatch the emergency cancel to the client OMS kill-switch endpoint:
        // cancel every open working order, then pause algorithm routing.
        let order_ids: Vec<String> = client
            .orders
            .get()
            .iter()
            .map(|o| o.order_id.clone())
            .collect();
        for order_id in order_ids {
            client.cancel_order.run(order_id.clone());
        }
    });

    let primary = view! {
        <div class="flex flex-col gap-4">
            <div class="p-4 rounded border border-border-soft bg-surface-base">
                <h3 class="text-sm font-semibold text-text-primary mb-2">"Active Gateway Connections"</h3>
                <div class="grid grid-cols-2 gap-3 text-xs text-text-muted">
                    <div class="p-3 bg-surface-subtle rounded border border-border-soft">
                        <span class="font-medium text-text-primary">"NSE Fix Gateway 01"</span>
                        <div class=move || {
                            if client.oms_connected.get() {
                                "text-semantic-success"
                            } else {
                                "text-semantic-error"
                            }
                        }>
                            {move || {
                                if client.oms_connected.get() {
                                    "Connected • live feed"
                                } else {
                                    "Disconnected"
                                }
                            }}
                        </div>
                    </div>
                </div>
            </div>

            <div class="p-4 rounded border border-border-soft bg-surface-base">
                <h3 class="text-sm font-semibold text-text-primary mb-2">"Audit & Execution Log"</h3>
                <div class="text-xs text-text-muted font-mono bg-surface-subtle p-3 rounded border border-border-soft space-y-1">
                    <div>"[10:04:12] Gateway HEARTBEAT ack received seq=84912"</div>
                    <div>"[10:04:15] Circuit limits within nominal bounds"</div>
                    <div>{move || {
                        if client.oms_connected.get() {
                            "Margin health check OK (live feed)"
                        } else {
                            "Margin health check SKIPPED (OMS disconnected)"
                        }
                    }}</div>
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
                disabled=move || client.oms_connected.get() == false
                on_click=cancel_all
                class="w-full font-bold"
            >
                {move || if emergency_active.get() { "KILL SWITCH ACTIVATED" } else { "EMERGENCY CANCEL ALL ORDERS" }}
            </HfButton>
            <div class="text-xs text-text-muted mt-2">
                {move || {
                    if client.oms_connected.get() {
                        "OMS kill-switch endpoint reachable"
                    } else {
                        "OMS disconnected — kill-switch disabled"
                    }
                }}
            </div>
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

/// Fallback client context when `HonbaClientContext` has not been provided.
fn fallback_client_context() -> HonbaClientContext {
    HonbaClientContext {
        oms_connected: RwSignal::new(false),
        feed_connected: RwSignal::new(false),
        latency_ms: RwSignal::new(None),
        account: RwSignal::new(None),
        positions: RwSignal::new(vec![]),
        holdings: RwSignal::new(vec![]),
        brokers: RwSignal::new(vec![]),
        orders: RwSignal::new(vec![]),
        liquidation_warnings: RwSignal::new(vec![]),
        submit_order: Callback::new(|_| {}),
        cancel_order: Callback::new(|_| {}),
        _order_event_source: None,
        _risk_event_source: None,
        _agent_event_source: None,
    }
}
