//! Markets — the breadth board (Stage 3 Phase A.3 of the gallery-rs
//! porting guide, replacing the stub page).
//!
//! The index top bar, the streaming quote strip and the breadth table
//! over the fixture universe. Breadth is integer counts; prices are
//! Decimal strings from the shared fixtures.

use crate::features::fixtures;
use honba_kao_ui_core::organisms::market_tape::{HfMarketTape, HfMarketTapeProps};
use honba_kao_ui_core::organisms::market_top_bar::{HfMarketTopBar, HfMarketTopBarProps};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps, TableCellAlign};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use honba_kao_ui_core::theme::tokens::HonbaStyle;
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct MarketsPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// The breadth rows: every fixture scrip's last print, day tics and
/// direction (integer/Decimal only).
fn breadth_rows() -> Vec<Vec<String>> {
    fixtures::TICKERS
        .iter()
        .enumerate()
        .map(|(i, ticker)| {
            let price = dec!(100) + Decimal::from((i * 313) % 4_500);
            let tics = ((i * 29) % 7) as i64 - 3;
            vec![
                ticker.to_string(),
                format!("₹{}", fixtures::fmt(price)),
                (if tics >= 0 { "+" } else { "-" }).to_string()
                    + &fixtures::fmt(Decimal::from(tics.abs())),
                (if tics >= 0 { "up" } else { "down" }).to_string(),
            ]
        })
        .collect()
}

/// Markets — the cross-section board.
#[component]
pub fn MarketsPage(_props: MarketsPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let indices = (0..4)
        .map(|i| dec!(22_000) + Decimal::from((i * 821) % 28_000))
        .collect::<Vec<_>>();

    let header = Some(
        ["Scrip", "Close", "Tics", "Day"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Markets".to_string())),
        subtitle: Prop::from(Some(
            "The session's cross-section: indexes, the live strip and breadth".to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(HfMarketTopBar::element(HfMarketTopBarProps {
        labels: Prop::from(
            ["NIFTY", "BANKNIFTY", "SENSEX", "SECTOR"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        ),
        values: Prop::from(
            indices
                .iter()
                .map(|d| fixtures::fmt(*d))
                .collect::<Vec<_>>(),
        ),
        desk: Prop::from(Some("Markets Desk".to_string())),
        currency: Prop::from(Some("INR".to_string())),
    }))
    .child(
        text("The strip and the breadth rows share the fixture strand.")
            .text_size(px(12.0))
            .text_muted(),
    )
    .child(HfMarketTape::element(HfMarketTapeProps {
        entries: Prop::from(fixtures::tape()),
    }))
    .child(HfTable::element(HfTableProps {
        rows: Prop::from(breadth_rows()),
        header: Prop::from(header),
        align: Prop::from(Some(TableCellAlign::Center)),
    }))
}
