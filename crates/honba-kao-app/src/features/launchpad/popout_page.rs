//! Pop-out window for one Launchpad panel. Rendered outside `AppShell`, so it
//! is a bare full-bleed panel. Also listens on the `honba-launchpad`
//! BroadcastChannel; received messages are only logged for now.

use honba_kao_ui_core::{HfPanel, HfPanelHeader};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::MessageEvent;

use super::panel_title;

/// Channel name shared by the Launchpad windows.
const CHANNEL: &str = "honba-launchpad";

#[component]
pub fn PopoutPage() -> impl IntoView {
    let query = use_query_map();
    let title = move || {
        query
            .with(|params| params.get("panel"))
            .as_deref()
            .and_then(panel_title)
            .unwrap_or("Unknown panel")
    };

    // Plumbing only: log what arrives. The channel and listener live as long
    // as this window; closing the window tears them down with it.
    match web_sys::BroadcastChannel::new(CHANNEL) {
        Ok(channel) => {
            let on_message = Closure::<dyn FnMut(MessageEvent)>::new(|event: MessageEvent| {
                web_sys::console::log_1(&event.data());
            });
            channel.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
            on_message.forget();
            std::mem::forget(channel);
        }
        Err(err) => web_sys::console::warn_1(&err),
    }

    view! {
        <div class="launchpad-popout">
            <HfPanel class="launchpad__slot launchpad__slot--popout">
                <HfPanelHeader title=title() />
                <div class="launchpad__body placeholder__note">
                    {move || format!("{} · mock data", title())}
                </div>
            </HfPanel>
        </div>
    }
}
