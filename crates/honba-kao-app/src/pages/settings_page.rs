//! Settings page. Mirrors `web/src/pages/SettingsPage.tsx`.

use leptos::prelude::*;

use crate::api;
use honba_kao_ui_core::{HfPageHeader, HfPanel, HfPanelHeader};
use crate::constants::branding;
use crate::stores::trading_mode::{TradingMode, use_trading_mode};

#[component]
pub fn SettingsPage() -> impl IntoView {
    let config = api::config::get_system_config();
    let telegram = api::telegram::get_status();
    // The paper toggle is the same state the header's PAPER|LIVE indicator
    // shows — one source of truth, persisted by the shared store.
    let trading = use_trading_mode();
    let paper = trading.is_paper();
    let notify = RwSignal::new(false);

    view! {
        <div class="console">
            <HfPageHeader title="Settings" subtitle="Runtime, trading and notifications" />

            <HfPanel>
                <HfPanelHeader title="Runtime" />
                <div class="hfui-panel__content">
                    <div class="summary-row">
                        <span>"Version"</span>
                        <strong class="mono">{config.version}</strong>
                    </div>
                    <div class="summary-row">
                        <span>"Registration"</span>
                        <strong>
                            {if config.registration_enabled { "enabled" } else { "closed" }}
                        </strong>
                    </div>
                    <div class="summary-row">
                        <span>"Telegram alerts"</span>
                        <strong>
                            {if telegram.configured { "configured" } else { "not configured" }}
                        </strong>
                    </div>
                </div>
            </HfPanel>

            <HfPanel>
                <HfPanelHeader title="Trading" />
                <div class="hfui-panel__content toggle-list">
                    <label class="toggle">
                        <input
                            type="checkbox"
                            prop:checked=move || paper.get()
                            on:change=move |_| {
                                let next = if paper.get() {
                                    TradingMode::Live
                                } else {
                                    TradingMode::Paper
                                };
                                trading.set(next);
                            }
                        />
                        <span>"Paper trading mode"</span>
                    </label>
                    <label class="toggle">
                        <input
                            type="checkbox"
                            prop:checked=move || notify.get()
                            on:change=move |_| notify.update(|v| *v = !*v)
                        />
                        <span>"Notify on every gate rejection"</span>
                    </label>
                </div>
            </HfPanel>

            <HfPanel>
                <HfPanelHeader title="About" />
                <div class="hfui-panel__content">
                    <p class="muted small">
                        {branding::APP_NAME}" · "{branding::APP_NAME_LATIN}" · v"{branding::VERSION}
                    </p>
                    <p class="muted small">
                        "Frontend structure mirrors "
                        <a href=branding::REPO_URL>"nofx/web"</a>
                        ", rendered with Leptos."
                    </p>
                </div>
            </HfPanel>
        </div>
    }
}
