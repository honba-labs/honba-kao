//! Tradebook — the executed orders ledger (Stage 3 Phase A.3 of the
//! gallery-rs porting guide, replacing the stub page).
//!
//! The filled slices of the live `orders` signal (the OMS journal's
//! order log), priced in the account's currency, with a session
//! notional far above and the fill-rate stats. Empty-safe like the
//! portfolio board.

use crate::features::fixtures;
use crate::shared::contexts::HonbaClientContext;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps, TableCellAlign};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use nsh_core::prelude::*;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct TradebookPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// Tradebook — the session's fills.
#[component]
pub fn TradebookPage(_props: TradebookPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let client = use_context::<HonbaClientContext>().expect("client context");
    let bind = client.clone();
    let bind_stats = client;

    let header = Some(
        ["Order id", "Symbol", "Side", "Status", "Price", "Filled"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Tradebook".to_string())),
        subtitle: Prop::from(Some(
            "The session's executed orders as the OMS journal logged them".to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(HfStatsBar::element(HfStatsBarProps {
        stats: Prop::bind(move || {
            let orders = bind_stats.orders.get();
            let filled = orders
                .iter()
                .filter(|o| format!("{:?}", o.status) == "Filled")
                .count();
            // Session notional: filled-slice price x qty walks in
            // Decimal (no floats); empty session prints the seed.
            let notional = orders.iter().fold(Decimal::default(), |total, o| {
                total + o.price.unwrap_or_default() * o.filled_quantity
            });
            vec![
                StatItem {
                    label: "Journal".into(),
                    value: format!("{}", orders.len()),
                    is_positive: None,
                },
                StatItem {
                    label: "Fills".into(),
                    value: format!("{filled}"),
                    is_positive: None,
                },
                StatItem {
                    label: "Notionally".into(),
                    value: format!("₹{}", fixtures::fmt(notional)),
                    is_positive: None,
                },
            ]
        }),
    }))
    .child(HfTable::element(HfTableProps {
        rows: Prop::bind(move || {
            let rows = bind
                .orders
                .get()
                .iter()
                .map(|o| {
                    vec![
                        o.order_id.clone(),
                        o.symbol.clone(),
                        o.side.clone(),
                        format!("{:?}", o.status),
                        o.price.map(fixtures::fmt).unwrap_or_else(|| "MKT".into()),
                        fixtures::fmt(o.filled_quantity),
                    ]
                })
                .collect::<Vec<_>>();
            if rows.is_empty() {
                vec![vec![
                    "—".to_string(),
                    "the journal is empty (no orders this session)".to_string(),
                ]]
            } else {
                rows
            }
        }),
        header: Prop::from(header),
        align: Prop::from(Some(TableCellAlign::Center)),
    }))
}
