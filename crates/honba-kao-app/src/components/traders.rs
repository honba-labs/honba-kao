//! Traders Overview panel modeled after bfx-hf-ui-core trading state panels.

use crate::ui::{
    ButtonVariant, HfBadge, HfButton, HfPLNumber, HfPanel, HfPanelHeader, TraderState,
};
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct TraderItem {
    pub id: &'static str,
    pub name: &'static str,
    pub strategy: &'static str,
    pub state: TraderState,
    pub pnl: f64,
    pub win_rate: f64,
}

#[component]
pub fn TradersOverviewPanel() -> impl IntoView {
    let traders = RwSignal::new(vec![
        TraderItem {
            id: "t-01",
            name: "Nifty Momentum",
            strategy: "Trend Following (EMA 20/50)",
            state: TraderState::Running,
            pnl: 14850.50,
            win_rate: 68.5,
        },
        TraderItem {
            id: "t-02",
            name: "BankNifty MeanRev",
            strategy: "Bollinger Squeeze",
            state: TraderState::Running,
            pnl: -3200.00,
            win_rate: 52.0,
        },
        TraderItem {
            id: "t-03",
            name: "IT Breakout",
            strategy: "Volume Expansion",
            state: TraderState::Paused,
            pnl: 8420.00,
            win_rate: 61.2,
        },
    ]);

    view! {
        <HfPanel>
            <HfPanelHeader title="Active Traders" counter=3>
                <HfButton variant=ButtonVariant::Green small=true>
                    "+ New"
                </HfButton>
            </HfPanelHeader>

            <div class="hfui-panel__content" style="display: flex; flex-direction: column; gap: 8px;">
                {move || traders.get().into_iter().map(|trader| {
                    view! {
                        <div style="background: var(--background-color); border: 1px solid var(--border-color); border-radius: 5px; padding: 10px;">
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;">
                                <strong style="color: var(--shade-9); font-size: 13px;">{trader.name}</strong>
                                <HfBadge state=trader.state />
                            </div>

                            <div style="font-size: 11px; color: var(--shade-5); margin-bottom: 8px;">
                                {trader.strategy}
                            </div>

                            <div style="display: flex; justify-content: space-between; align-items: center; font-size: 12px; margin-bottom: 10px;">
                                <span>"Session PnL:"</span>
                                <HfPLNumber value=trader.pnl is_currency=true />
                            </div>

                            <div style="display: flex; gap: 6px; justify-content: flex-end;">
                                <HfButton
                                    variant=if trader.state == TraderState::Running { ButtonVariant::Default } else { ButtonVariant::Green }
                                    small=true
                                >
                                    {if trader.state == TraderState::Running { "Pause" } else { "Resume" }}
                                </HfButton>
                                <HfButton variant=ButtonVariant::Red small=true>
                                    "Stop"
                                </HfButton>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </HfPanel>
    }
}
