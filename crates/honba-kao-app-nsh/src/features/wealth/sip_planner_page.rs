//! SIP Planner — the monthly-plan workbench (Stage 3 Phase A.3 of the
//! gallery-rs porting guide, replacing the stub page).
//!
//! The pool's SIP calculator card over deterministic Decimal loads:
//! monthly amount, horizon and expected annual return render; the card's
//! own change events re-dispatch into the page's signal and re-render
//! the strip below (the card owns its pill wiring, the page owns the
//! state). No float math — the events are the app's Decimal updates.

use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::molecules::wealth::sip_calculator::{
    HfSipCalculatorCard, HfSipCalculatorCardProps, HfSipCalculatorEvent,
};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use nsh_core::prelude::*;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct SipPlannerPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// SIP Planner — the monthly plan.
#[component]
pub fn SipPlannerPage(_props: SipPlannerPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let monthly = signal("10,000".to_string());
    let years = signal("10".to_string());
    let annual_return = signal("12%".to_string());

    let monthly_head = monthly;
    let years_head = years;
    let return_head = annual_return;

    HfPage::element(HfPageProps {
        title: Prop::from(Some("SIP Planner".to_string())),
        subtitle: Prop::from(Some(
            "The monthly plan: amounts, horizon and the expected-return print".to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(
        HfSipCalculatorCard::element(HfSipCalculatorCardProps {
            monthly: Prop::bind(move || monthly.get()),
            years: Prop::bind(move || years.get()),
            annual_return: Prop::bind(move || annual_return.get()),
        })
        .on_event(move |event: &HfSipCalculatorEvent| match event {
            HfSipCalculatorEvent::MonthlyChange(value) => monthly.set(value.clone()),
            HfSipCalculatorEvent::YearsChange(value) => years.set(value.clone()),
            HfSipCalculatorEvent::AnnualReturnChange(value) => {
                annual_return.set(value.clone());
            }
        }),
    )
    .child(HfStatsBar::element(HfStatsBarProps {
        stats: Prop::bind(move || {
            vec![
                StatItem {
                    label: "Monthly".into(),
                    value: format!("₹{}", monthly_head.get()),
                    is_positive: None,
                },
                StatItem {
                    label: "Horizon".into(),
                    value: format!("{} yr", years_head.get()),
                    is_positive: None,
                },
                StatItem {
                    label: "Expected".into(),
                    value: return_head.get(),
                    is_positive: Some(true),
                },
            ]
        }),
    }))
}
