//! Lab Optimizer — the allocation workbench (Stage 3 Phase A.3 of the
//! gallery-rs porting guide, replacing the stub page).
//!
//! The portfolio optimizer's print: the AI artifact card with the
//! optimizer's reasoning, the headline metric tiles and the allocation
//! table over the fixture universe (weights from a deterministic walk,
//! renormalized in Decimal).

use crate::features::fixtures;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::molecules::quant::artifact_card::{HfAiArtifactCard, HfAiArtifactCardProps};
use honba_kao_ui_core::molecules::quant::backtest_card::{
    HfBacktestMetricCard, HfBacktestMetricCardProps,
};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps, TableCellAlign};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use nsh_core::prelude::*;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct LabOptimizerPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// One metric tile.
fn metric(label: &'static str, value: String, positive: bool) -> impl IntoElement {
    HfBacktestMetricCard::element(HfBacktestMetricCardProps {
        label: Prop::from(label.to_string()),
        value: Prop::from(value),
        positive: Prop::from(positive),
    })
}

/// The allocation walk: raw scores renormalized to 100% in Decimal.
fn allocation_rows() -> Vec<Vec<String>> {
    let scores = (0..6)
        .map(|i| Decimal::from(35 + (i * 29) % 120))
        .collect::<Vec<_>>();
    let total = scores.iter().sum::<Decimal>();
    fixtures::TICKERS
        .iter()
        .take(6)
        .zip(scores.iter())
        .map(|(ticker, score)| {
            let weight = score / total * Decimal::from(100);
            vec![
                ticker.to_string(),
                format!("{}%", fixtures::fmt(weight)),
                format!("₹{}", fixtures::fmt(weight / Decimal::from(100))),
            ]
        })
        .collect()
}

/// Lab Optimizer — the allocation workbench.
#[component]
pub fn LabOptimizerPage(_props: LabOptimizerPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let header = Some(
        ["Weight", "Allocation %", "₹ notional per 100k"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Lab Optimizer".to_string())),
        subtitle: Prop::from(Some(
            "The optimizer's latest solve: weights, notional and the engine's reasoning"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(
        HfStatsBar::element(HfStatsBarProps {
            stats: Prop::from(vec![
                StatItem {
                    label: "Solver".into(),
                    value: "MVO".into(),
                    is_positive: None,
                },
                StatItem {
                    label: "Paper universe".into(),
                    value: "6 scrips".into(),
                    is_positive: None,
                },
            ]),
        }),
    )
    .child(
        view()
            .flex_row()
            .flex_wrap()
            .gap(px(10.0))
            .w_full()
            .child(view().w(px(180.0)).child(metric(
                "Expected vol",
                "13.7%".into(),
                true,
            )))
            .child(view().w(px(180.0)).child(metric(
                "Sharpe",
                "1.34".into(),
                true,
            )))
            .child(view().w(px(180.0)).child(metric(
                "Solver status",
                "CONVERGED".into(),
                true,
            ))),
    )
    .child(
        HfAiArtifactCard::element(HfAiArtifactCardProps {
            title: Prop::from("Solve 7 — reasoning".to_string()),
            content: Prop::from(
                "Shortening RELIANCE's weight 3.1 pts cut the solve's vol 41 bps; the\nmid-caps retained their risk-parity weights. Next solve: month-end flow\nattributes."
                    .to_string(),
            ),
            class: Prop::default(),
        }),
    )
    .child(
        HfTable::element(HfTableProps {
            rows: Prop::from(allocation_rows()),
            header: Prop::from(header),
            align: Prop::from(Some(TableCellAlign::Center)),
        }),
    )
}
