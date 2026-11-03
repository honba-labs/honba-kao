//! Shadow Account — the paper-trading blotter (Stage 3 Phase A.3 of the
//! gallery-rs porting guide, replacing the stub page).
//!
//! The shadow book runs the same OMS state machine against mirrored
//! fills: deterministic Decimal fixture prints here, so the page is
//! offline-stable. The `ShadowAccount` UI reads the paper ledger's
//! equity arc and its open shadow positions.

use crate::features::fixtures;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::molecules::quant::backtest_card::{
    HfBacktestMetricCard, HfBacktestMetricCardProps,
};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps, TableCellAlign};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct ShadowAccountPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// The paper equity arc: 1,00,000 seed compounding the per-session
/// papers (integer/Decimal only).
fn arc_rows() -> Vec<Vec<String>> {
    let mut equity = dec!(1_00_000.00);
    let mut rows = Vec::new();
    for (i, ticker) in fixtures::TICKERS.iter().take(8).enumerate() {
        let paper = Decimal::from(((i * 61) % 240) - 100) / dec!(100);
        equity *= Decimal::from(1) + paper;
        rows.push(vec![
            ticker.to_string(),
            format!("{paper}"),
            format!("₹{}", fixtures::fmt(equity)),
        ])
    }
    rows
}

/// Shadow Account — the mirrored paper book.
#[component]
pub fn ShadowAccountPage(_props: ShadowAccountPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let arc = arc_rows();
    let paper_end = arc.last().map(|r| r[2].clone()).unwrap_or_default();

    let header = Some(
        ["Scrip", "Paper P&L %", "Paper equity"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Shadow Account".to_string())),
        subtitle: Prop::from(Some(
            "The paper-trading blotter: the same OMS over mirrored fills, offline-stable"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(HfStatsBar::element(HfStatsBarProps {
        stats: Prop::from(vec![
            StatItem {
                label: "Paper sessions".into(),
                value: "8".into(),
                is_positive: None,
            },
            StatItem {
                label: "Mirror".into(),
                value: "seeded".into(),
                is_positive: None,
            },
        ]),
    }))
    .child(
        view()
            .flex_row()
            .flex_wrap()
            .gap(px(10.0))
            .w_full()
            .child(view().w(px(180.0)).child(HfBacktestMetricCard::element(
                HfBacktestMetricCardProps {
                    label: Prop::from("Paper start".to_string()),
                    value: Prop::from("₹1,00,000.00".to_string()),
                    positive: Prop::from(true),
                },
            )))
            .child(view().w(px(180.0)).child(HfBacktestMetricCard::element(
                HfBacktestMetricCardProps {
                    label: Prop::from("Paper end".to_string()),
                    value: Prop::from(paper_end.clone()),
                    positive: Prop::from(true),
                },
            ))),
    )
    .child(HfTable::element(HfTableProps {
        rows: Prop::from(arc),
        header: Prop::from(header),
        align: Prop::from(Some(TableCellAlign::Center)),
    }))
}
