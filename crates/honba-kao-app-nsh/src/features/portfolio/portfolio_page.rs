//! Portfolio — the real-time holding and margin board (Stage 3 Phase A.3
//! of the gallery-rs porting guide, replacing the stub page).
//!
//! The primary read of `HonbaClientContext`: the account summary strip
//! (equity, margins, P&L off the live `account` signal), the positions
//! and holdings tables off the live `positions`/`holdings` signals, and
//! the collateral organism over the fixture haircut walk. Empty-safe:
//! every table renders its no-rows sentence until the REST snapshot
//! lands.

use crate::features::fixtures;
use crate::shared::contexts::HonbaClientContext;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::organisms::collateral_organism::{
    CollateralRow, HfCollateralOrganism, HfCollateralOrganismProps,
};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps, TableCellAlign};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use honba_kao_ui_core::theme::tokens::HonbaStyle;
use nsh_core::prelude::*;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct PortfolioPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// The collateral fixture walk: (rows, net pledge total % weighted
/// haircut). All Decimal; the organism gets pre-formatted strings, the
/// header strip gets the totals.
fn collateral_walk() -> (Vec<CollateralRow>, Decimal, Decimal) {
    let mut rows = Vec::new();
    let mut net_total = Decimal::default();
    let mut haircut_weighted = Decimal::default();
    let mut value_total = Decimal::default();
    for i in 0..5 {
        let value = Decimal::from(90_000 + (i * 77) % 240_000);
        let haircut = Decimal::from(8 + (i * 7) % 14);
        let net = value * (Decimal::from(100) - haircut) / Decimal::from(100);
        rows.push(CollateralRow {
            asset: fixtures::TICKERS[i].to_string(),
            value: format!("₹{}", fixtures::fmt(value)),
            haircut: format!("{}%", fixtures::fmt(haircut)),
            net_value: format!("₹{}", fixtures::fmt(net)),
        });
        net_total += net;
        haircut_weighted += value * haircut / Decimal::from(100);
        value_total += value;
    }
    let total_haircut = if value_total.is_zero() {
        Decimal::default()
    } else {
        haircut_weighted / value_total * Decimal::from(100)
    };
    (rows, net_total, total_haircut)
}

/// Portfolio — holdings, positions, margins, collateral.
#[component]
pub fn PortfolioPage(_props: PortfolioPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let client = use_context::<HonbaClientContext>().expect("client context");
    let bind = client.clone(); // account strip binding
    let bind_positions = client.clone();
    let bind_holdings = client.clone();

    let (collateral, total_collateral_decimal, total_haircut_decimal) = collateral_walk();

    let position_header = Some(
        ["Symbol", "Side", "Qty", "Entry", "Mark"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );
    let holding_header = Some(
        ["Symbol", "Qty", "Avg", "Last", "P&L", "P&L %"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Portfolio".to_string())),
        subtitle: Prop::from(Some(
            "Balance, margin utilization, FIFO holdings and the live position book".to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(HfStatsBar::element(HfStatsBarProps {
        stats: Prop::bind(move || match bind.account.get() {
            Some(acc) => vec![
                StatItem {
                    label: "Equity".into(),
                    value: format!("₹{}", fixtures::fmt(acc.total_equity)),
                    is_positive: Some(true),
                },
                StatItem {
                    label: "Available".into(),
                    value: format!("₹{}", fixtures::fmt(acc.available_margin)),
                    is_positive: None,
                },
                StatItem {
                    label: "Used".into(),
                    value: format!("₹{}", fixtures::fmt(acc.used_margin)),
                    is_positive: None,
                },
                StatItem {
                    label: "Unrealized".into(),
                    value: format!("₹{}", fixtures::fmt(acc.unrealized_pnl)),
                    is_positive: Some(acc.unrealized_pnl >= Default::default()),
                },
            ],
            None => vec![StatItem {
                label: "Account".into(),
                value: "waiting for snapshot".into(),
                is_positive: None,
            }],
        }),
    }))
    .child(HfTable::element(HfTableProps {
        rows: Prop::bind(move || {
            let rows = bind_positions
                .positions
                .get()
                .iter()
                .map(|p| {
                    vec![
                        p.symbol.clone(),
                        p.side.clone(),
                        fixtures::fmt(p.quantity),
                        fixtures::fmt(p.entry_price),
                        fixtures::fmt(p.current_price),
                    ]
                })
                .collect::<Vec<_>>();
            if rows.is_empty() {
                vec![vec![
                    "—".to_string(),
                    "no open positions".to_string(),
                    String::new(),
                    String::new(),
                    String::new(),
                ]]
            } else {
                rows
            }
        }),
        header: Prop::from(position_header),
        align: Prop::from(Some(TableCellAlign::Center)),
    }))
    .child(HfTable::element(HfTableProps {
        rows: Prop::bind(move || {
            let rows = bind_holdings
                .holdings
                .get()
                .iter()
                .map(|h| {
                    vec![
                        h.symbol.clone(),
                        fixtures::fmt(h.quantity),
                        fixtures::fmt(h.average_price),
                        fixtures::fmt(h.current_price),
                        fixtures::fmt(h.pnl),
                        format!("{}%", fixtures::fmt(h.pnl_pct)),
                    ]
                })
                .collect::<Vec<_>>();
            if rows.is_empty() {
                vec![vec![
                    "—".to_string(),
                    "no equity holdings".to_string(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                ]]
            } else {
                rows
            }
        }),
        header: Prop::from(holding_header),
        align: Prop::from(Some(TableCellAlign::Center)),
    }))
    .child(
        text("The pledged-collateral walk (fixture strand until the margin stream binds)")
            .text_size(px(12.0))
            .text_muted(),
    )
    .child(HfCollateralOrganism::element(HfCollateralOrganismProps {
        rows: Prop::from(collateral),
        total_collateral: Prop::from(format!("₹{}", fixtures::fmt(total_collateral_decimal))),
        total_haircut: Prop::from(format!("{}%", fixtures::fmt(total_haircut_decimal))),
    }))
}
