//! Asset Allocation — the weight strip (Stage 3 Phase A.3 of the
//! gallery-rs porting guide, replacing the stub page).
//!
//! The pool's allocation bar over a determinant-free Decimal weight
//! walk (equity/debt/cash/gold renormalized to 100 in Decimal), the
//! split table and the shelf's risk meter. The bar's `Allocation` takes
//! `f64` fractions only at the organism boundary (one conversion per
//! weight; no float math app-side).

use crate::features::fixtures;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::molecules::wealth::RiskLevel;
use honba_kao_ui_core::molecules::wealth::asset_allocation_bar::{
    Allocation, HfAssetAllocationBar, HfAssetAllocationBarProps,
};
use honba_kao_ui_core::molecules::wealth::risk_o_meter::{HfRiskOMeter, HfRiskOMeterProps};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use honba_kao_ui_core::theme::tokens::HonbaStyle;
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct AssetAllocationPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// The allocation walk: raw Decimal weights renormalized to 100. Returns
/// the organism's `Allocation` (f64 at the boundary) and the split rows.
fn allocation() -> (Allocation, Vec<Vec<String>>) {
    let raw = (0..4)
        .map(|i| Decimal::from(28 + (i * 41) % 44))
        .collect::<Vec<_>>();
    let total = raw.iter().sum::<Decimal>();
    let parts = raw
        .iter()
        .map(|r| r / total * Decimal::from(100))
        .collect::<Vec<_>>();
    let alloc = Allocation {
        equity: parts[0].to_f64().unwrap_or_default(),
        debt: parts[1].to_f64().unwrap_or_default(),
        cash: parts[2].to_f64().unwrap_or_default(),
        gold: parts[3].to_f64().unwrap_or_default(),
    };
    let rows = ["Equity", "Debt", "Cash", "Gold"]
        .iter()
        .zip(parts.iter())
        .map(|(label, weight)| vec![label.to_string(), format!("{}%", fixtures::fmt(*weight))])
        .collect();
    (alloc, rows)
}

/// Asset Allocation — the weight strip.
#[component]
pub fn AssetAllocationPage(
    _props: AssetAllocationPageProps,
    _cx: Scope<NoEvent>,
) -> impl IntoElement {
    let (alloc, rows) = allocation();

    let header = Some(
        ["Asset", "Weight"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Asset Allocation".to_string())),
        subtitle: Prop::from(Some(
            "The portfolio's weight strip: equity, debt, cash and gold rails".to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(HfStatsBar::element(HfStatsBarProps {
        stats: Prop::from(vec![
            StatItem {
                label: "Instruments".into(),
                value: "4".into(),
                is_positive: None,
            },
            StatItem {
                label: "Rebalance".into(),
                value: "quarterly".into(),
                is_positive: None,
            },
        ]),
    }))
    .child(
        view()
            .flex_row()
            .gap_3()
            .child(HfAssetAllocationBar::element(HfAssetAllocationBarProps {
                alloc: Prop::from(alloc),
            })),
    )
    .child(
        view()
            .flex_row()
            .gap_3()
            .child(HfRiskOMeter::element(HfRiskOMeterProps {
                level: Prop::from(RiskLevel::LowToModerate),
            })),
    )
    .child(
        text("Weights renormalized in Decimal; the bar takes fractions at the boundary only.")
            .text_size(px(12.0))
            .text_muted(),
    )
    .child(HfTable::element(HfTableProps {
        rows: Prop::from(rows),
        header: Prop::from(header),
        align: Prop::default(),
    }))
}
