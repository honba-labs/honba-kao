//! `HfPromptSymbol`, `HfThinkingOrb`, `HfStatusLed` — terminal prompt / agent glyphs.
//! Ported from `secured-finance-app` atom `Identicon`-style procedural glyphs and
//! OpenTerminalUI `AiVisuals` thinking states.

use leptos::prelude::*;
use crate::icons::{SvgPrompt, SvgThinkingOrb};

/// Terminal prompt symbol (`>_`) rendered inside input rows (e.g. Go-bar).
#[component]
pub fn HfPromptSymbol() -> impl IntoView {
    view! {
        <span class="hf-prompt-symbol flex items-center gap-1 text-accent-primary text-xs font-mono">
            <SvgPrompt />
            <span class="hf-prompt-text text-text-high">">"</span>
        </span>
    }
}

/// Thinking / processing orb with optional active spin animation.
#[component]
pub fn HfThinkingOrb(
    #[prop(into, optional)] active: Option<Signal<bool>>,
) -> impl IntoView {
    let active = Signal::derive(move || active.as_ref().map(|a| a.get()).unwrap_or(false));
    view! {
        <span class=move || format!("hf-thinking-orb inline-block {}", if active.get() { "animate-spin" } else { "" } )>
            <SvgThinkingOrb />
        </span>
    }
}

/// Status led / dot for live mode, crosshair sync, broker latency.
#[component]
pub fn HfStatusLed(
    #[prop(default = false)] live: bool,
    #[prop(into, optional)] label: Option<String>,
) -> impl IntoView {
    let color = if live { "bg-emerald-500 box-shadow-[0_0_8px_rgba(16,185,129,0.6)]" } else { "bg-text-muted" };
    view! {
        <div class="hf-status-led flex items-center gap-1.5 text-[11px] text-text-muted">
            <span class=format!("hf-led-dot w-2 h-2 rounded-full {}", color)></span>
            {label.map(|l| view! { <span>{l}</span> })}
        </div>
    }
}
