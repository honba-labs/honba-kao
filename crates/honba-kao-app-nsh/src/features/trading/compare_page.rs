//! Compare — two-chart side-by-side comparison (Stage 3 Phase A.3 of the
//! gallery-rs porting guide, replacing the stub page).
//!
//! RELIANCE v TCS: two M5 candle canvases, the day's stats above, and a
//! compare table below. The day change derives from the fixture walk in
//! Decimal arithmetic: the walk's candle-0 close sits −21 off base, the
//! candle-39 close +3 off base (+24 over the walk), so percent =
//! Δ ÷ first close × 100 — zero floats.

use crate::features::fixtures;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::organisms::candlestick_chart::{
    ChartPeriod, HfCandlestickChart, HfCandlestickChartProps,
};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct ComparePageProps {
    #[prop(default)]
    pub _unused: String,
}

/// The fixture walk's per-candle drift: `(i·37 mod 43) − 21` — mirrors
/// `fixtures::candles` (integer math only), pulled out so the compare
/// table can state the walk's start/end without re-deriving candles.
fn drift(i: i64) -> i64 {
    (i * 37).rem_euclid(43) - 21
}

/// The day-over-walk change percent of a fixture candle series, against
/// its own first close.
fn day_change(base: Decimal) -> Decimal {
    let first = base + Decimal::from(drift(0));
    let last = base + Decimal::from(drift(39));
    (last - first) / first * Decimal::from(100)
}

/// The two-script compare table: one metric per row, one column each.
fn compare_rows(reliance: &Decimal, tcs: &Decimal) -> Vec<Vec<String>> {
    let metric = |title: &str, a: Decimal, b: Decimal| -> Vec<String> {
        vec![title.to_string(), fixtures::fmt(a), fixtures::fmt(b)]
    };
    vec![
        metric("Base price", *reliance, *tcs),
        metric("Prev close", *reliance - dec!(42.50), *tcs - dec!(42.50)),
        metric(
            "Close +39 candles",
            *reliance + Decimal::from(drift(39)),
            *tcs + Decimal::from(drift(39)),
        ),
        metric("Day change %", day_change(*reliance), day_change(*tcs)),
    ]
}

/// Compare — RELIANCE v TCS, side by side.
#[component]
pub fn ComparePage(_props: ComparePageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let reliance = dec!(22842.50);
    let tcs = dec!(4118.20);

    let header = Some(
        ["Metric", "RELIANCE-EQ", "TCS-EQ"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Compare".to_string())),
        subtitle: Prop::from(Some(
            "RELIANCE-EQ v TCS-EQ — synchronized M5 canvases and the day's compare table"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(HfStatsBar::element(HfStatsBarProps {
        stats: Prop::from(vec![
            StatItem {
                label: "RELIANCE Δ%".into(),
                value: format!("{}%", fixtures::fmt(day_change(reliance))),
                is_positive: Some(true),
            },
            StatItem {
                label: "TCS Δ%".into(),
                value: format!("{}%", fixtures::fmt(day_change(tcs))),
                is_positive: Some(true),
            },
            StatItem {
                label: "Price ratio".into(),
                value: fixtures::fmt(reliance / tcs),
                is_positive: None,
            },
        ]),
    }))
    .child(
        view()
            .flex_row()
            .gap(px(12.0))
            .w_full()
            .child(
                view()
                    .flex_1()
                    .child(HfCandlestickChart::element(HfCandlestickChartProps {
                        candles: Prop::from(fixtures::candles(reliance, 40)),
                        period: Prop::from(ChartPeriod::M5),
                        show_volume: Prop::from(true),
                        show_averages: Prop::from(false),
                        prev_close: Prop::from(fixtures::fmt(reliance - dec!(42.50))),
                        height: 240.0,
                    })),
            )
            .child(
                view()
                    .flex_1()
                    .child(HfCandlestickChart::element(HfCandlestickChartProps {
                        candles: Prop::from(fixtures::candles(tcs, 40)),
                        period: Prop::from(ChartPeriod::M5),
                        show_volume: Prop::from(true),
                        show_averages: Prop::from(false),
                        prev_close: Prop::from(fixtures::fmt(tcs - dec!(42.50))),
                        height: 240.0,
                    })),
            ),
    )
    .child(HfTable::element(HfTableProps {
        rows: Prop::from(compare_rows(&reliance, &tcs)),
        header: Prop::from(header),
        align: Prop::default(),
    }))
}
