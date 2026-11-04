//! Greeks Heatmap — the greeks panel around money (Stage 3 Phase A.3 of
//! the gallery-rs porting guide, replacing the stub page).
//!
//! The ATM [`greeks_group`] organism (the pool's five-decimal rail) over
//! the full greek matrix as a table: every strike × greek cell walks
//! deterministically via the integer hash; values are built with
//! `Decimal::new(mantissa, 2)` — no float math on this side. The
//! borderline-regime state is surfaced with the market regime badge.

use honba_kao_ui_core::molecules::fno::greeks_group::{Greeks, HfGreeksGroup, HfGreeksGroupProps};
use honba_kao_ui_core::molecules::quant::market_regime_badge::{
    HfMarketRegimeBadge, HfMarketRegimeBadgeProps, Regime,
};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps, TableCellAlign};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use nsh_core::prelude::*;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct GreeksHeatmapPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// The greek columns the matrix shows.
const GREEK_COLS: [&str; 5] = ["Delta", "Gamma", "Theta", "Vega", "Rho"];

/// One cell of the matrix: a signed two-dp walk derived from the row and
/// column indexes (integer math only; mirrors the fixture hash).
fn cell_value(row: usize, col: usize) -> String {
    let signed = (row * 17 + col * 31) % 9;
    let mantissa = ((row + 1) * 31 + col * 97) % 100;
    let sign = if signed < 4 { "-" } else { "" };
    format!("{sign}{}", Decimal::new(mantissa as i64, 2))
}

/// The matrix rows: first nine strikes, five greeks.
fn heatmap_rows() -> Vec<Vec<String>> {
    (0..9)
        .map(|row| {
            let mut line = vec![format!("{}", 22570 + row * 100)];
            line.extend((0..GREEK_COLS.len()).map(|col| cell_value(row, col)));
            line
        })
        .collect()
}

/// Greeks Heatmap — the option greeks rails around the position.
#[component]
pub fn GreeksHeatmapPage(_props: GreeksHeatmapPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let header = Some(
        ["Strike"]
            .into_iter()
            .chain(GREEK_COLS.iter().copied())
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Greeks Heatmap".to_string())),
        subtitle: Prop::from(Some(
            "Delta/Gamma/Theta/Vega/Rho — the position's greek rails around money".to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(
        view()
            .flex_row()
            .gap_3()
            .child(HfMarketRegimeBadge::element(HfMarketRegimeBadgeProps {
                regime: Prop::from(Regime::Range),
            })),
    )
    .child(HfGreeksGroup::element(HfGreeksGroupProps {
        greeks: Prop::from(Greeks {
            delta: 0.42,
            gamma: 0.0012,
            theta: -18.75,
            vega: 9.4,
            rho: 2.6,
        }),
    }))
    .child(HfTable::element(HfTableProps {
        rows: Prop::from(heatmap_rows()),
        header: Prop::from(header),
        align: Prop::from(Some(TableCellAlign::Center)),
    }))
}
