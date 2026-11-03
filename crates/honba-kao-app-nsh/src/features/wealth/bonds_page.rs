//! Bonds — the fixed-income shelf (Stage 3 Phase A.3 of the gallery-rs
//! porting guide, replacing the stub page).
//!
//! The bond ladder's Decimal walk: issue, rating, coupon, yield-to-
//! maturity and residual tenor, plus the ladder's totals. The yields
//! and coupons stay Decimal; the page never floats.

use crate::features::fixtures;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct BondsPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// The ladder: (issuer, rating, coupon %, YTM %, tenor yrs). Coupons and
/// yields walk deterministically in Decimal half-points.
fn ladder_rows() -> Vec<Vec<String>> {
    let issues = [
        ("G-SEC 7.1 GOV 2034", "SOV"),
        ("HDFC BCE BOND 2033", "AAA"),
        ("SBI INFRA 2032", "AAA"),
        ("RECL LTD 2031", "AA+"),
        ("MUTHOOT FIN 2029", "AA"),
        ("TATA CAPS 2030", "AAA"),
        ("PFC LTD 2036", "AA+"),
    ];
    issues
        .iter()
        .enumerate()
        .map(|(i, (issuer, rating))| {
            let coupon = dec!(5.5) + Decimal::from(i * 5) / dec!(10);
            let ytm = coupon + dec!(0.4) + Decimal::from((3 * i) % 5) / dec!(10);
            let tenor = Decimal::from(3 + (i * 7) % 9);
            vec![
                issuer.to_string(),
                rating.to_string(),
                format!("{}%", fixtures::fmt(coupon)),
                format!("{}%", fixtures::fmt(ytm)),
                format!("{}y", fixtures::fmt(tenor)),
            ]
        })
        .collect()
}

/// Bonds — the fixed-income ladder.
#[component]
pub fn BondsPage(_props: BondsPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let header = Some(
        ["Issue", "Rating", "Coupon", "YTM", "Tenor"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Bonds".to_string())),
        subtitle: Prop::from(Some(
            "The fixed-income ladder: sovereign to AA, coupons and yield-to-maturity Decimal precision"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(
        HfStatsBar::element(HfStatsBarProps {
            stats: Prop::from(vec![
                StatItem {
                    label: "Issues".into(),
                    value: "7".into(),
                    is_positive: None,
                },
                StatItem {
                    label: "Lowest rating".into(),
                    value: "AA".into(),
                    is_positive: Some(false),
                },
                StatItem {
                    label: "Avg tenor".into(),
                    value: "6.6y".into(),
                    is_positive: None,
                },
            ]),
        }),
    )
    .child(
        HfTable::element(HfTableProps {
            rows: Prop::from(ladder_rows()),
            header: Prop::from(header),
            align: Prop::default(),
        }),
    )
}
