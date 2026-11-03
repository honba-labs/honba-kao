//! Ops — the operations console (Stage 3 Phase A.3 of the gallery-rs
//! porting guide, replacing the stub page).
//!
//! The session's operational rails: broker health off the live
//! `brokers` signal, the OMS/feed connection lights, latency, and the
//! desk note. Everything the context carries renders; the deeper
//! runbooks stay the desk's PDFs until the ops stream lands.

use crate::shared::contexts::HonbaClientContext;
use honba_kao_client::dto::portfolio::BrokerConnection;
use honba_kao_ui_core::molecules::core::stats_bar::{HfStatsBar, HfStatsBarProps, StatItem};
use honba_kao_ui_core::primitive::table::{HfTable, HfTableProps, TableCellAlign};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use honba_kao_ui_core::theme::tokens::HonbaStyle;
use nsh_core::prelude::*;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct OpsPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// One broker's session row (id, connection lamp, latency).
fn broker_rows(brokers: Vec<BrokerConnection>) -> Vec<Vec<String>> {
    brokers
        .iter()
        .map(|b| {
            vec![
                format!("{} ({})", b.name, b.broker_id),
                (if b.is_connected {
                    "connected"
                } else {
                    "offline"
                })
                .to_string(),
                b.latency_ms
                    .map_or("—".to_string(), |ms| format!("{ms} ms")),
            ]
        })
        .collect()
}

/// Ops — the operations console.
#[component]
pub fn OpsPage(_props: OpsPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let client = use_context::<HonbaClientContext>().expect("client context");
    let bind_strip = client.clone();
    let bind_table = client.clone();
    let bind_note = client;

    let header = Some(
        ["Broker", "Session", "Latency"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    HfPage::element(HfPageProps {
        title: Prop::from(Some("Ops".to_string())),
        subtitle: Prop::from(Some(
            "The session's operational rails: broker health, connection lights, latency"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(
        HfStatsBar::element(HfStatsBarProps {
            stats: Prop::bind(move || {
                vec![
                    StatItem {
                        label: "OMS".into(),
                        value: (if bind_strip.oms_connected.get() { "up" } else { "down" }).into(),
                        is_positive: Some(bind_strip.oms_connected.get()),
                    },
                    StatItem {
                        label: "Feed".into(),
                        value: (if bind_strip.feed_connected.get() {
                            "live"
                        } else {
                            "idle"
                        })
                            .into(),
                        is_positive: Some(bind_strip.feed_connected.get()),
                    },
                    StatItem {
                        label: "Latency".into(),
                        value: bind_strip
                            .latency_ms
                            .get()
                            .map_or("—".to_string(), |ms| format!("{ms} ms")),
                        is_positive: None,
                    },
                ]
            }),
        }),
    )
    .child(
        HfTable::element(HfTableProps {
            rows: Prop::bind(move || {
                let rows = broker_rows(bind_table.brokers.get());
                if rows.is_empty() {
                    vec![vec![
                        "—".to_string(),
                        "no broker sessions".to_string(),
                        String::new(),
                    ]]
                } else {
                    rows
                }
            }),
            header: Prop::from(header),
            align: Prop::from(Some(TableCellAlign::Center)),
        }),
    )
    .child(
        text(move || {
            let oms = bind_note.oms_connected.get();
            let feed = bind_note.feed_connected.get();
            format!("OMS lane {oms} · feed lane {feed}: the desk keeps the offline mirror identical to the live one.")
        })
        .text_size(px(12.0))
        .text_muted(),
    )
}
