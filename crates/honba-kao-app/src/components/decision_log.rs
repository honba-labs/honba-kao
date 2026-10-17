//! Decision Log audit panel.

use crate::ui::{HfPanel, HfPanelHeader};
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct DecisionItem {
    pub time: &'static str,
    pub strategy: &'static str,
    pub symbol: &'static str,
    pub action: &'static str,
    pub approved: bool,
    pub reason: &'static str,
}

#[component]
pub fn DecisionLogPanel() -> impl IntoView {
    let decisions = RwSignal::new(vec![
        DecisionItem {
            time: "15:24:02.114",
            strategy: "Nifty Momentum",
            symbol: "RELIANCE",
            action: "BUY 50 @ 2884.20",
            approved: true,
            reason: "PASSED (Risk limits OK)",
        },
        DecisionItem {
            time: "15:23:45.082",
            strategy: "BankNifty MeanRev",
            symbol: "SBIN",
            action: "SELL 200 @ 812.50",
            approved: false,
            reason: "MAX_DRAWDOWN_LIMIT (Breached ₹50,000)",
        },
        DecisionItem {
            time: "15:21:10.590",
            strategy: "IT Breakout",
            symbol: "WIPRO",
            action: "BUY 300 @ 540.00",
            approved: false,
            reason: "CIRCUIT_FILTER (Near upper band 5%)",
        },
        DecisionItem {
            time: "15:18:22.311",
            strategy: "Nifty Momentum",
            symbol: "HDFCBANK",
            action: "BUY 100 @ 1690.00",
            approved: true,
            reason: "PASSED (Risk limits OK)",
        },
    ]);

    view! {
        <HfPanel>
            <HfPanelHeader title="Decision Log & Risk Audit" counter=4>
                <span style="font-size: 11px; color: var(--shade-5);">
                    "Realtime Engine Stream"
                </span>
            </HfPanelHeader>

            <div class="hfui-panel__content" style="padding: 0;">
                <div style="display: flex; flex-direction: column;">
                    {move || decisions.get().into_iter().map(|item| {
                        let badge_color = if item.approved { "var(--pnl-gain)" } else { "var(--pnl-loss)" };
                        let badge_bg = if item.approved { "rgba(0, 224, 173, 0.12)" } else { "rgba(240, 83, 89, 0.12)" };

                        view! {
                            <div style="padding: 10px 12px; border-bottom: 1px solid var(--border-color); display: flex; flex-direction: column; gap: 4px;">
                                <div style="display: flex; justify-content: space-between; align-items: center;">
                                    <span class="tnum" style="color: var(--shade-5); font-size: 11px;">{item.time}</span>
                                    <span style=format!("font-size: 10px; font-weight: 700; padding: 2px 6px; border-radius: 100px; color: {}; background: {};", badge_color, badge_bg)>
                                        {if item.approved { "APPROVED" } else { "REJECTED" }}
                                    </span>
                                </div>

                                <div style="display: flex; justify-content: space-between; align-items: center;">
                                    <strong style="color: var(--shade-9); font-size: 12px;">{item.action}</strong>
                                    <span style="color: var(--shade-5); font-size: 11px;">{item.strategy}</span>
                                </div>

                                <div style="font-size: 11px; color: var(--shade-6); margin-top: 2px;">
                                    "Gate: "
                                    <span style="color: var(--shade-8); font-family: monospace;">{item.reason}</span>
                                </div>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>
        </HfPanel>
    }
}
