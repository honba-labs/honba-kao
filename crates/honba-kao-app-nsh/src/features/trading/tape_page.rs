//! Tape — the full time & sales stream (Stage 3 Phase A.3 of the
//! gallery-rs porting guide, replacing the stub page).
//!
//! A top stats strip (print split, last price), the full prints table and
//! the streaming-tape organism; all values are Decimal fixtures
//! (`crate::features::fixtures`) pre-formatted at the module boundary.

use crate::features::fixtures;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::organisms::market_tape::{HfMarketTape, HfMarketTapeProps};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps, TableCellAlign};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use nsh_core::prelude::*;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct TapePageProps {
    #[prop(default)]
    pub _unused: String,
}

/// The prints table rows (time · symbol · price · qty · side) from the
/// fixture tape: the timestamp walks in six-minute prints, the quantity
/// is the deterministic volume walk (integer math only).
fn print_rows() -> Vec<Vec<String>> {
    fixtures::tape()
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let minute = 9u32 * 60 + 15 + (i as u32) * 6;
            // Ticks in 5 lots: the fixture walk never spells a board lot.
            let qty = Decimal::from(1_700 + ((i * 431) % 5_800) as i64);
            vec![
                format!("{:02}:{:02}", minute / 60, minute % 60),
                t.symbol.clone(),
                format!("₹{}", t.price),
                fixtures::fmt(qty),
                (if t.up { "BUY" } else { "SELL" }).to_string(),
            ]
        })
        .collect()
}

/// Tape — live prints table and the streaming quote strip.
#[component]
pub fn TapePage(_props: TapePageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let entries = fixtures::tape();
    let last = entries.last().map(|t| t.price.clone()).unwrap_or_default();
    let buys = entries.iter().filter(|t| t.up).count();
    let sells = entries.len() - buys;

    let header = Some(
        ["Print", "Symbol", "Price", "Quantity", "Side"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Tape".to_string())),
        subtitle: Prop::from(Some(
            "Time & sales — every print this session, engine-built from Decimal fixtures"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(HfStatsBar::element(HfStatsBarProps {
        stats: Prop::from(vec![
            StatItem {
                label: "Last".into(),
                value: format!("₹{last}"),
                is_positive: None,
            },
            StatItem {
                label: "Buy Prints".into(),
                value: format!("{buys}"),
                is_positive: Some(true),
            },
            StatItem {
                label: "Sell Prints".into(),
                value: format!("{sells}"),
                is_positive: Some(false),
            },
            StatItem {
                label: "Total".into(),
                value: format!("{}", entries.len()),
                is_positive: None,
            },
        ]),
    }))
    .child(HfTable::element(HfTableProps {
        rows: Prop::from(print_rows()),
        header: Prop::from(header),
        align: Prop::from(Some(TableCellAlign::Center)),
    }))
    .child(HfMarketTape::element(HfMarketTapeProps {
        entries: Prop::from(entries),
    }))
}
