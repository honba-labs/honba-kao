//! MTA — multi-timeframe analysis (Stage 3 Phase A.3 of the gallery-rs
//! porting guide, replacing the stub page).
//!
//! A trend matrix over the fixture universe: every instrument × timeframe
//! cell resolves to up/down/range from a deterministic integer hash, so
//! the board is stable across offline reruns (the temporal-determinism
//! invariant — no clock, no RNG). The badge above states the session
//! regime.

use crate::features::fixtures;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::molecules::quant::market_regime_badge::{
    HfMarketRegimeBadge, HfMarketRegimeBadgeProps, Regime,
};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps, TableCellAlign};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use nsh_core::prelude::*;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct MtaPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// The timeframes the matrix shows (a 5-attribute fixture, exchange-day
/// resolution).
const TIMEFRAMES: [&str; 5] = ["1m", "5m", "15m", "1h", "1D"];

/// One instrument's trend verdict on one timeframe: deterministic hash
/// (integer math only) → ▲ up / ▼ down / — range.
fn trend(instrument: usize, timeframe: usize) -> &'static str {
    match (instrument * 17 + timeframe * 31) % 5 {
        0 | 1 => "▲",
        2 | 3 => "▼",
        _ => "—",
    }
}

/// The matrix rows: each of the first 10 TICKERS × the 5 timeframes.
fn matrix_rows() -> Vec<Vec<String>> {
    fixtures::TICKERS
        .iter()
        .take(10)
        .enumerate()
        .map(|(t, ticker)| {
            let mut row = vec![ticker.to_string()];
            row.extend((0..TIMEFRAMES.len()).map(|tf| trend(t, tf).to_string()));
            row
        })
        .collect()
}

/// MTA — the multi-timeframe trend matrix.
#[component]
pub fn MtaPage(_props: MtaPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let header = Some(
        std::iter::once("Instrument".to_string())
            .chain(TIMEFRAMES.iter().map(|s| s.to_string()))
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("MTA".to_string())),
        subtitle: Prop::from(Some(
            "Multi-timeframe analysis — the trend matrix, 1m through 1D".to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(
        view()
            .flex_row()
            .gap_2()
            .child(HfMarketRegimeBadge::element(HfMarketRegimeBadgeProps {
                regime: Prop::from(Regime::Bull),
            })),
    )
    .child(HfStatsBar::element(HfStatsBarProps {
        stats: Prop::from(vec![
            StatItem {
                label: "Instruments".into(),
                value: "10".into(),
                is_positive: None,
            },
            StatItem {
                label: "Timeframes".into(),
                value: "5".into(),
                is_positive: None,
            },
            StatItem {
                label: "Cells".into(),
                value: "50".into(),
                is_positive: None,
            },
        ]),
    }))
    .child(HfTable::element(HfTableProps {
        rows: Prop::from(matrix_rows()),
        header: Prop::from(header),
        align: Prop::from(Some(TableCellAlign::Center)),
    }))
}
