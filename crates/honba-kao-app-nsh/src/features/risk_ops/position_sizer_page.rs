//! Position Sizer — the sizing workbench (Stage 3 Phase A.3 of the
//! gallery-rs porting guide, replacing the stub page).
//!
//! The risk rails' sizing arithmetic on deterministic prints: risk per
//! trade, stop distance, band width and the freeze cap, all Decimal —
//! the sizing formula the desk uses (position × stop × band), no
//! floats.

use crate::features::fixtures;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use nsh_core::prelude::*;
use rust_decimal::{Decimal, RoundingStrategy};
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct PositionSizerPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// The sizing rows: per fixture scrip — last price, 2% stop, 1% risk
/// rail and the computed position (risk ÷ stop-size, floored in
/// Decimal). Integer/Decimal walk end to end.
fn sizing_rows() -> Vec<Vec<String>> {
    fixtures::TICKERS
        .iter()
        .take(8)
        .enumerate()
        .map(|(i, ticker)| {
            let price = dec!(500) + Decimal::from((i * 217) % 9_800);
            let stop = price * dec!(2) / dec!(100);
            let risk = dec!(10_000);
            let size = risk / stop;
            vec![
                ticker.to_string(),
                format!("₹{}", fixtures::fmt(price)),
                fixtures::fmt(stop),
                fixtures::fmt(size.round_dp_with_strategy(0, RoundingStrategy::ToZero)),
            ]
        })
        .collect()
}

/// Position Sizer — the sizing workbench.
#[component]
pub fn PositionSizerPage(_props: PositionSizerPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let header = Some(
        ["Scrip", "Last", "2% stop", "Position size"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Position Sizer".to_string())),
        subtitle: Prop::from(Some(
            "Risk-rail sizing: risk per trade ÷ stop distance, floored to board lots".to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(HfStatsBar::element(HfStatsBarProps {
        stats: Prop::from(vec![
            StatItem {
                label: "Risk per trade".into(),
                value: "₹10,000".into(),
                is_positive: None,
            },
            StatItem {
                label: "Stop".into(),
                value: "2%".into(),
                is_positive: None,
            },
            StatItem {
                label: "Freeze cap".into(),
                value: "25,000 EQ".into(),
                is_positive: Some(true),
            },
        ]),
    }))
    .child(HfTable::element(HfTableProps {
        rows: Prop::from(sizing_rows()),
        header: Prop::from(header),
        align: Prop::default(),
    }))
}
