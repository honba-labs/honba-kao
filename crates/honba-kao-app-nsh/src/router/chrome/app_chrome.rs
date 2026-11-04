//! `AppChrome` — the persistent app shell: the workstation chrome on
//! desktop (`docs/workstation-chrome-instruction.md`: the 54 pt
//! [`AppChrome`]'s NavRail, the 44 pt HeaderBar and the full-bleed
//! viewport that replaced the pool's 264 pt/64 pt showcase shell from
//! 700 pt) and the pool's responsive phone stack below it.
//!
//! What this file owns:
//!
//! - the global `Ctrl+G` (GoBar) and `Ctrl+J` (AI analyst) hotkeys via
//!   NSH's native [`on_key`]; the HeaderBar's buttons toggle the same
//!   signals through the [`app_shell::WorkspaceHotkeys`] global (its
//!   builder slots cannot `use_context`);
//! - the GoBar and AI-analyst overlays floating *over* the shell (they
//!   are absolute siblings of the shell region, so they survive layout
//!   switches);
//! - routing the GoBar's destination to the shell entry
//!   ([`app_shell::open_path`]);
//! - publishing the client context's connection status into the
//!   [`app_shell::ChromeStatus`] global the workstation's HeaderBar
//!   LEDs render;
//! - the persistent [`SiteFooter`] status bar below the shell.
//!
//! Browser `window.history`/popstate sync is gone on purpose: the nav
//! stack + storage persistence replace deep links (Phase A.1's "abstract
//! route stack without `window`").

use crate::router::chrome::app_shell;
use crate::router::chrome::site_footer::{SiteFooter, SiteFooterProps};
use crate::shared::contexts::HonbaClientContext;
use honba_kao_ui_core::molecules::quant::agent_console::{
    AgentThoughtStep, HfAgentConsole, HfAgentConsoleEvent, HfAgentConsoleProps,
};
use honba_kao_ui_core::molecules::quant::go_bar::{HfGoBar, HfGoBarEvent, HfGoBarProps};
use honba_kao_ui_core::templates::shell::desktop as hf_desktop;
use honba_kao_ui_core::theme::tokens::HonbaStyle;
use nsh_core::prelude::*;

/// Autocomplete entries for [`HfGoBar`], matching the destinations
/// [`navigate_target`] understands (mirrors the original Leptos
/// `on_navigate` match's recognized tokens).
const GOBAR_AUTOCOMPLETE: &[&str] = &[
    "TRADING",
    "TRADE",
    "FNO",
    "OPTIONS",
    "PORTFOLIO",
    "WEALTH",
    "MF",
    "QUANT",
    "SIM",
    "SCREENER",
    "RESEARCH",
    "RISK",
];

/// The destination path for a GoBar selection or typed query.
fn navigate_target(target: &str) -> &'static str {
    match target {
        "TRADING" | "TRADE" => "/trading",
        "FNO" | "OPTIONS" => "/derivatives/option-chain",
        "PORTFOLIO" => "/portfolio",
        "WEALTH" | "MF" => "/wealth/mutual-funds",
        "QUANT" | "SIM" => "/quant/simulator",
        "SCREENER" => "/screener",
        "RESEARCH" => "/research/security",
        "RISK" => "/risk",
        _ => "/trading",
    }
}

/// Props for [`AppChrome`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct AppChromeProps {}

/// The persistent application shell.
#[component]
pub fn AppChrome(_props: AppChromeProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let client = use_context::<HonbaClientContext>().expect("client context");

    // Restores the entry open when the app last ran, then keeps it fresh.
    app_shell::restore();

    // The workstation chrome's mirror of the restored selection.
    app_shell::seed_selection();

    // The workstation's HeaderBar LEDs render the connection status
    // from the ChromeStatus global (fn pointers and builder slots
    // cannot use_context); this subscription keeps it live.
    effect(move || {
        app_shell::install_status(
            client.oms_connected.get(),
            client.feed_connected.get(),
            client.latency_ms.get(),
        );
    });

    let gobar_open = signal(false);
    let agent_open = signal(false);

    // The HeaderBar's GoBar/Analyst buttons get their openers from the
    // chrome global (the builders cannot use_context).
    app_shell::install_hotkeys(gobar_open, agent_open);

    // The console's step list: the user's prompts (kept locally; there
    // is no prompt endpoint yet) merged with the live `agent_thoughts`
    // feed the context's SSE agent stream appends to. `AgentStep` (the
    // client stream's DTO) and `AgentThoughtStep` (the console's prop
    // item) are field-for-field identical, so the map is a rename; the
    // newest row goes first, matching the streaming rail's top-down
    // read.
    let user_steps = signal(Vec::<AgentThoughtStep>::new());
    let live_steps = client.agent_thoughts;
    let thoughts = Prop::bind(move || {
        let mut steps: Vec<AgentThoughtStep> = live_steps
            .get()
            .iter()
            .map(|step| AgentThoughtStep {
                agent_name: step.agent_name.clone(),
                step_type: step.step_type.clone(),
                content: step.content.clone(),
                timestamp: step.timestamp.clone(),
            })
            .collect();
        steps.extend(user_steps.get().iter().cloned());
        steps.reverse();
        steps
    });
    // The whiteboard's streaming flag: the feed is live while the OMS
    // bridge is (one session, one SSE connection per richer runtime).
    let is_streaming = client.oms_connected;

    // Global hotkeys: Ctrl+G toggles GoBar, Ctrl+J toggles the Agent
    // Console, regardless of focus.
    on_key("ctrl+g", move |_| gobar_open.update(|v| *v = !*v));
    on_key("ctrl+j", move |_| agent_open.update(|v| *v = !*v));

    let go_bar = HfGoBar::element(HfGoBarProps {
        open: Prop::from(gobar_open),
        autocomplete: Prop::from(
            GOBAR_AUTOCOMPLETE
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        ),
    })
    .on_event(move |event: &HfGoBarEvent| match event {
        HfGoBarEvent::Close => gobar_open.set(false),
        HfGoBarEvent::Navigate(target) => app_shell::open_path(navigate_target(target)),
    });

    let agent_console = HfAgentConsole::element(HfAgentConsoleProps {
        open: Prop::from(agent_open),
        thoughts,
        is_streaming: Prop::from(is_streaming),
    })
    .on_event(move |event: &HfAgentConsoleEvent| match event {
        HfAgentConsoleEvent::Close => agent_open.set(false),
        HfAgentConsoleEvent::SubmitPrompt(prompt) => {
            let prompt = prompt.clone();
            user_steps.update(|list| {
                list.push(AgentThoughtStep {
                    agent_name: "User".to_string(),
                    step_type: "thought".to_string(),
                    content: prompt,
                    timestamp: "Just now".to_string(),
                });
            });
        }
    });

    view()
        .flex_col()
        .w_full()
        .h_full()
        .overflow_hidden()
        .surface_base()
        .text_high()
        .child(
            // The responsive dual shell with the overlays floating over
            // it: the desktop region remounts when the window crosses
            // the 700 pt threshold, so what is open carries across (the
            // pool's spec handles the storage).
            view()
                .flex_1()
                .flex_col()
                .relative()
                .overflow_hidden()
                .child(
                    when(hf_desktop, move || {
                        app_shell::desktop_layout().into_element()
                    })
                    .otherwise(move || app_shell::phone().into_element()),
                )
                .child(go_bar)
                .child(agent_console),
        )
        .child(SiteFooter::element(SiteFooterProps::default()))
}
