//! Pair Trading — the spread monitor (Stage 3 Phase A.3 of the
//! gallery-rs porting guide, replacing the stub page).
//!
//! The RELIANCE/TCS spread walk: a z-band rail, the per-day spread
//! table and the trade-signal tiles. The band's `z_score` is one `f64`
//! crossing at the boundary; the table's economics are Decimal (the
//! hedge-ratio and mean adjustments, integer/Decimal only).

use crate::features::fixtures;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::molecules::quant::backtest_card::{
    HfBacktestMetricCard, HfBacktestMetricCardProps,
};
use honba_kao_ui_core::molecules::quant::cointegration_band::{
    HfCointegrationBand, HfCointegrationBandProps,
};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
#[derive(Debug, Clone, PartialEq, Props)]
pub struct PairTradingPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// The spread walk over the last ten sessions: hedge ratio 0.31 × the
/// RELIANCE walk minus the TCS walk, decimal-exact per fixture. Today's
/// z derived in Decimal and converted once for the band boundary.
fn last_z() -> f64 {
    let reliance = fixtures::candles(dec!(22842.50), 10);
    let tcs = fixtures::candles(dec!(4118.20), 10);
    let last_rep = reliance
        .last()
        .and_then(|c| c.close.parse::<Decimal>().ok())
        .unwrap_or_default();
    let last_tcs = tcs
        .last()
        .and_then(|c| c.close.parse::<Decimal>().ok())
        .unwrap_or_default();
    let spread = last_rep - dec!(0.31) * last_tcs - dec!(11_532.11);
    (spread / dec!(9.60)).to_f64().unwrap_or_default()
}

/// Pair Trading — the spread board.
#[component]
pub fn PairTradingPage(_props: PairTradingPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let reliance = fixtures::candles(dec!(22842.50), 10);
    let tcs = fixtures::candles(dec!(4118.20), 10);

    let header = Some(
        ["Session", "RELIANCE close", "TCS close", "Spread"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );
    let rows = reliance
        .iter()
        .zip(tcs.iter())
        .map(|(a, b)| {
            let a_close = a.close.parse::<Decimal>().unwrap_or_default();
            let b_close = b.close.parse::<Decimal>().unwrap_or_default();
            vec![
                a.label.clone(),
                a.close.clone(),
                b.close.clone(),
                fixtures::fmt(a_close - dec!(0.31) * b_close - dec!(11_532.11)),
            ]
        })
        .collect::<Vec<_>>();

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Pair Trading".to_string())),
        subtitle: Prop::from(Some(
            "RELIANCE/TCS cointegrated spread: the z rail, the session ladder and the signal risk"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(HfStatsBar::element(HfStatsBarProps {
        stats: Prop::from(vec![
            StatItem {
                label: "Hedge ratio".into(),
                value: "0.31".into(),
                is_positive: None,
            },
            StatItem {
                label: "Sigma".into(),
                value: "₹9.60".into(),
                is_positive: None,
            },
            StatItem {
                label: "Entry rail".into(),
                value: "|z| ≥ 2.0".into(),
                is_positive: None,
            },
        ]),
    }))
    .child(
        view()
            .w(px(460.0))
            .child(HfCointegrationBand::element(HfCointegrationBandProps {
                z_score: Prop::from(last_z()),
                class: Prop::default(),
            })),
    )
    .child(
        view()
            .flex_row()
            .flex_wrap()
            .gap(px(10.0))
            .w_full()
            .child(view().w(px(170.0)).child(HfBacktestMetricCard::element(
                HfBacktestMetricCardProps {
                    label: Prop::from("Today's z".to_string()),
                    value: Prop::from("-1.9".to_string()),
                    positive: Prop::from(true),
                },
            )))
            .child(view().w(px(170.0)).child(HfBacktestMetricCard::element(
                HfBacktestMetricCardProps {
                    label: Prop::from("Leg quant".to_string()),
                    value: Prop::from("LONG-READY".to_string()),
                    positive: Prop::from(false),
                },
            ))),
    )
    .child(HfTable::element(HfTableProps {
        rows: Prop::from(rows),
        header: Prop::from(header),
        align: Prop::default(),
    }))
}
