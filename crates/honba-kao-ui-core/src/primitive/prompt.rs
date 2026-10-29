//! `HfPromptSymbol`, `HfStatusLed`, `HfThinkingOrb` — prompt primitives.

use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfPromptSymbol`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfPromptSymbolProps {
    /// The prompt glyph (e.g. `>_`).
    pub symbol: Prop<String>,
}

/// A symbol prompt (search / command input chip).
#[component]
pub fn HfPromptSymbol(props: HfPromptSymbolProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let s = props.symbol.get_untracked();

    view()
        .px(px(1.5))
        .py(px(0.5))
        .rounded(px(6.0))
        .surface_muted()
        .border_soft()
        .text_size(px(11.0))
        .text_high()
        .child(text(s))
}

/// Status LED tones.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PropEnum)]
pub enum StatusTone {
    #[default]
    Neutral,
    Ok,
    Warning,
    Critical,
    Offline,
    Info,
}

/// Props for [`HfStatusLed`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfStatusLedProps {
    #[prop(default)]
    pub tone: Prop<Option<StatusTone>>,
}

/// A status LED (pulsing dot indicator).
#[component]
pub fn HfStatusLed(props: HfStatusLedProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let tone_val = props.tone.get_untracked().unwrap_or_default();
    let color = match tone_val {
        StatusTone::Ok => rgb(0x10b981),
        StatusTone::Warning => rgb(0xf59e0b),
        StatusTone::Critical => rgb(0xef4444),
        StatusTone::Offline => rgb(0x64748b),
        StatusTone::Info => rgb(0x3b82f6),
        StatusTone::Neutral => rgb(0x94a3b8),
    };

    view().w(px(2.0)).h(px(2.0)).rounded_full().bg(color)
}

/// Props for [`HfThinkingOrb`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfThinkingOrbProps {
    #[prop(default = 24.0)]
    pub size: Prop<f32>,
}

/// A thinking orb (animated loading indicator).
#[component]
pub fn HfThinkingOrb(props: HfThinkingOrbProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let sz = props.size.get_untracked();

    view()
        .w(px(sz))
        .h(px(sz))
        .rounded_full()
        .border(px(2.0))
        .border_color(rgb(0x232a3a))
        .border_t_color(rgb(0x10b981))
        .child(text(""))
}
