//! Signal matrix: per-strategy signal strength grid.

use leptos::prelude::*;

use crate::components::ui::{HfPanel, HfPanelHeader};

struct Signal {
    strategy: &'static str,
    symbol: &'static str,
    score: f64,
}

const SIGNALS: &[Signal] = &[
    Signal {
        strategy: "EMA Trend Rider",
        symbol: "RELIANCE",
        score: 0.82,
    },
    Signal {
        strategy: "EMA Trend Rider",
        symbol: "HDFCBANK",
        score: 0.44,
    },
    Signal {
        strategy: "Bollinger Squeeze",
        symbol: "INFY",
        score: -0.61,
    },
    Signal {
        strategy: "Volume Expansion",
        symbol: "TCS",
        score: 0.35,
    },
    Signal {
        strategy: "Volume Expansion",
        symbol: "SBIN",
        score: -0.28,
    },
];

fn tone(score: f64) -> &'static str {
    if score > 0.5 {
        "sig-strong-up"
    } else if score > 0.0 {
        "sig-up"
    } else if score > -0.5 {
        "sig-down"
    } else {
        "sig-strong-down"
    }
}

#[component]
pub fn SignalMatrix() -> impl IntoView {
    view! {
        <HfPanel>
            <HfPanelHeader title="Signal matrix" counter=SIGNALS.len() />

            <div class="hfui-panel__content flush">
                <table class="hfui-table">
                    <thead>
                        <tr>
                            <th>"Strategy"</th>
                            <th>"Symbol"</th>
                            <th>"Score"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {SIGNALS
                            .iter()
                            .map(|s| {
                                view! {
                                    <tr>
                                        <td class="muted small">{s.strategy}</td>
                                        <td><strong class="sym">{s.symbol}</strong></td>
                                        <td>
                                            <span class=format!("signal {}", tone(s.score))>
                                                {format!("{:+.2}", s.score)}
                                            </span>
                                        </td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>
        </HfPanel>
    }
}
