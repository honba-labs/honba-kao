//! Dossiers — the research rag-shelf (Stage 3 Phase A.3 of the gallery-rs
//! porting guide, replacing the stub page).
//!
//! The AI artifact shelf: the research engine's printed briefs (title +
//! content, borrow-cased), one card per strip, with the section's binary
//! verdicts. Deterministic text stands in until the research stream
//! binds over the Phase A.1 seam.

use crate::features::fixtures;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::molecules::quant::artifact_card::{HfAiArtifactCard, HfAiArtifactCardProps};
use honba_kao_ui_core::molecules::quant::filings_card::{HfFilingsCard, HfFilingsCardProps};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use nsh_core::prelude::*;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct DossiersPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// Dossiers — the research brief shelf.
#[component]
pub fn DossiersPage(_props: DossiersPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let lead = fixtures::TICKERS[0].replace("-EQ", "");

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Dossiers".to_string())),
        subtitle: Prop::from(Some(
            "The research engine's printed briefs: one strip per dossier, verdict attached"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(
        HfStatsBar::element(HfStatsBarProps {
            stats: Prop::from(vec![
                StatItem {
                    label: "Briefs".into(),
                    value: "3".into(),
                    is_positive: None,
                },
                StatItem {
                    label: "Engine".into(),
                    value: "honba-nou".into(),
                    is_positive: None,
                },
                StatItem {
                    label: "Fresh".into(),
                    value: "quarter".into(),
                    is_positive: None,
                },
            ]),
        }),
    )
    .child(
        HfAiArtifactCard::element(HfAiArtifactCardProps {
            title: Prop::from(format!("{lead} — segment brief").to_string()),
            content: Prop::from(
                "Retail-to-telecom rotation acknowledged: the ARPU led the LAST four quarters;\nthe cap table reads affordable, with the tower leg over half the print."
                    .to_string(),
            ),
            class: Prop::default(),
        }),
    )
    .child(
        HfFilingsCard::element(HfFilingsCardProps {
            title: Prop::from("Dossier note — O2C deconsolidation".to_string()),
            excerpt: Prop::from(
                "The journalizing prints a systemic narrowing: the receiving entity carried the\nlook-through at par. The declared step-ups are the sector's own."
                    .to_string(),
            ),
            class: Prop::default(),
        }),
    )
    .child(
        HfAiArtifactCard::element(HfAiArtifactCardProps {
            title: Prop::from("Verdict — sector stance".to_string()),
            content: Prop::from(
                "MAINTAIN OVERWEIGHT. The tape's own compression offers the re-ranking window;\nthe desk's own line items are the journal's tie-outs."
                    .to_string(),
            ),
            class: Prop::default(),
        }),
    )
}
