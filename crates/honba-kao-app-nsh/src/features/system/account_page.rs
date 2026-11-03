//! Institutional Account & Settings — trader identity, exchange gateway
//! credentials, API keys, and notification webhooks.
//! Real layout ported onto NSH primitives (Phase 5, system cluster).

use honba_kao_ui_core::primitive::badge::{BadgeVariant, HfBadgeProps, HfStatusChipProps};
use honba_kao_ui_core::primitive::base::button::HfButtonProps;
use honba_kao_ui_core::primitive::currency::HfAssetBadgeProps;
use honba_kao_ui_core::primitive::panel::{HfPanelProps, HfSectionGridProps};
use honba_kao_ui_core::primitive::{
    ButtonSize, ButtonVariant, Currency, HfAssetBadge, HfBadge, HfButton, HfPanel, HfSectionGrid,
    HfStatusChip,
};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use honba_kao_ui_core::theme::tokens::HonbaStyle;
use nsh_core::prelude::*;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct AccountPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// Trader identity, exchange gateway credentials, API keys, and notification webhooks.
#[component]
pub fn AccountPage(_props: AccountPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    HfPage::element(HfPageProps {
        title: Prop::from(Some("Institutional Account & Settings".to_string())),
        subtitle: Prop::from(Some(
            "Trader identity, exchange gateway credentials, API keys, and notification webhooks"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(
        HfSectionGrid::element(HfSectionGridProps {
            cols: Prop::from(2),
            children: Children::default(),
        })
        .child(
            HfPanel::element(HfPanelProps {
                header: Prop::from(Some("Trader Profile".into())),
                children: Children::default(),
            })
            .child(
                view()
                    .flex_col()
                    .gap(px(2.0))
                    .child(field_row("Name", "Priya Iyer"))
                    .child(field_row("Email", "priya@example.in"))
                    .child(field_row("Currency", "INR"))
                    .child(HfStatusChip::element(HfStatusChipProps {
                        status: Prop::from("KYC Verified".to_string()),
                        variant: Prop::from(Some(BadgeVariant::Success)),
                    })),
            ),
        )
        .child(
            HfPanel::element(HfPanelProps {
                header: Prop::from(Some("Base Currency".into())),
                children: Children::default(),
            })
            .child(
                HfAssetBadge::element(HfAssetBadgeProps {
                    currency: Prop::from(Currency::Inr),
                    symbol: Prop::from(None),
                    children: Children::default(),
                })
                .child(text("Indian Rupee")),
            ),
        )
        .child(
            HfPanel::element(HfPanelProps {
                header: Prop::from(Some("Exchange Gateway".into())),
                children: Children::default(),
            })
            .child(
                view()
                    .flex_col()
                    .gap(px(2.0))
                    .child(field_row("Default Broker", "Zerodha Kite"))
                    .child(field_row("Feed", "WebSocket (binary)"))
                    .child(field_row("Order Routing", "Smart Order Router"))
                    .child(
                        HfBadge::element(HfBadgeProps {
                            variant: Prop::from(Some(BadgeVariant::Info)),
                            outline: Prop::from(false),
                            children: Children::default(),
                        })
                        .child(text("Connected")),
                    ),
            ),
        )
        .child(
            HfPanel::element(HfPanelProps {
                header: Prop::from(Some("API Keys".into())),
                children: Children::default(),
            })
            .child(
                view()
                    .flex_col()
                    .gap(px(2.0))
                    .child(field_row("Key ID", "zk_live_…8f3a"))
                    .child(field_row("Scope", "read, place_orders"))
                    .child(field_row("Created", "2026-09-12"))
                    .child(
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
                        .child(text("Rotate Key")),
                    ),
            ),
        ),
    )
    .child(
        HfPanel::element(HfPanelProps {
            header: Prop::from(Some("Notification Webhooks".into())),
            children: Children::default(),
        })
        .child(
            view()
                .flex_col()
                .gap(px(2.0))
                .child(field_row("Order Fills", "https://hooks.example.in/fills"))
                .child(field_row("Risk Breach", "https://hooks.example.in/risk"))
                .child(
                    HfBadge::element(HfBadgeProps {
                        variant: Prop::from(Some(BadgeVariant::Success)),
                        outline: Prop::from(false),
                        children: Children::default(),
                    })
                    .child(text("2 endpoints active")),
                ),
        ),
    )
    .child(
        HfButton::element(HfButtonProps {
            href: Prop::default(),
            size: Prop::from(Some(ButtonSize::Md)),
            variant: Prop::from(Some(ButtonVariant::Primary)),
            full_width: Prop::from(false),
            disabled: Prop::from(false),
            aria_label: Prop::from(None),
            class: Prop::default(),
            children: Children::default(),
        })
        .child(text("Save Changes")),
    )
}

/// Label/value field row inside a settings panel.
fn field_row(label: &'static str, value: &'static str) -> impl IntoElement {
    view()
        .flex_row()
        .justify_between()
        .gap(px(8.0))
        .py(px(1.0))
        .child(text(label).text_muted().text_size(px(12.0)))
        .child(
            text(value)
                .text_high()
                .text_size(px(12.0))
                .font_family("monospace"),
        )
}
