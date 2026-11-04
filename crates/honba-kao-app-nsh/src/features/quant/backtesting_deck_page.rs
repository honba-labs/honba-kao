//! Backtesting Deck — the strategy's risk arc and metric rails (Stage 3
//! Phase A.3 of the gallery-rs porting guide, replacing the stub page).
//!
//! The performance metric deck over the engine's summary, the underwater
//! (drawdown) gauge and the rolling-candle source table. The gauge's
//! `drawdown_pct` is the one `f64` crossing — the drawdown itself is
//! computed in Decimal and converted once at the organism boundary.

use crate::features::fixtures;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::molecules::quant::backtest_card::{
    HfBacktestMetricCard, HfBacktestMetricCardProps,
};
use honba_kao_ui_core::molecules::quant::underwater_gauge::{
    HfUnderwaterGauge, HfUnderwaterGaugeProps,
};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct BacktestingDeckPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// One metric tile (shared shape with the simulator deck).
fn metric(label: &'static str, value: String, positive: bool) -> impl IntoElement {
    HfBacktestMetricCard::element(HfBacktestMetricCardProps {
        label: Prop::from(label.to_string()),
        value: Prop::from(value),
        positive: Prop::from(positive),
    })
}

/// The drawdown walk per fixture candle: peak-to-trough in percent,
/// Decimal math, then one conversion at the gauge boundary.
fn drawdown_pct() -> f64 {
    let candles = fixtures::candles(dec!(4118.20), 41);
    let closes = candles
        .iter()
        .filter_map(|c| c.close.parse::<Decimal>().ok())
        .collect::<Vec<_>>();
    let mut peak = closes[0];
    let mut worst = dec!(0);
    for close in &closes {
        if *close > peak {
            peak = *close;
        }
        let dd = (peak - close) / peak * Decimal::from(100);
        if dd > worst {
            worst = dd;
        }
    }
    worst.to_f64().unwrap_or_default()
}

/// Backtesting Deck — the run's risk arc and metric rails.
#[component]
pub fn BacktestingDeckPage(
    _props: BacktestingDeckPageProps,
    _cx: Scope<NoEvent>,
) -> impl IntoElement {
    let header = Some(
        ["Candle", "High", "Low", "Volume"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );
    let rows: Vec<Vec<String>> = fixtures::candles(dec!(4118.20), 41)
        .iter()
        .enumerate()
        .map(|(i, c)| {
            // Every third candle shows a doubled notional print.
            vec![
                c.label.clone(),
                c.high.clone(),
                c.low.clone(),
                if i % 3 == 0 {
                    format!("x2 · {}", c.volume)
                } else {
                    c.volume.clone()
                },
            ]
        })
        .collect();

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Backtesting Deck".to_string())),
        subtitle: Prop::from(Some(
            "Backtest run 12: the metric rails over the engine's summary and the underwater arc"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(HfStatsBar::element(HfStatsBarProps {
        stats: Prop::from(vec![
            StatItem {
                label: "Engine".into(),
                value: "honba-ya".into(),
                is_positive: None,
            },
            StatItem {
                label: "Candles walked".into(),
                value: "41".into(),
                is_positive: None,
            },
            StatItem {
                label: "Seed".into(),
                value: "deterministic".into(),
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
                    .child(metric("Net P&L", "₹1,42,880".into(), true)),
            )
            .child(
                view()
                    .w(px(170.0))
                    .child(metric("Trades", "412".into(), true)),
            )
            .child(
                view()
                    .w(px(170.0))
                    .child(metric("Avg Win", "+₹2,140".into(), true)),
            )
            .child(
                view()
                    .w(px(170.0))
                    .child(metric("Avg Loss", "-₹980".into(), false)),
            )
            .child(
                view()
                    .w(px(170.0))
                    .child(metric("Payoff", "2.18".into(), true)),
            ),
    )
    .child(
        view()
            .w(px(480.0))
            .child(HfUnderwaterGauge::element(HfUnderwaterGaugeProps {
                drawdown_pct: Prop::from(drawdown_pct()),
                class: Prop::default(),
            })),
    )
    .child(HfTable::element(HfTableProps {
        rows: Prop::from(rows),
        header: Prop::from(header),
        align: Prop::default(),
    }))
}
