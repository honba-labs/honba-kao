//! DOM — depth-of-market ladder (Stage 3 Phase A.3 of the gallery-rs
//! porting guide, replacing the stub page).
//!
//! The full [`HfOrderBook`] ladder with the top-of-book stats above and
//! the per-level size imbalance below. All values come from the shared
//! Decimal fixtures; the imbalance columns are pure integer arithmetic.

use crate::features::fixtures;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::organisms::order_book::{HfOrderBook, HfOrderBookProps};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct DomPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// Per-level size-imbalance rows (same deterministic walk the fixture
/// book draws): level, bid size, ask size, and the ask/bid skew in
/// percent (Decimal math only).
fn imbalance_rows() -> Vec<Vec<String>> {
    (0..10)
        .map(|i| {
            let bid = 1_900 + 240 * (9 - i) + 40;
            let ask = 2_100 + 260 * i + 40;
            let skew =
                Decimal::from(2 * ask - bid - ask) * Decimal::from(100) / Decimal::from(bid + ask);
            vec![
                format!("L{}", i + 1),
                fixtures::fmt(Decimal::from(bid)),
                fixtures::fmt(Decimal::from(ask)),
                format!("{}%", fixtures::fmt(skew)),
            ]
        })
        .collect()
}

/// DOM — the L2 ladder with top-of-book stats and level imbalance.
#[component]
pub fn DomPage(_props: DomPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let base = dec!(22842.50);
    let (bids, asks, spread) = fixtures::book(base);

    // Cumulative depth from the same deterministic walk the ladder draws.
    let bid_depth: i64 = (0..10).map(|i| 1_900 + 240 * (9 - i) + 40).sum();
    let ask_depth: i64 = (0..10).map(|i| 2_100 + 260 * i + 40).sum();

    let header = Some(
        ["Level", "Bid Size", "Ask Size", "Ask Skew"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("DOM".to_string())),
        subtitle: Prop::from(Some(
            "Depth of market — the ten-deep L2 ladder with level imbalance".to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(HfStatsBar::element(HfStatsBarProps {
        stats: Prop::from(vec![
            StatItem {
                label: "Mid".into(),
                value: format!("₹{}", fixtures::fmt(base)),
                is_positive: None,
            },
            StatItem {
                label: "Spread".into(),
                value: format!(
                    "₹{} ({}%)",
                    spread
                        .as_ref()
                        .map(|s| s.absolute.clone())
                        .unwrap_or_default(),
                    spread
                        .as_ref()
                        .map(|s| s.percentage.clone())
                        .unwrap_or_default()
                ),
                is_positive: None,
            },
            StatItem {
                label: "Bid Depth".into(),
                value: fixtures::thousands(bid_depth),
                is_positive: Some(true),
            },
            StatItem {
                label: "Ask Depth".into(),
                value: fixtures::thousands(ask_depth),
                is_positive: Some(false),
            },
        ]),
    }))
    .child(HfOrderBook::element(HfOrderBookProps {
        bids: Prop::from(bids),
        asks: Prop::from(asks),
        spread: Prop::from(spread),
    }))
    .child(HfTable::element(HfTableProps {
        rows: Prop::from(imbalance_rows()),
        header: Prop::from(header),
        align: Prop::default(),
    }))
}
