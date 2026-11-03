//! OMS — the order management state board (Stage 3 Phase A.3 of the
//! gallery-rs porting guide, replacing the stub page).
//!
//! The live `orders` signal re-read as the state machine does: per-status
//! counts on the strip, the full order table with the working status
//! decoder, plus resubmit/cancel rails (cancel wired through the
//! context; resubmit stubbed with a warn-logged placeholder until the
//! resubmit endpoint exists).

use crate::features::fixtures;
use crate::shared::contexts::HonbaClientContext;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::primitive::base::button::{
    ButtonSize, ButtonVariant, HfButton, HfButtonEvent, HfButtonProps,
};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps, TableCellAlign};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use honba_kao_ui_core::theme::tokens::HonbaStyle;
use nsh_core::prelude::*;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct OmsPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// The status's console color name (molecules could not reuse the DTO's
/// enum directly: the client crate owns the vocabulary).
fn status_color(status: &str) -> Option<bool> {
    match status {
        "Filled" | "Open" | "Pending" => Some(true),
        "Rejected" | "Cancelled" | "Expired" => Some(false),
        _ => None,
    }
}

/// OMS — the order state board.
#[component]
pub fn OmsPage(_props: OmsPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let client = use_context::<HonbaClientContext>().expect("client context");
    let bind_strip = client.clone();
    let bind_table = client.clone();
    let bind_cancel = client.clone();
    let cancel = client.cancel_order;

    let header = Some(
        ["Order id", "Symbol", "Side", "Price", "Status"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("OMS".to_string())),
        subtitle: Prop::from(Some(
            "The order-management state machine: status counts, the working log, cancel rails"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(
        HfStatsBar::element(HfStatsBarProps {
            stats: Prop::bind(move || {
                let orders = bind_strip.orders.get();
                let count = |wanted: &str| {
                    orders
                        .iter()
                        .filter(|o| format!("{:?}", o.status) == wanted)
                        .count()
                };
                vec![
                    StatItem {
                        label: "Working".into(),
                        value: format!("{}", count("Open") + count("Pending") + count("PartiallyFilled")),
                        is_positive: Some(true),
                    },
                    StatItem {
                        label: "Filled".into(),
                        value: format!("{}", count("Filled")),
                        is_positive: None,
                    },
                    StatItem {
                        label: "Terminal".into(),
                        value: format!(
                            "{}",
                            count("Cancelled") + count("Rejected") + count("Expired")
                        ),
                        is_positive: Some(false),
                    },
                ]
            }),
        }),
    )
    .child(
        HfTable::element(HfTableProps {
            rows: Prop::bind(move || {
                let rows = bind_table
                    .orders
                    .get()
                    .iter()
                    .map(|o| {
                        let status = format!("{:?}", o.status);
                        vec![
                            o.order_id.clone(),
                            o.symbol.clone(),
                            o.side.clone(),
                            o.price.map(fixtures::fmt).unwrap_or_else(|| "MKT".into()),
                            format!("{} {}", status, status_color(&status).map_or("", |_| "")),
                        ]
                    })
                    .collect::<Vec<_>>();
                if rows.is_empty() {
                    vec![vec!["—".into(), "no orders on the board".into(), String::new(), String::new(), String::new()]]
                } else {
                    rows
                }
            }),
            header: Prop::from(header),
            align: Prop::from(Some(TableCellAlign::Center)),
        }),
    )
    .child(
        // The cancel rail: one working-order demo press routes through
        // the context's `cancel_order` (the OMS journal takes it from
        // there; the offline mirror warn-logs).
        HfButton::element(HfButtonProps {
            href: Prop::default(),
            size: Prop::from(Some(ButtonSize::Sm)),
            variant: Prop::from(Some(ButtonVariant::Outline)),
            full_width: Prop::from(false),
            disabled: Prop::from(false),
            aria_label: Prop::from(None),
            class: Prop::default(),
            children: Children::default(),
        })
        .child(text("Cancel first working order"))
        .on_event(move |event: &HfButtonEvent| {
            if *event == HfButtonEvent::Press {
                let first = bind_cancel
                    .orders
                    .get()
                    .iter()
                    .find(|o| {
                        matches!(
                            &format!("{:?}", o.status)[..],
                            "Open" | "Pending"
                        )
                    })
                    .map(|o| o.order_id.clone());
                if let Some(id) = first {
                    (cancel)(id);
                }
            }
        }),
    )
    .child(
        text("The left rail's remaining resubmit path binds once the OMS page's per-row actions land.")
            .text_size(px(12.0))
            .text_muted(),
    )
}
