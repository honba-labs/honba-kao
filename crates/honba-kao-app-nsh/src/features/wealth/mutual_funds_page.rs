//! Mutual Funds — the fund shelf (Stage 3 Phase A.3 of the gallery-rs
//! porting guide, replacing the stub page).
//!
//! The dossier organism (the pool's fund card) with the shelf's MATIC
//! stats, the rolling-returns visualizer, and the funds' risk-O-meter.
//! Percentages come from the Decimal fixture walk; the pool's
//! `f64`-meter props cross only at the organism boundary.

use crate::features::fixtures;
use honba_kao_ui_core::molecules::wealth::overlap_pill::HfPortfolioOverlapPillProps;
use honba_kao_ui_core::molecules::wealth::risk_o_meter::HfRiskOMeterProps;
use honba_kao_ui_core::molecules::wealth::rolling_returns::HfRollingReturnsVisualizerProps;
use honba_kao_ui_core::molecules::wealth::{HfPortfolioOverlapPill, HfRiskOMeter};
use honba_kao_ui_core::molecules::wealth::{HfRollingReturnsVisualizer, RiskLevel};
use honba_kao_ui_core::organisms::mutual_fund_dossier::{
    HfMutualFundDossier, HfMutualFundDossierProps,
};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use honba_kao_ui_core::theme::tokens::HonbaStyle;
use nsh_core::prelude::*;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct MutualFundsPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// Mutual Funds — the wealth shelf.
#[component]
pub fn MutualFundsPage(_props: MutualFundsPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let nav_per_ticker =
        fixtures::fmt(Decimal::from(248) + Decimal::from((7 * 71) % 340) / Decimal::from(10));

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Mutual Funds".to_string())),
        subtitle: Prop::from(Some(
            "The fund shelf: dossiers, rolling returns and the risk meters".to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(
        HfMutualFundDossier::element(HfMutualFundDossierProps {
            fund_name: Prop::from("Honba Bluechip Direct Growth".to_string()),
            nav: Prop::from(format!("₹{nav_per_ticker}")),
            children: Children::default(),
        })
        .child(
            view()
                .flex_col()
                .gap_2()
                .child(
                    text("AUM ₹12,400 cr · TER 0.68% · 3Y return 21.4%")
                        .text_size(px(12.0))
                        .text_muted(),
                )
                .child(
                    HfRollingReturnsVisualizer::element(
                        HfRollingReturnsVisualizerProps {
                            fund: Prop::from(21.4),
                            benchmark: Prop::from(16.9),
                            class: Prop::default(),
                        },
                    ),
                ),
        ),
    )
    .child(
        view()
            .flex_row()
            .gap_3()
            .child(
                HfRiskOMeter::element(HfRiskOMeterProps {
                    level: Prop::from(RiskLevel::Moderate),
                }),
            )
            .child(
                HfPortfolioOverlapPill::element(HfPortfolioOverlapPillProps {
                    overlap: Prop::from(42.0),
                    class: Prop::default(),
                }),
            ),
    )
    .child(
        text("The shelf's run: dossier nav and overlaps walk deterministically until the wealth stream binds.")
            .text_size(px(12.0))
            .text_muted(),
    )
}
