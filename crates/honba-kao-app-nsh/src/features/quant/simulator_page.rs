//! Quant Simulator — the simulated session's performance board (Stage 3
//! Phase A.3 of the gallery-rs porting guide, replacing the stub page).
//!
//! The market top bar over a simulator desk, the eight performance
//! metric tiles, and the annual returns table. Every value is a Decimal
//! fixture strand (no floats); the simulator semantics live in
//! honba-ya's tester, this page reads its printed summary.

use crate::features::fixtures;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::molecules::quant::backtest_card::{
    HfBacktestMetricCard, HfBacktestMetricCardProps,
};
use honba_kao_ui_core::organisms::market_top_bar::{HfMarketTopBar, HfMarketTopBarProps};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct SimulatorPageProps {
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

/// The simulated session's eight headline tiles (decimal fixtures).
fn metric_deck() -> impl IntoElement {
    view()
        .flex_row()
        .flex_wrap()
        .gap(px(10.0))
        .w_full()
        .child(
            view()
                .w(px(170.0))
                .child(metric("CAGR", "18.4%".into(), true)),
        )
        .child(
            view()
                .w(px(170.0))
                .child(metric("Sharpe", "1.92".into(), true)),
        )
        .child(
            view()
                .w(px(170.0))
                .child(metric("Sortino", "2.61".into(), true)),
        )
        .child(
            view()
                .w(px(170.0))
                .child(metric("Max DD", "-9.8%".into(), false)),
        )
        .child(
            view()
                .w(px(170.0))
                .child(metric("Win rate", "57.3%".into(), true)),
        )
        .child(
            view()
                .w(px(170.0))
                .child(metric("Expectancy", "+₹312".into(), true)),
        )
        .child(
            view()
                .w(px(170.0))
                .child(metric("Profit factor", "1.84".into(), true)),
        )
        .child(
            view()
                .w(px(170.0))
                .child(metric("Exposure", "42.1%".into(), true)),
        )
}

/// The five simulated calendar years (decimal arithmetic on the walk).
fn year_rows() -> Vec<Vec<String>> {
    let mut equity = dec!(1_000_000.00);
    const RETURNS: [i64; 5] = [21_400, -6_900, 33_200, 12_800, 47_100];
    RETURNS
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let ret = Decimal::from(*r) / dec!(1000);
            let start = equity;
            equity *= Decimal::from(1) + ret;
            vec![
                format!("{}", 2022 + i),
                format!("₹{}", fixtures::fmt(start)),
                format!("{ret}%"),
                format!("₹{}", fixtures::fmt(equity)),
            ]
        })
        .collect()
}

/// Quant Simulator — the simulated session board.
#[component]
pub fn SimulatorPage(_props: SimulatorPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let header = Some(
        ["Year", "Start equity", "Return", "End equity"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Quant Simulator".to_string())),
        subtitle: Prop::from(Some(
            "The tester's printed summary: a simulated equity walk with its headline metrics"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(HfMarketTopBar::element(HfMarketTopBarProps {
        labels: Prop::from(
            ["SIM", "DRAWDOWN", "MODE"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        ),
        values: Prop::from(vec![
            "nsh://tester".to_string(),
            "-9.8%".to_string(),
            "deterministic".to_string(),
        ]),
        desk: Prop::from(Some("Quant Desk".to_string())),
        currency: Prop::from(Some("INR".to_string())),
    }))
    .child(HfStatsBar::element(HfStatsBarProps {
        stats: Prop::from(vec![
            StatItem {
                label: "Trades".into(),
                value: "412".into(),
                is_positive: None,
            },
            StatItem {
                label: "Sessions".into(),
                value: "97".into(),
                is_positive: None,
            },
            StatItem {
                label: "Fill rate".into(),
                value: "99.2%".into(),
                is_positive: None,
            },
        ]),
    }))
    .child(metric_deck())
    .child(HfTable::element(HfTableProps {
        rows: Prop::from(year_rows()),
        header: Prop::from(header),
        align: Prop::default(),
    }))
}
