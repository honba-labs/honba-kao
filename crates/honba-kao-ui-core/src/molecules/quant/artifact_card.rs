//! `HfAiArtifactCard` — interactive card rendering AI-generated structured
//! artifacts (valuation matrices, thesis summaries, scenario tables). OpenTerminalUI
//! `AiVisuals` equivalent.

use leptos::prelude::*;

#[component]
pub fn HfAiArtifactCard(
    #[prop(into)] title: String,
    #[prop(into)] content: String,
    #[prop(into, optional)] class: Option<String>,
) -> impl IntoView {
    let base = "hf-ai-artifact-card rounded-lg border border-border-soft bg-surface-base shadow-sm flex flex-col";
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    view! {
        <div class=classes>
            <div class="hf-artifact-header px-3 py-2 border-b border-border-soft flex items-center justify-between">
                <span class="text-xs font-semibold text-text-high">{title}</span>
                <span class="hf-kbd-badge text-[10px] bg-surface-subtle text-text-muted">"AI"</span>
            </div>
            <div class="hf-artifact-body p-3 whitespace-pre-wrap break-words text-xs text-text-high">{content}</div>
        </div>
    }
}
