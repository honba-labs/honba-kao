//! Multi-Chart — four synchronized charts across the watchlist (Stage 3
//! Phase A.3 of the gallery-rs porting guide, replacing the stub page).
//!
//! A 2×2 grid of the pool's native-canvas [`HfCandlestickChart`]s, one per
//! flagship scrip, each on the shared Decimal fixtures.

use crate::features::fixtures;
use honba_kao_ui_core::organisms::candlestick_chart::{
    ChartPeriod, HfCandlestickChart, HfCandlestickChartProps,
};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use honba_kao_ui_core::theme::tokens::HonbaStyle;
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct MultiChartPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// One chart tile: the scrip's caption over its M5 candle canvas.
fn chart_tile(name: &str, base: Decimal) -> impl IntoElement {
    let chart = HfCandlestickChart::element(HfCandlestickChartProps {
        candles: Prop::from(fixtures::candles(base, 40)),
        period: Prop::from(ChartPeriod::M5),
        show_volume: Prop::from(true),
        show_averages: Prop::from(true),
        prev_close: Prop::from(fixtures::fmt(base - dec!(42.50))),
        height: 220.0,
    });
    view()
        .flex_col()
        .gap_1()
        .p_2()
        .child(
            text(name.to_string())
                .text_size(px(13.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_muted(),
        )
        .child(chart)
}

/// Multi-Chart — the 2×2 synchronized chart wall.
#[component]
pub fn MultiChartPage(_props: MultiChartPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    HfPage::element(HfPageProps {
        title: Prop::from(Some("Multi-Chart".to_string())),
        subtitle: Prop::from(Some(
            "Four synchronized M5 tables across the active watchlist".to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(
        view()
            .flex_row()
            .flex_wrap()
            .gap(px(10.0))
            .w_full()
            .child(
                view()
                    .w(px(430.0))
                    .child(chart_tile("RELIANCE-EQ", dec!(22842.50))),
            )
            .child(
                view()
                    .w(px(430.0))
                    .child(chart_tile("TCS-EQ", dec!(4118.20))),
            )
            .child(
                view()
                    .w(px(430.0))
                    .child(chart_tile("HDFCBANK-EQ", dec!(1682.85))),
            )
            .child(
                view()
                    .w(px(430.0))
                    .child(chart_tile("INFY-EQ", dec!(1786.30))),
            ),
    )
}
