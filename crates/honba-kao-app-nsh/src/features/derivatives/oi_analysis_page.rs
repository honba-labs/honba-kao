//! OI Analysis — the open-interest skew board (Stage 3 Phase A.3 of the
//! gallery-rs porting guide, replacing the stub page).
//!
//! The strike-level OI ladder (call vs put build-up, change columns), the
//! pool's [`HfOiBar`] call/put share rail, and the [`HfColorBar`]
//! exposure strip whose segments are percentages of the session OI. The
//! bars take `f32`/`u64` fractions only at the pool's boundary — the
//! aggregation math is integer/Decimal. `oi_change` is the fixture walk
//! signed so the ladder reads like a real shift day.

use crate::features::fixtures;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::primitive::separator::{
    ColorBarSegment, HfColorBar, HfColorBarProps, HfOiBar, HfOiBarProps,
};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps, TableCellAlign};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use honba_kao_ui_core::theme::tokens::HonbaStyle;
use nsh_core::prelude::*;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct OiAnalysisPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// One ladder row of the OI walk: the tuple economics (strike, call OI,
/// put OI, OI change, call share) HfTable renders and HfColorBar scales.
fn oi_rows() -> Vec<(i64, u64, u64, i64, f32)> {
    (0..9)
        .map(|row| {
            let strike = (22570 + row * 100) as i64;
            let call = 41_000 + ((strike as u64).wrapping_mul(613) % 460_000);
            let put = 38_000 + ((strike as u64).wrapping_mul(743) % 500_000);
            let change = ((strike * 173) % 3_400) - 1_700;
            let share = (call as f32 / (call + put) as f32).clamp(0.2, 0.8);
            (strike, call, put, change, share)
        })
        .collect()
}

/// OI Analysis — the open-interest skew board.
#[component]
pub fn OiAnalysisPage(_props: OiAnalysisPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let rows = oi_rows();
    let call_total: u64 = rows.iter().map(|r| r.1).sum();
    let put_total: u64 = rows.iter().map(|r| r.2).sum();
    let call_share = (call_total as f32 / (call_total + put_total) as f32).clamp(0.2, 0.8);

    let silver = 0x38bdf8;
    let rose = 0xef4444;

    let header = Some(
        [
            "Strike".to_string(),
            "Call OI".to_string(),
            "Put OI".to_string(),
            "OI Δ".to_string(),
            "Call %".to_string(),
        ]
        .to_vec(),
    );
    let table_rows = rows
        .iter()
        .map(|(strike, call, put, change, _share)| {
            let sign = (if *change >= 0 { "+" } else { "" }).to_string();
            vec![
                format!("{strike}"),
                fixtures::thousands(*call as i64),
                fixtures::thousands(*put as i64),
                sign + &fixtures::thousands(*change),
                // Integer percent of the total (no floats): 100·call/total.
                format!("{}%", (*call * 100) / (*call + *put)),
            ]
        })
        .collect::<Vec<_>>();

    HfPage::element(HfPageProps {
        title: Prop::from(Some("OI Analysis".to_string())),
        subtitle: Prop::from(Some(
            "Where the session's open interest is built: the strike ladder, its shift and skew"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(HfStatsBar::element(HfStatsBarProps {
        stats: Prop::from(vec![
            StatItem {
                label: "Call OI".into(),
                value: fixtures::thousands(call_total as i64),
                is_positive: Some(true),
            },
            StatItem {
                label: "Put OI".into(),
                value: fixtures::thousands(put_total as i64),
                is_positive: Some(false),
            },
            StatItem {
                label: "PCR".into(),
                value: "0.78".into(),
                is_positive: None,
            },
            StatItem {
                label: "Max OI".into(),
                value: fixtures::thousands(
                    rows.iter().map(|r| r.1).max().unwrap_or_default() as i64
                ),
                is_positive: None,
            },
        ]),
    }))
    .child(
        view()
            .flex_col()
            .gap_2()
            .w_full()
            .child(
                text("Call / put share")
                    .text_size(px(13.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_muted(),
            )
            .child(HfOiBar::element(HfOiBarProps {
                call_pct: Prop::from(call_share),
            }))
            .child(HfColorBar::element(HfColorBarProps {
                segments: Prop::from(vec![
                    ColorBarSegment {
                        color: silver,
                        pct: call_share,
                    },
                    ColorBarSegment {
                        color: rose,
                        pct: 1.0 - call_share,
                    },
                ]),
                vertical: Prop::from(false),
            })),
    )
    .child(HfTable::element(HfTableProps {
        rows: Prop::from(table_rows),
        header: Prop::from(header),
        align: Prop::from(Some(TableCellAlign::Center)),
    }))
}
