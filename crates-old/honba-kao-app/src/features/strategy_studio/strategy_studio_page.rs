//! Strategy Studio — the strategy library and the node-graph editor
//! (HBD-2026-DES-006, through phase P2).
//!
//! The page owns a page-scoped [`StudioStore`] (the simulator page's
//! idiom): every child reads and writes the same signals, so the canvas,
//! inspector, issue list and generated source stay consistent through every
//! gesture. With nothing open the library shows (saved rows + fe's six
//! starting templates); opening one mounts the workbench — palette, canvas,
//! inspector, the grid & risk panel and the read-only generated Python.
//!
//! Later phases (not here yet): persistence of layout/library/versions
//! (P3), the editable code pane + server verify + simulator hand-off (P4),
//! run/sweeps (P5), co-pilot and research loop (P6–P8).

use leptos::prelude::*;

use crate::features::strategy_studio::canvas::{CanvasEditor, Inspector, Palette, Selection};
use crate::features::strategy_studio::components::{GridRiskPanel, StrategyStudio};
use crate::features::strategy_studio::stores::StudioStore;
use honba_kao_ui_core::HfPageHeader;

#[component]
pub fn StrategyStudioPage() -> impl IntoView {
    // Page-scoped: the store's memos root in this component's owner.
    let store = StudioStore::new();
    let open = store.open;

    view! {
        <div class="console studio">
            <HfPageHeader
                title="Strategy Studio"
                subtitle="Author strategy graphs on a node canvas · typed ports · generated Python"
            />
            <Show
                when=move || open.get()
                fallback=move || view! { <StrategyStudio store=store /> }
            >
                <StudioWorkbench store=store />
            </Show>
        </div>
    }
}

/// The open-strategy surface: toolbar, palette | canvas | side column, and
/// the validation issue list.
#[component]
fn StudioWorkbench(store: StudioStore) -> impl IntoView {
    let graph = store.graph;
    let issues = store.issues;
    let issue_count = Memo::new(move |_| issues.with(Vec::len));
    let hash_short = Memo::new(move |_| {
        store.hash.with(|h| {
            let hex = h.strip_prefix("sha256:").unwrap_or(h);
            format!("sha256:{}", &hex[..hex.len().min(12)])
        })
    });

    view! {
        <div class="studio-toolbar">
            <button
                type="button"
                class="studio-btn studio-btn--ghost"
                title="Back to the strategy library"
                on:click=move |_| store.close()
            >
                "← Library"
            </button>
            <span class="studio-toolbar__title">{move || graph.with(|g| g.name.clone())}</span>
            <span class="studio-toolbar__meta">{move || store.origin.get()}</span>
            <Show when=move || store.modified.get()>
                <span class="studio-toolbar__meta studio-toolbar__modified">"edited · not saved"</span>
            </Show>
            <span class="studio-toolbar__spacer"></span>
            <span class=move || {
                if issue_count.get() == 0 {
                    "studio-pill studio-pill--ok"
                } else {
                    "studio-pill studio-pill--bad"
                }
            }>
                {move || match issue_count.get() {
                    0 => "graph valid (local checks)".to_string(),
                    1 => "1 issue".to_string(),
                    n => format!("{n} issues"),
                }}
            </span>
            <span class="studio-toolbar__hash tnum" title="fe-compatible graph hash">
                {move || hash_short.get()}
            </span>
        </div>

        <div class="studio-workbench">
            <Palette store=store />
            <CanvasEditor store=store />
            <div class="studio-side">
                <Inspector store=store />
                <GridRiskPanel />
                <details class="studio-code">
                    <summary>"Generated Python (read-only)"</summary>
                    <pre class="studio-code__src">{move || store.source.get()}</pre>
                </details>
            </div>
        </div>

        <IssueList store=store />
    }
}

/// The local validation issues, in `graph_issues` order; a node-local
/// issue selects its node on click (fe's jump-to-node).
#[component]
fn IssueList(store: StudioStore) -> impl IntoView {
    view! {
        <section class="studio-issues" aria-label="Validation issues">
            <div class="studio-issues__head">
                "Validation"
                <span class="studio-issues__note">
                    "local structural checks · server verify lands with P4"
                </span>
            </div>
            {move || {
                let issues = store.issues.get();
                if issues.is_empty() {
                    return view! {
                        <div class="studio-issues__empty">"No structural issues."</div>
                    }
                        .into_any();
                }
                issues
                    .into_iter()
                    .map(|issue| {
                        let node = issue.node_id.clone().filter(|id| !id.is_empty());
                        let jump = node.clone();
                        view! {
                            <button
                                type="button"
                                class="studio-issue"
                                disabled={node.is_none()}
                                on:click=move |_| {
                                    if let Some(id) = jump.clone() {
                                        store.select(Selection::Node(id));
                                    }
                                }
                            >
                                <code class="studio-issue__code">{issue.code}</code>
                                <span class="studio-issue__msg">{issue.message}</span>
                            </button>
                        }
                    })
                    .collect_view()
                    .into_any()
            }}
        </section>
    }
}
