//! Competition leaderboard. Mirrors
//! `web/src/components/trader/CompetitionPage.tsx`.

use leptos::prelude::*;

use crate::components::ui::{HfPLNumber, HfPanel, HfPanelHeader, PageHeader};
use crate::data::demo_data::demo_traders;
use crate::hooks::use_interval;
use crate::utils::percent;

#[component]
pub fn CompetitionPage() -> impl IntoView {
    let tick = use_interval(30_000);

    let ranked = move || {
        let mut traders = demo_traders();
        traders.sort_by(|a, b| {
            b.pnl
                .partial_cmp(&a.pnl)
                .unwrap_or(core::cmp::Ordering::Equal)
        });
        traders
    };

    view! {
        <div class="console">
            <PageHeader title="Competition" subtitle="Paper-trading leaderboard, ranked by realised P&L">
                <span class="muted small">{move || format!("tick {}", tick.get())}</span>
            </PageHeader>

            <HfPanel>
                <HfPanelHeader title="Leaderboard" counter=ranked().len() />
                <div class="hfui-panel__content flush">
                    <table class="hfui-table">
                        <thead>
                            <tr>
                                <th>"#"</th>
                                <th>"Trader"</th>
                                <th>"Model"</th>
                                <th>"Win rate"</th>
                                <th>"P&L"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || ranked()
                                .into_iter()
                                .enumerate()
                                .map(|(i, t)| {
                                    view! {
                                        <tr>
                                            <td class="tnum">{i + 1}</td>
                                            <td><strong>{t.trader_name.clone()}</strong></td>
                                            <td class="muted small">{t.ai_model.clone()}</td>
                                            <td class="tnum">{percent(t.win_rate)}</td>
                                            <td><HfPLNumber value=t.pnl is_currency=true /></td>
                                        </tr>
                                    }
                                })
                                .collect_view()}
                        </tbody>
                    </table>
                </div>
            </HfPanel>
        </div>
    }
}
