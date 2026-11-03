//! Risk — the pre-trade gate board (Stage 3 Phase A.3 of the gallery-rs
//! porting guide, replacing the stub page).
//!
//! `honba-mon`'s live print: OTR counters, algo-id state, circuit
//! breakers, price-band and freeze rails, and the liquidation warnings
//! the SSE risk stream feeds. The strip binds the context's
//! `liquidation_warnings` signal (empty-safe), the rails walking
//! deterministically until their stream lands.

use crate::features::fixtures;
use crate::shared::contexts::HonbaClientContext;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::molecules::quant::artifact_card::{HfAiArtifactCard, HfAiArtifactCardProps};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps, TableCellAlign};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use honba_kao_ui_core::theme::tokens::HonbaStyle;
use nsh_core::prelude::*;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct RiskPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// The gate's rails table (rail, limit, live, state).
fn rail_rows() -> Vec<Vec<String>> {
    [
        ("OTR counter", "5/session", "2", "ARMED"),
        ("Price band", "± 20%", "± 2.1%", "OK"),
        ("Freeze quantity", "25,000 EQ", "1,900", "OK"),
        ("Algo id", "SEBI-TRADE-040", "registered", "OK"),
        ("Session kill switch", "manual", "not pressed", "OK"),
        ("Margin shortfall", "none", "0.0%", "OK"),
    ]
    .iter()
    .map(|(a, b, c, d)| vec![a.to_string(), b.to_string(), c.to_string(), d.to_string()])
    .collect()
}

/// Risk — the regulatory gate's board.
#[component]
pub fn RiskPage(_props: RiskPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let client = use_context::<HonbaClientContext>().expect("client context");
    let bind = client.clone();
    let bind_strip = client;

    let header = Some(
        ["Rail", "Limit", "Live", "State"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Risk".to_string())),
        subtitle: Prop::from(Some(
            "The pre-trade gate: OTR, algo id, bands, freeze limits and liquidation warnings"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(
        HfStatsBar::element(HfStatsBarProps {
            stats: Prop::bind(move || {
                let warnings = bind_strip.liquidation_warnings.get();
                vec![
                    StatItem {
                        label: "Gate".into(),
                        value: "ARMED".into(),
                        is_positive: Some(true),
                    },
                    StatItem {
                        label: "Liquidation warnings".into(),
                        value: format!("{}", warnings.len()),
                        is_positive: Some(warnings.is_empty()),
                    },
                    StatItem {
                        label: "Kill switch".into(),
                        value: "not pressed".into(),
                        is_positive: None,
                    },
                ]
            }),
        }),
    )
    .child(
        view()
            .flex_col()
            .gap_2()
            // The warnings strip: one line per live warning (empty-safe).
            .child(
                text(move || {
                    let warnings = bind.liquidation_warnings.get();
                    if warnings.is_empty() {
                        "No liquidation warnings on the board.".to_string()
                    } else {
                        warnings
                            .iter()
                            .map(|w| {
                                format!(
                                    "{} · pos {} · liq ₹{} · buffer {}%",
                                    w.symbol,
                                    fixtures::fmt(w.position_size),
                                    fixtures::fmt(w.liquidation_price),
                                    fixtures::fmt(w.buffer_pct),
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    }
                })
                .font_family("monospace")
                .text_size(px(12.0))
                .text_muted(),
            ),
    )
    .child(
        HfTable::element(HfTableProps {
            rows: Prop::from(rail_rows()),
            header: Prop::from(header),
            align: Prop::from(Some(TableCellAlign::Center)),
        }),
    )
    .child(
        HfAiArtifactCard::element(HfAiArtifactCardProps {
            title: Prop::from("Gate memo (A.3)".to_string()),
            content: Prop::from(
                "The gate is synchronous and non-bypassable: every AI decision re-parses the\nhonba_core::Decision grammar and re-enters the clamp before the buffer."
                    .to_string(),
            ),
            class: Prop::default(),
        }),
    )
}
