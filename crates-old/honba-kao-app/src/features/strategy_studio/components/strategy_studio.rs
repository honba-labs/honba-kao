//! Strategy studio body: the strategy library. Mirrors
//! `web/src/pages/StrategyStudioPage.tsx` + `components/strategy/`.
//!
//! Two shelves: the saved-strategy library rows (`Open` lands on the
//! template graph each row is authored from; a row without a graph says so
//! rather than opening a placeholder) and fe's six starting templates
//! (`STRATEGY_TEMPLATES`). `+ New strategy` opens the blank graph.

use leptos::prelude::*;

use crate::features::strategy_studio::kernel::templates::{library_template, strategy_templates};
use crate::features::strategy_studio::stores::StudioStore;
use crate::shared::api;
use crate::shared::utils::percent;
use honba_kao_ui_core::{HfButton, HfButtonSize, HfButtonVariant, HfPanel, HfPanelHeader};

#[component]
pub fn StrategyStudio(
    /// The studio store; opening a strategy flips the page to the editor.
    store: StudioStore,
) -> impl IntoView {
    let strategies = api::strategies::list_strategies();
    let count = strategies.len();
    let templates = strategy_templates();
    let template_count = templates.len();

    view! {
        <HfPanel>
            <HfPanelHeader title="Strategy library" counter=count>
                <HfButton
                    variant=HfButtonVariant::Green
                    size=HfButtonSize::Small
                    on_click=Callback::new(move |_| store.new_strategy())
                >
                    "+ New strategy"
                </HfButton>
            </HfPanelHeader>

            <div class="hfui-panel__content strategy-grid">
                {strategies
                    .into_iter()
                    .map(|s| {
                        let graph = library_template(s.id);
                        view! {
                            <div class="strategy-card">
                                <div class="strategy-card__head">
                                    <strong>{s.name}</strong>
                                    <span class="tnum muted">{percent(s.win_rate)}</span>
                                </div>
                                <p class="strategy-card__desc">{s.description}</p>
                                <div class="strategy-card__foot">
                                    <span class="muted small">
                                        {format!("by {}", s.author)}
                                        {match graph {
                                            Some(name) => format!(" · graph {name}"),
                                            None => " · no graph yet".to_string(),
                                        }}
                                    </span>
                                    <HfButton
                                        variant=HfButtonVariant::Ghost
                                        size=HfButtonSize::Small
                                        disabled={graph.is_none()}
                                        title={
                                            if graph.is_some() {
                                                "Open in the node editor"
                                            } else {
                                                "This strategy has no graph to open yet"
                                            }
                                        }
                                        on_click=Callback::new(move |_| {
                                            if let Some(name) = graph {
                                                store.open_template(name);
                                            }
                                        })
                                    >
                                        "Open"
                                    </HfButton>
                                </div>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        </HfPanel>

        <HfPanel>
            <HfPanelHeader title="Starting templates" counter=template_count />
            <div class="hfui-panel__content strategy-grid">
                {templates
                    .into_iter()
                    .map(|t| {
                        let name = t.name;
                        let nodes = t.graph.nodes.len();
                        view! {
                            <div class="strategy-card">
                                <div class="strategy-card__head">
                                    <strong>{t.label}</strong>
                                    <span class="studio-tag">{t.category}</span>
                                </div>
                                <p class="strategy-card__desc">{t.description}</p>
                                <div class="strategy-card__foot">
                                    <span class="muted small tnum">{format!("{nodes} nodes · {name}")}</span>
                                    <HfButton
                                        variant=HfButtonVariant::Ghost
                                        size=HfButtonSize::Small
                                        on_click=Callback::new(move |_| {
                                            store.open_template(name);
                                        })
                                    >
                                        "Use template"
                                    </HfButton>
                                </div>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        </HfPanel>
    }
}
