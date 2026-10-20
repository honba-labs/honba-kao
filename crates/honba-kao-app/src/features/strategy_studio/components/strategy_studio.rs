//! Strategy studio body: the strategy library. Mirrors
//! `web/src/pages/StrategyStudioPage.tsx` + `components/strategy/`.

use leptos::prelude::*;

use crate::shared::api;
use honba_kao_ui_core::{HfButton, HfButtonSize, HfButtonVariant, HfPanel, HfPanelHeader};
use crate::shared::utils::percent;

#[component]
pub fn StrategyStudio() -> impl IntoView {
    let strategies = api::strategies::list_strategies();
    let count = strategies.len();

    view! {
        <HfPanel>
            <HfPanelHeader title="Strategy library" counter=count>
                <HfButton variant=HfButtonVariant::Green size=HfButtonSize::Small>"+ New strategy"</HfButton>
            </HfPanelHeader>

            <div class="hfui-panel__content strategy-grid">
                {strategies
                    .into_iter()
                    .map(|s| {
                        view! {
                            <div class="strategy-card">
                                <div class="strategy-card__head">
                                    <strong>{s.name}</strong>
                                    <span class="tnum muted">{percent(s.win_rate)}</span>
                                </div>
                                <p class="strategy-card__desc">{s.description}</p>
                                <div class="strategy-card__foot">
                                    <span class="muted small">{format!("by {}", s.author)}</span>
                                    <HfButton variant=HfButtonVariant::Ghost size=HfButtonSize::Small>"Open"</HfButton>
                                </div>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        </HfPanel>
    }
}
