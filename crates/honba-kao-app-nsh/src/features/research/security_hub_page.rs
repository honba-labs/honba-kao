//! Security Hub — the scrip dossier (Stage 3 Phase A.3 of the gallery-rs
//! porting guide, replacing the stub page).
//!
//! One scrip (RELIANCE) end to end: the top-bar quote, the M5 candle
//! chart, the L2 ladder, and the filings card strip — the page-level
//! composition the guide's Phase A.3 calls for (the same organisms the
//! cockpit assembles, one scrip deep).

use crate::features::fixtures;
use honba_kao_ui_core::molecules::quant::filings_card::{HfFilingsCard, HfFilingsCardProps};
use honba_kao_ui_core::organisms::candlestick_chart::{
    ChartPeriod, HfCandlestickChart, HfCandlestickChartProps,
};
use honba_kao_ui_core::organisms::order_book::{HfOrderBook, HfOrderBookProps};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct SecurityHubPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// Security Hub — one scrip's end-to-end dossier.
#[component]
pub fn SecurityHubPage(_props: SecurityHubPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let base = dec!(2912.45);
    let candles = fixtures::candles(base, 40);
    let close = candles
        .last()
        .and_then(|c| c.close.parse::<Decimal>().ok())
        .unwrap_or(base);

    let (bids, asks, spread) = fixtures::book(close);

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Security Hub".to_string())),
        subtitle: Prop::from(Some(
            "RELIANCE end to end: quote, L2 ladder, M5 chart and the corporate filings strip"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(view().flex_row().gap_2().child(quote_top_bar(base, close)))
    .child(
        HfCandlestickChart::element(HfCandlestickChartProps {
            candles: Prop::from(candles),
            period: Prop::from(ChartPeriod::D1),
            show_volume: Prop::from(true),
            show_averages: Prop::from(true),
            prev_close: Prop::from(fixtures::fmt(base - dec!(18.50))),
            height: 260.0,
        }),
    )
    .child(
        HfOrderBook::element(HfOrderBookProps {
            bids: Prop::from(bids),
            asks: Prop::from(asks),
            spread: Prop::from(spread),
        }),
    )
    .child(
        HfFilingsCard::element(HfFilingsCardProps {
            title: Prop::from("BSE — outcome of board meeting".to_string()),
            excerpt: Prop::from(
                "The board considered and approved the unaudited financial results for the\nhalf year ended September 30, 2026. Letter of outcome attached."
                    .to_string(),
            ),
            class: Prop::default(),
        }),
    )
    .child(
        HfFilingsCard::element(HfFilingsCardProps {
            title: Prop::from("NSE — analyst/institutional investor meet intimation".to_string()),
            excerpt: Prop::from(
                "The exchange has been informed of an investor call scheduled for November 4,\n2026. Recording and transcript to follow on the exchange portal."
                    .to_string(),
            ),
            class: Prop::default(),
        }),
    )
}

/// The quote strip above the dossier (kept page-local).
fn quote_top_bar(base: Decimal, close: Decimal) -> impl IntoElement {
    use honba_kao_ui_core::organisms::market_top_bar::{HfMarketTopBar, HfMarketTopBarProps};
    HfMarketTopBar::element(HfMarketTopBarProps {
        labels: Prop::from(vec![
            "RELIANCE".to_string(),
            "LTP".to_string(),
            "Δ".to_string(),
        ]),
        values: Prop::from(vec![
            "NSE EQ".to_string(),
            format!("₹{}", fixtures::fmt(close)),
            fixtures::fmt(close - base),
        ]),
        desk: Prop::from(Some("Security Hub".to_string())),
        currency: Prop::from(Some("INR".to_string())),
    })
}
