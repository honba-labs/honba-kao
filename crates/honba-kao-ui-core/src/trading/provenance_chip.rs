//! Provenance indicator: where a number came from and how fresh it is.
//!
//! Port of OpenTerminalUI `common/ProvenanceChip.tsx`. The quality set is
//! `Live / Delayed / Cached / Synthetic` plus `Unavailable` (rendered as
//! `NO DATA`), as in the OpenTerminalUI wire contract.

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProvenanceQuality {
    Live,
    Delayed,
    Cached,
    Synthetic,
    Unavailable,
}

impl ProvenanceQuality {
    /// Chip text and CSS tone, as in the OpenTerminalUI chip.
    pub fn label(self) -> &'static str {
        match self {
            Self::Live => "LIVE",
            Self::Delayed => "DELAYED",
            Self::Cached => "CACHED",
            Self::Synthetic => "SYNTHETIC",
            Self::Unavailable => "NO DATA",
        }
    }

    pub fn tone(self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::Delayed => "info",
            Self::Cached => "neutral",
            Self::Synthetic => "mock",
            Self::Unavailable => "warn",
        }
    }
}

#[component]
pub fn HfProvenanceChip(
    quality: ProvenanceQuality,
    /// Feed or source name, shown after the label unless `compact`.
    #[prop(optional, into)]
    source: String,
    #[prop(optional)] compact: bool,
    /// Tooltip text, e.g. "as of 14:31:02 · 42ms".
    #[prop(optional, into)]
    title: String,
) -> impl IntoView {
    let label = if compact || source.is_empty() {
        quality.label().to_string()
    } else {
        format!("{} \u{00B7} {}", quality.label(), source.to_uppercase())
    };
    view! {
        <span
            class=format!("hf-provenance hf-provenance--{}", quality.tone())
            data-quality=quality.label().to_lowercase()
            title=title
        >
            <span class="hf-provenance__dot" aria-hidden="true"></span>
            {label}
        </span>
    }
}
