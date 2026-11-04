//! `HfPanel`, `HfPanelHeader` — dockable panel container with title bar.
//! Ported from `secured-finance-app` atom `Section` / `SectionWithItems`.

use leptos::prelude::*;

#[component]
pub fn HfPanel(
    #[prop(into, optional)] header: Option<String>,
    #[prop(into, optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let base = "hf-panel flex flex-col rounded-lg border border-border-soft bg-surface-base overflow-hidden";
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    view! {
        <div class=classes>
            {header.map(|title| view! {
                <div class="hf-panel-header px-3 py-2 border-b border-border-soft bg-surface-subtle">
                    <HfPanelHeader title />
                </div>
            })}
            <div class="hf-panel-body flex-grow p-3 overflow-auto">{children()}</div>
        </div>
    }
}

/// Structured card content section header with optional subtitle.
#[component]
pub fn HfPanelHeader(
    title: String,
    #[prop(into, optional)] subtitle: Option<String>,
) -> impl IntoView {
    view! {
        <div class="hf-panel-header-inner flex flex-col gap-0.5">
            <span class="text-xs font-semibold text-text-high tracking-tight uppercase">{title}</span>
            {subtitle.map(|s| view! { <p class="text-[10px] text-text-muted">{s}</p> })}
        </div>
    }
}

/// Section hosting a grid of key-value parameter cells.
#[component]
pub fn HfSectionGrid(children: Children) -> impl IntoView {
    view! {
        <div class="hf-section-grid grid grid-cols-2 gap-2 text-xs">{children()}</div>
    }
}
