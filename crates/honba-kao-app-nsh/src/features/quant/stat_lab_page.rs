//! Stat Lab — the statistical workbench (Stage 3 Phase A.3 of the
//! gallery-rs porting guide, replacing the stub page).
//!
//! The pair's z-band plot (the stationarity rail) over the lab's printed
//! statistics table. The band's `z_score` is one `f64` crossing at the
//! organism boundary; the table's statistics are Decimal strings.

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

#[derive(Debug, Clone, PartialEq, Props)]
pub struct StatLabPageProps {
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

/// The lab's stat rows (each a Decimal fixture print).
fn stat_rows() -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    for [a, b] in [
        ["Adf p-value", "0.041"],
        ["Correlation", "0.88"],
        ["Hedge ratio", "0.62"],
        ["Half-life", "11 sessions"],
        ["Spread mean", "₹14.20"],
        ["Spread stdev", "₹9.60"],
        ["Sample days", "252"],
    ] {
        rows.push(vec![a.to_string(), b.to_string()]);
    }
    rows
}

/// Stat Lab — stationarity and the pair's z rail.
#[component]
pub fn StatLabPage(_props: StatLabPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let header = Some(
        ["Measure", "Value"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Stat Lab".to_string())),
        subtitle: Prop::from(Some(
            "ADF, correlation and the spread's sigma rails — the mean-reversion preflight"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(HfStatsBar::element(HfStatsBarProps {
        stats: Prop::from(vec![
            StatItem {
                label: "Pair".into(),
                value: "RELIANCE/TCS".into(),
                is_positive: None,
            },
            StatItem {
                label: "Lookback".into(),
                value: "252d".into(),
                is_positive: None,
            },
            StatItem {
                label: "Verbose rail".into(),
                value: "±2σ".into(),
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
            .child(
                view()
                    .w(px(170.0))
                    .child(metric("Cointegrated", "PASS".into(), true)),
            )
            .child(
                view()
                    .w(px(170.0))
                    .child(metric("Z score", "-1.24".into(), true)),
            )
            .child(
                view()
                    .w(px(170.0))
                    .child(metric("Signal", "NOT ENTERED".into(), false)),
            ),
    )
    .child(
        view()
            .w(px(460.0))
            .child(HfCointegrationBand::element(HfCointegrationBandProps {
                z_score: Prop::from(-1.24),
                class: Prop::default(),
            })),
    )
    .child(HfTable::element(HfTableProps {
        rows: Prop::from(stat_rows()),
        header: Prop::from(header),
        align: Prop::default(),
    }))
}
