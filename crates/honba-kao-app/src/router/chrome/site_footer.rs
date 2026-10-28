use crate::shared::contexts::HonbaClientContext;
use leptos::prelude::*;

#[component]
pub fn SiteFooter() -> impl IntoView {
    let client = use_context::<HonbaClientContext>().unwrap_or_else(fallback_client_context);

    let oms_status = move || {
        if client.oms_connected.get() {
            "OMS: CONNECTED"
        } else {
            "OMS: DISCONNECTED"
        }
    };
    let oms_class = move || {
        if client.oms_connected.get() {
            "text-semantic-success"
        } else {
            "text-semantic-error"
        }
    };
    let latency = move || match client.latency_ms.get() {
        Some(ms) => format!("LATENCY: {ms}ms"),
        None => "LATENCY: --".to_string(),
    };
    let feed_ticks = move || {
        if client.feed_connected.get() {
            "FEED: LIVE"
        } else {
            "FEED: IDLE"
        }
    };

    view! {
        <footer class="site-footer flex items-center justify-between px-4 py-1.5 bg-surface-subtle border-t border-border-soft text-[11px] text-text-muted font-mono select-none">
            <div class="flex items-center gap-3">
                <div class="flex items-center gap-1.5">
                    <span
                        class="w-1.5 h-1.5 rounded-full"
                        class:bg-emerald-400=move || client.oms_connected.get()
                        class:bg-rose-500=move || !client.oms_connected.get()
                    ></span>
                    <span class=oms_class>{oms_status}</span>
                </div>
                <span>"|"</span>
                <span class="text-text-muted">{latency}</span>
                <span>"|"</span>
                <span class="text-text-muted">{feed_ticks}</span>
            </div>
            <div class="flex items-center gap-3">
                <a href="/components.html" target="_blank" rel="noopener" class="hover:text-text-high">
                    "Components"
                </a>
                <span>"|"</span>
                <span>"HONBA-KAO v3.0 Institutional"</span>
            </div>
        </footer>
    }
}

/// Fallback client context when `HonbaClientContext` has not been provided
/// (e.g. a standalone preview of the footer).
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
