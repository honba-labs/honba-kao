use super::{header_bar::HeaderBar, nav_rail::NavRail, site_footer::SiteFooter};
use honba_kao_ui_core::molecules::quant::{AgentThoughtStep, HfAgentConsole, HfGoBar};
use leptos::prelude::*;
use leptos_router::components::Outlet;

#[component]
pub fn AppChrome() -> impl IntoView {
    let gobar_open = RwSignal::new(false);
    let agent_open = RwSignal::new(false);
    let agent_streaming = RwSignal::new(false);
    let thoughts = RwSignal::new(Vec::<AgentThoughtStep>::new());

    let on_toggle_gobar = Callback::new(move |_| {
        gobar_open.update(|v| *v = !*v);
    });

    let on_toggle_agent = Callback::new(move |_| {
        agent_open.update(|v| *v = !*v);
    });

    let on_navigate = Callback::new(move |target: String| {
        let loc = web_sys::window().and_then(|w| w.location().href().ok());
        if let Some(_curr) = loc {
            let path = match target.as_str() {
                "TRADING" | "TRADE" => "/trading",
                "FNO" | "OPTIONS" => "/derivatives/option-chain",
                "PORTFOLIO" => "/portfolio",
                "WEALTH" | "MF" => "/wealth/mutual-funds",
                "QUANT" | "SIM" => "/quant/simulator",
                "SCREENER" => "/screener",
                "RESEARCH" => "/research/security",
                "RISK" => "/risk",
                _ => "/trading",
            };
            if let Some(w) = web_sys::window() {
                let _ = w.location().set_href(path);
            }
        }
    });

    let on_submit_prompt = Callback::new(move |prompt: String| {
        thoughts.update(|list| {
            list.push(AgentThoughtStep {
                agent_name: "User".to_string(),
                step_type: "thought".to_string(),
                content: prompt,
                timestamp: "Just now".to_string(),
            });
            list.push(AgentThoughtStep {
                agent_name: "Honba-Analyst".to_string(),
                step_type: "thought".to_string(),
                content: "Analyzing order flow, SPAN margin impact, and volatility smile..."
                    .to_string(),
                timestamp: "Just now".to_string(),
            });
        });
    });

    view! {
        <div class="app-chrome flex flex-col h-screen w-screen overflow-hidden bg-surface-base text-text-high">
            <HeaderBar
                on_toggle_gobar=on_toggle_gobar
                on_toggle_agent=on_toggle_agent
            />

            <div class="flex flex-grow overflow-hidden relative">
                <NavRail />

                <main class="flex-grow overflow-y-auto min-w-0 p-3 bg-surface-subtle">
                    <Outlet />
                </main>

                // Persistent Global GoBar (Ctrl+G)
                <HfGoBar
                    open=gobar_open
                    on_navigate=on_navigate
                />

                // Persistent Global Agent Research Console (Ctrl+J)
                <HfAgentConsole
                    open=agent_open
                    thoughts=Signal::derive(move || thoughts.get())
                    is_streaming=Signal::derive(move || agent_streaming.get())
                    on_submit_prompt=on_submit_prompt
                />
            </div>

            <SiteFooter />
        </div>
    }
}
