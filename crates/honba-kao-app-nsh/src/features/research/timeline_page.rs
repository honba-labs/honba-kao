//! Timeline — the research chronology (Stage 3 Phase A.3 of the
//! gallery-rs porting guide, replacing the stub page).
//!
//! The season's research events in order: the printed briefs' schedule
//! table over the fixture universe (session dates, scrip, paper,
//! verdict). The table is the pool's; the strip above states the desk's
//! season window.

use crate::features::fixtures;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps, TableCellAlign};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use nsh_core::prelude::*;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct TimelinePageProps {
    #[prop(default)]
    pub _unused: String,
}

/// The chronology rows: the quarter's briefs in fixture order (date,
/// scrip, paper kind, verdict). Deterministic; the research stream's
/// real prints bind over the Phase A.1 seam.
fn timeline_rows() -> Vec<Vec<String>> {
    fixtures::TICKERS
        .iter()
        .take(8)
        .enumerate()
        .map(|(i, ticker)| {
            let month = ["AUG", "SEP", "OCT", "NOV"][i % 4];
            let paper = ["Initiation", "Note", "Result review", "Verdict"][i % 4];
            let verdict = ["KEPT", "UNDER REVIEW", "KEPT", "ARCHIVED"][i % 4];
            vec![
                format!("{} 2026", month),
                ticker.to_string(),
                paper.to_string(),
                verdict.to_string(),
            ]
        })
        .collect()
}

/// Timeline — the research chronology.
#[component]
pub fn TimelinePage(_props: TimelinePageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let header = Some(
        ["Date", "Scrip", "Paper", "Verdict"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Timeline".to_string())),
        subtitle: Prop::from(Some(
            "The season's research chronology: briefs as the desk printed them".to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(HfStatsBar::element(HfStatsBarProps {
        stats: Prop::from(vec![
            StatItem {
                label: "Season".into(),
                value: "AUG–NOV 2026".into(),
                is_positive: None,
            },
            StatItem {
                label: "Papers".into(),
                value: "8".into(),
                is_positive: None,
            },
        ]),
    }))
    .child(HfTable::element(HfTableProps {
        rows: Prop::from(timeline_rows()),
        header: Prop::from(header),
        align: Prop::from(Some(TableCellAlign::Center)),
    }))
}
