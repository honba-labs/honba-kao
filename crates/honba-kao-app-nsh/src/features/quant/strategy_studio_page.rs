//! Strategy Studio — the synthesized-strategy shelf (Stage 3 Phase A.3
//! of the gallery-rs porting guide, replacing the stub page).
//!
//! `honba-nou`'s strategy synthesizer output as AI artifact cards (one
//! kept, one rejected), the studio's metric tiles and the assignment
//! table. The node-canvas studio itself lives in honba-kao-fe (the web
//! desktop); the app page is the read-only shelf of synthesized runs.

use crate::features::fixtures;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::molecules::quant::artifact_card::{HfAiArtifactCard, HfAiArtifactCardProps};
use honba_kao_ui_core::molecules::quant::backtest_card::{
    HfBacktestMetricCard, HfBacktestMetricCardProps,
};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use nsh_core::prelude::*;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct StrategyStudioPageProps {
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

/// The session's synthesized strategies per ticker.
fn assignment_rows() -> Vec<Vec<String>> {
    fixtures::TICKERS
        .iter()
        .take(6)
        .enumerate()
        .map(|(i, ticker)| {
            let strategy = ["Turtle breakout", "VWAP slicer", "Quoting", "Risk parity"][i % 4];
            let verdict = ["KEPT", "REJECTED", "KEPT", "KEPT", "REJECTED", "KEPT"][i];
            let edge = format!(
                "{} EV",
                Decimal::from(120 + (i * 37) % 90) / Decimal::from(100)
            );
            vec![
                ticker.to_string(),
                strategy.to_string(),
                verdict.to_string(),
                edge,
            ]
        })
        .collect()
}

/// Strategy Studio — the synthesized strategies shelf.
#[component]
pub fn StrategyStudioPage(
    _props: StrategyStudioPageProps,
    _cx: Scope<NoEvent>,
) -> impl IntoElement {
    let header = Some(
        ["Scrip", "Synthesized", "Verdict", "Out-of-sample edge"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Strategy Studio".to_string())),
        subtitle: Prop::from(Some(
            "The AI strategy synthesizer's latest run: kept and rejected candidates, per scrip"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(
        HfStatsBar::element(HfStatsBarProps {
            stats: Prop::from(vec![
                StatItem {
                    label: "Run".into(),
                    value: "ODYSSEUS 12".into(),
                    is_positive: None,
                },
                StatItem {
                    label: "Candidates".into(),
                    value: "6".into(),
                    is_positive: None,
                },
                StatItem {
                    label: "Kept".into(),
                    value: "4".into(),
                    is_positive: Some(true),
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
            .child(view().w(px(170.0)).child(metric(
                "Avg of kept",
                "1.52 EV".into(),
                true,
            )))
            .child(view().w(px(170.0)).child(metric(
                "Reject rate",
                "33%".into(),
                false,
            )))
            .child(view().w(px(170.0)).child(metric(
                "Budget used",
                "12/60 generations".into(),
                true,
            ))),
    )
    .child(
        HfAiArtifactCard::element(HfAiArtifactCardProps {
            title: Prop::from("Synthesis 12 — kept candidate".to_string()),
            content: Prop::from(
                "Turtle breakout over RELIANCE: the 20/55 Donchian rails held through the\nsynthetic session with 1.52 out-of-sample EV; the slicer variant was\nrejected after the 2σ regime shift test."
                    .to_string(),
            ),
            class: Prop::default(),
        }),
    )
    .child(
        HfTable::element(HfTableProps {
            rows: Prop::from(assignment_rows()),
            header: Prop::from(header),
            align: Prop::default(),
        }),
    )
}
