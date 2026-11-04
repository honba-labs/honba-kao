//! Model Lab — the enhancement models' shelf (Stage 3 Phase A.3 of the
//! gallery-rs porting guide, replacing the stub page).
//!
//! Filter pills over the lab's model layers (each pill's `Toggle` flips
//! the page's own selection signal the summary strip reads back), the
//! engine's AI-generated artifact notes, and the model scores table.
//! `honba-nou` prints these — the page renders one strand, deterministic.

use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::molecules::quant::artifact_card::{HfAiArtifactCard, HfAiArtifactCardProps};
use honba_kao_ui_core::molecules::quant::backtest_card::{
    HfBacktestMetricCard, HfBacktestMetricCardProps,
};
use honba_kao_ui_core::molecules::quant::filter_pill::{
    HfFilterPill, HfFilterPillEvent, HfFilterPillProps,
};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use nsh_core::prelude::*;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct ModelLabPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// The lab's model layers (the same slots `honba-nou` enumerates).
const MODELS: [&str; 4] = ["Genetic", "Bayesian", "Walk-forward", "Deflated Sharpe"];

/// One metric tile.
/// The models' score table, recomputed from the live selection signal.
fn score_rows(selected: Signal<Vec<bool>>) -> Vec<Vec<String>> {
    let notes = [
        ("Genetic", "OOS 1.84 EV", "201 gens"),
        ("Bayesian", "OOS 1.61 EV", "600 pts"),
        ("Walk-forward", "OOS 1.42 EV", "6 folds"),
        ("Deflated Sharpe", "DSR 0.13", "signif 3σ"),
    ];
    let flags = selected.get();
    let mut rows = Vec::new();
    for (i, note) in notes.iter().enumerate() {
        if flags[i] {
            rows.push(vec![
                note.0.to_string(),
                note.1.to_string(),
                note.2.to_string(),
            ]);
        }
    }
    rows
}

/// Model Lab — the enhancer's shelf.
#[component]
pub fn ModelLabPage(_props: ModelLabPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let selected = signal(vec![true, false, true, false]);
    let selected_for_rows = selected;

    let pills = MODELS
        .iter()
        .enumerate()
        .map(|(i, model)| {
            let toggle = selected;
            HfFilterPill::element(HfFilterPillProps {
                label: Prop::from(model.to_string()),
                active: Prop::bind(move || selected.get()[i]),
            })
            .on_event(move |_event: &HfFilterPillEvent| {
                // The pill never flips itself: the page owns the state.
                toggle.update(|v| v[i] = !v[i]);
            })
        })
        .collect::<Vec<_>>();

    let header = Some(
        ["Model", "Out-of-sample", "Budget"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Model Lab".to_string())),
        subtitle: Prop::from(Some(
            "Automatic Strategy Enhancer: the genetic, Bayesian and walk-forward layers"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(
        HfStatsBar::element(HfStatsBarProps {
            stats: Prop::from(vec![
                StatItem {
                    label: "Engine".into(),
                    value: "honba-nou".into(),
                    is_positive: None,
                },
                StatItem {
                    label: "Replayable".into(),
                    value: "SHA-256 seeded".into(),
                    is_positive: None,
                },
                StatItem {
                    label: "Selections".into(),
                    value: "4 layers".into(),
                    is_positive: None,
                },
            ]),
        }),
    )
    .child(view().w(px(230.0)).child(
        HfBacktestMetricCard::element(HfBacktestMetricCardProps {
            label: Prop::from("Lab verdict".to_string()),
            value: Prop::from("KEEP".to_string()),
            positive: Prop::from(true),
        }),
    ))
    .child(
        view()
            .flex_row()
            .flex_wrap()
            .gap(px(8.0))
            .children(pills.into_iter().map(|pill| view().child(pill))),
    )
    .child(
        HfAiArtifactCard::element(HfAiArtifactCardProps {
            title: Prop::from("Enhancer notes".to_string()),
            content: Prop::from(
                "Round 12 synthesis: the genetic layer carried the edge in trending regimes\n(the GA's OOS exceeded the point-estimate by 1.84 EV), while the Bayesian\nlayer trimmed the parameter footprint 18%. Recommended: keep both."
                    .to_string(),
            ),
            class: Prop::default(),
        }),
    )
    .child(
        HfTable::element(HfTableProps {
            rows: Prop::bind(move || score_rows(selected_for_rows)),
            header: Prop::from(header),
            align: Prop::default(),
        }),
    )
}
