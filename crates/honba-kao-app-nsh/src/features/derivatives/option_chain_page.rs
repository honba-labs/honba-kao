//! Option Chain — the NIFTY chain with detailed rows (Stage 3 Phase A.3
//! of the gallery-rs porting guide, replacing the stub page).
//!
//! The summary chain organism over the whole strike walk, three detailed
//! [`option_chain_row`]s around the money, the PCR meter and the max-pain
//! badge. Quotes come from the shared Decimal fixtures — per-strike
//! premium = intrinsic + time value walk, integer/Decimal math only. The
//! meters take the pool's `f64`/`Row` props only at the organism
//! boundary (no float math happens app-side); a selected leg's verdict
//! renders reactively from the row's `SelectLeg` event.

use crate::features::fixtures;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::molecules::fno::max_pain_badge::{HfMaxPainBadge, HfMaxPainBadgeProps};
use honba_kao_ui_core::molecules::fno::option_chain_row::{
    HfOptionChainRow, HfOptionChainRowEvent, HfOptionChainRowProps, OptionLegQuote,
};
use honba_kao_ui_core::molecules::fno::pcr_meter::{HfPcrMeter, HfPcrMeterProps};
use honba_kao_ui_core::organisms::option_chain::{HfOptionChain, HfOptionChainProps};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use honba_kao_ui_core::theme::tokens::HonbaStyle;
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct OptionChainPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// The strike walk: NIFTY 22,570 through 23,170 in hundreds.
const STRIKES: [i64; 7] = [22570, 22670, 22770, 22870, 22970, 23070, 23170];

/// The day's spot (the fixtures' base).
fn spot() -> Decimal {
    dec!(22842.50)
}

/// One quote leg for a strike: premium walks up with distance from spot
/// (integer/Decimal only), OI/IV are the deterministic fixture walk.
fn leg(strike: i64, is_call: bool) -> OptionLegQuote {
    let distance = (strike - 22870).abs().max(0);
    let strike_d = Decimal::from(strike);
    let spot_d = spot();
    let intrinsic = if is_call && strike_d < spot_d {
        spot_d - strike_d
    } else if !is_call && strike_d > spot_d {
        strike_d - spot_d
    } else {
        dec!(0)
    };
    // Time value: 40 paise per 100 points away + 180 base.
    let time_value = dec!(1.8) + Decimal::from(distance) / Decimal::from(100) * dec!(0.4);
    let ltp = intrinsic + time_value;
    OptionLegQuote {
        ltp: fixtures::fmt(ltp),
        change_pct: format!(
            "{}%",
            fixtures::fmt(dec!(0.35) * Decimal::from(((2 * distance) % 40) + 5))
        ),
        volume: 12_500 + ((strike as u64 * 937) % 90_000),
        open_interest: 41_000 + ((strike as u64 * 613) % 460_000),
        oi_change: ((strike * 173) % 3_400) - 1_700,
        iv: format!(
            "{}%",
            fixtures::fmt(dec!(9.7) + Decimal::from(distance % 30) / dec!(10))
        ),
        delta: fixtures::fmt(dec!(0.05) + Decimal::from(distance % 90) / dec!(100)),
        gamma: fixtures::fmt(dec!(0.0012) + Decimal::from(distance % 7) / dec!(10_000)),
    }
}

/// The chain-summary rows the organism takes (strike · bid · ask · IV · IO).
fn chain_rows() -> Vec<honba_kao_ui_core::organisms::option_chain::OptionChainRow> {
    use honba_kao_ui_core::organisms::option_chain::OptionChainRow;
    STRIKES
        .iter()
        .map(|s| {
            let call = leg(*s, true);
            let put = leg(*s, false);
            OptionChainRow {
                strike: format!("{s}"),
                bid: call.ltp.clone(),
                ask: put.ltp.clone(),
                iv: call.iv.clone(),
                delta: call.delta.clone(),
            }
        })
        .collect()
}

/// One detailed row element (same strike walk, call/put around money).
fn detail_row(strike: i64) -> impl IntoElement {
    HfOptionChainRow::element(HfOptionChainRowProps {
        strike: Prop::from(format!("{strike}")),
        spot_price: Prop::from(fixtures::fmt(spot())),
        call: Prop::from(leg(strike, true)),
        put: Prop::from(leg(strike, false)),
        max_oi: Prop::from(Some(480_000u64)),
        is_buy: Prop::default(),
    })
}

/// Option Chain — the NIFTY chain, detail rows and sentiment meters.
#[component]
pub fn OptionChainPage(_props: OptionChainPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let selected = signal(None::<String>);

    let detail = (22870, 22770, 22970);

    let detail_rows = view()
        .flex_col()
        .gap_2()
        .w_full()
        .child(
            HfOptionChainRow::element(HfOptionChainRowProps {
                strike: Prop::from(format!("{}", detail.0)),
                spot_price: Prop::from(fixtures::fmt(spot())),
                call: Prop::from(leg(detail.0, true)),
                put: Prop::from(leg(detail.0, false)),
                max_oi: Prop::from(Some(480_000u64)),
                is_buy: Prop::default(),
            })
            .on_event(move |event: &HfOptionChainRowEvent| match event {
                HfOptionChainRowEvent::SelectLeg {
                    strike,
                    is_call,
                    is_buy,
                } => {
                    let side = if *is_call { "CALL" } else { "PUT" };
                    let action = if *is_buy { "BUY" } else { "SELL" };
                    selected.set(Some(format!("{action} {side} {strike} selected")));
                }
            }),
        )
        .child(detail_row(detail.1))
        .child(detail_row(detail.2));

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Option Chain".to_string())),
        subtitle: Prop::from(Some(
            "NIFTY chain — detailed LTP/OI rows around money, PCR and max pain".to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(HfStatsBar::element(HfStatsBarProps {
        stats: Prop::from(vec![
            StatItem {
                label: "Spot".into(),
                value: format!("₹{}", fixtures::fmt(spot())),
                is_positive: Some(true),
            },
            StatItem {
                label: "ATM IV".into(),
                value: "10.7%".into(),
                is_positive: None,
            },
            StatItem {
                label: "PCR".into(),
                value: "0.78".into(),
                is_positive: Some(false),
            },
            StatItem {
                label: "Max Pain".into(),
                value: "23070".into(),
                is_positive: None,
            },
        ]),
    }))
    .child(
        view()
            .flex_row()
            .gap_3()
            .child(
                view()
                    .w(px(260.0))
                    .child(HfPcrMeter::element(HfPcrMeterProps {
                        pcr: Prop::from(0.78),
                        class: Prop::default(),
                    })),
            )
            .child(HfMaxPainBadge::element(HfMaxPainBadgeProps {
                strike: Prop::from(23070.0),
                class: Prop::default(),
            })),
    )
    .child(HfOptionChain::element(HfOptionChainProps {
        underlying: Prop::from("NIFTY50-INDEX".to_string()),
        rows: Prop::from(chain_rows()),
    }))
    .child(detail_rows)
    .child(
        text(move || {
            selected
                .get()
                .unwrap_or_else(|| "Select a leg: press its quote panel".to_string())
        })
        .text_size(px(12.0))
        .text_muted(),
    )
}
