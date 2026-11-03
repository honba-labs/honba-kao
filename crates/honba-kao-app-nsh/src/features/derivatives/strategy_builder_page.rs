//! Strategy Builder — the payoff workbench (Stage 3 Phase A.3 of the
//! gallery-rs porting guide, replacing the stub page).
//!
//! An iron-condor-like vertical spread composed of two
//! [`strategy_leg_row`]s inside [`HfStrategyBuilder`], the payoff curve a
//! real decimal walk (BUY 22870 CE − SELL 23170 CE, net debit) plotted in
//! the pool's [`payoff_chart`]. The `spot`/`pnl` points are `f64` only at
//! the organism's plot boundary — the payoff itself is computed in
//! Decimal and converted once per point (zero-floats invariant). Leg
//! removal is wired to the row's `Remove` event and surfaces in the
//! status strip (the retained row list is rebuilt by the next rebuild of
//! this page, which the shell performs on re-entry).

use crate::features::fixtures;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::molecules::fno::payoff_chart::{
    HfOptionPayoffChart, HfOptionPayoffChartProps, PayoffPoint,
};
use honba_kao_ui_core::molecules::fno::strategy_leg_row::{
    HfStrategyLegRow, HfStrategyLegRowEvent, HfStrategyLegRowProps, StrategyLeg,
};
use honba_kao_ui_core::organisms::strategy_builder::{HfStrategyBuilder, HfStrategyBuilderProps};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use honba_kao_ui_core::theme::tokens::HonbaStyle;
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct StrategyBuilderPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// The demo structure: long call at the money, short call 300 up (per
/// share; the lot and its qty multiply in the payoff).
const LONG_STRIKE: Decimal = dec!(22870);
const SHORT_STRIKE: Decimal = dec!(23170);
const NET_DEBIT: Decimal = dec!(96.50);

/// The spread's payoff per unit at expiry: `max(0, S−L) − max(0, S−H) −
/// debit`, all Decimal.
fn payoff(spot: Decimal) -> Decimal {
    let long = (spot - LONG_STRIKE).max(dec!(0));
    let short = (spot - SHORT_STRIKE).max(dec!(0));
    long - short - NET_DEBIT
}

/// The plot's points: spots from 22,570 to 23,470 in 25-point steps, the
/// leg pnl converted to the chart's `f64` axis once, at the boundary.
fn payoff_points() -> Vec<PayoffPoint> {
    let spots = (22570..=23470).step_by(25);
    spots
        .map(|s| {
            let spot = Decimal::from(s);
            let pnl = payoff(spot) * Decimal::from(150);
            PayoffPoint {
                spot: spot.to_f64().unwrap_or_default(),
                pnl: pnl.to_f64().unwrap_or_default(),
            }
        })
        .collect()
}

/// Strategy Builder — the iron-condor workbench.
#[component]
pub fn StrategyBuilderPage(
    _props: StrategyBuilderPageProps,
    _cx: Scope<NoEvent>,
) -> impl IntoElement {
    let last_removed = signal(None::<String>);

    let max_profit = payoff(SHORT_STRIKE) * Decimal::from(150);
    let max_loss = -NET_DEBIT * Decimal::from(150);

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Strategy Builder".to_string())),
        subtitle: Prop::from(Some(
            "Compose multi-leg spreads, read the payoff arc and the risk rails".to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(HfStatsBar::element(HfStatsBarProps {
        stats: Prop::from(vec![
            StatItem {
                label: "Net Debit".into(),
                value: format!("₹{}", fixtures::fmt(-max_loss)),
                is_positive: Some(false),
            },
            StatItem {
                label: "Max Profit".into(),
                value: format!("₹{}", fixtures::fmt(max_profit)),
                is_positive: Some(true),
            },
            StatItem {
                label: "Lot Multiplier".into(),
                value: "150".into(),
                is_positive: None,
            },
            StatItem {
                label: "Expiry".into(),
                value: "26DEC".into(),
                is_positive: None,
            },
        ]),
    }))
    .child(
        HfStrategyBuilder::element(HfStrategyBuilderProps {
            children: Children::default(),
        })
        .child(HfStrategyLegRow::element(HfStrategyLegRowProps {
            leg: Prop::from(StrategyLeg {
                side: "BUY".to_string(),
                strike: fixtures::fmt(LONG_STRIKE),
                expiry: "26DEC".to_string(),
                lots: "2".to_string(),
            }),
        }))
        .child(
            HfStrategyLegRow::element(HfStrategyLegRowProps {
                leg: Prop::from(StrategyLeg {
                    side: "SELL".to_string(),
                    strike: fixtures::fmt(SHORT_STRIKE),
                    expiry: "26DEC".to_string(),
                    lots: "2".to_string(),
                }),
            })
            .on_event(move |event: &HfStrategyLegRowEvent| {
                if event == &HfStrategyLegRowEvent::Remove {
                    last_removed.set(Some(format!("SELL {SHORT_STRIKE} CE leg removed")));
                }
            }),
        ),
    )
    .child(HfOptionPayoffChart::element(HfOptionPayoffChartProps {
        points: Prop::from(payoff_points()),
        class: Prop::default(),
    }))
    .child(
        text(move || {
            last_removed
                .get()
                .unwrap_or_else(|| "Legs unchanged".to_string())
        })
        .text_size(px(12.0))
        .text_muted(),
    )
}
