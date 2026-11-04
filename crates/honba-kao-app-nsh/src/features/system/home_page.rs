//! Home dashboard — institutional financial workstation.
//! Real layout ported onto NSH primitives/molecules (Phase 5, system cluster).

use honba_kao_ui_core::molecules::core::stats_bar::HfStatsBarProps;
use honba_kao_ui_core::molecules::core::{HfStatsBar, StatItem};
use honba_kao_ui_core::primitive::badge::{BadgeVariant, HfBadgeProps, HfStatusChipProps};
use honba_kao_ui_core::primitive::base::button::HfButtonProps;
use honba_kao_ui_core::primitive::currency::HfAssetBadgeProps;
use honba_kao_ui_core::primitive::panel::HfSectionGridProps;
use honba_kao_ui_core::primitive::{
    ButtonSize, ButtonVariant, Currency, HfAssetBadge, HfBadge, HfButton, HfCard, HfSectionGrid,
    HfStatusChip,
};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use honba_kao_ui_core::theme::tokens::HonbaStyle;
use nsh_core::prelude::*;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct HomePageProps {
    #[prop(default)]
    pub _unused: String,
}

/// Institutional Financial Workstation — executive summary, market indices,
/// portfolio overview, and direct workstation entry.
#[component]
pub fn HomePage(_props: HomePageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    HfPage::element(HfPageProps {
        title: Prop::from(Some("Institutional Financial Workstation".to_string())),
        subtitle: Prop::from(Some(
            "Executive summary, market indices, portfolio overview, and direct workstation entry"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(
        HfStatsBar::element(HfStatsBarProps {
            stats: Prop::from(vec![
                StatItem {
                    label: "Portfolio Value".into(),
                    value: "₹2,84,51,234.50".into(),
                    is_positive: Some(true),
                },
                StatItem {
                    label: "Day P&L".into(),
                    value: "+₹12,450.00".into(),
                    is_positive: Some(true),
                },
                StatItem {
                    label: "Margin Utilised".into(),
                    value: "34.2%".into(),
                    is_positive: None,
                },
                StatItem {
                    label: "Open Positions".into(),
                    value: "12".into(),
                    is_positive: None,
                },
            ]),
        }),
    )
    .child(
        view()
            .flex_col()
            .gap(px(4.0))
            .w_full()
            .child(
                HfSectionGrid::element(HfSectionGridProps {
                    cols: Prop::from(2),
                    children: Children::default(),
                })
                .child(
                    HfCard::element(honba_kao_ui_core::primitive::card::HfCardProps {
                        outlined: Prop::from(false),
                        children: Children::default(),
                    })
                    .child(
                        view()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                view()
                                    .flex_row()
                                    .justify_between()
                                    .child(
                                        text("NIFTY 50")
                                            .text_high()
                                            .font_weight(FontWeight::SEMIBOLD),
                                    )
                                    .child(
                                        HfBadge::element(HfBadgeProps {
                                            variant: Prop::from(Some(BadgeVariant::Info)),
                                            outline: Prop::from(false),
                                            children: Children::default(),
                                        })
                                        .child(text("22,847.30")),
                                    ),
                                )
                                .child(
                                    text("+0.42% • 15:28 IST")
                                        .text_muted()
                                        .text_size(px(11.0)),
                                ),
                            ),
                )
                .child(
                    HfCard::element(honba_kao_ui_core::primitive::card::HfCardProps {
                        outlined: Prop::from(false),
                        children: Children::default(),
                    })
                    .child(
                        view()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                view()
                                    .flex_row()
                                    .justify_between()
                                    .child(
                                        text("SENSEX")
                                            .text_high()
                                            .font_weight(FontWeight::SEMIBOLD),
                                    )
                                    .child(
                                        HfBadge::element(HfBadgeProps {
                                            variant: Prop::from(Some(BadgeVariant::Success)),
                                            outline: Prop::from(false),
                                            children: Children::default(),
                                        })
                                        .child(text("75,210.15")),
                                    ),
                                )
                                .child(
                                    text("+0.31% • 15:28 IST")
                                        .text_muted()
                                        .text_size(px(11.0)),
                                ),
                            ),
                )
                .child(
                    HfAssetBadge::element(HfAssetBadgeProps {
                        currency: Prop::from(Currency::Inr),
                        symbol: Prop::from(None),
                        ..Default::default()
                    })
                    .child(text("INR")),
                )
                .child(
                    view()
                        .flex_row()
                        .gap(px(2.0))
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
                            .child(text("Open Cockpit")),
                        )
                        .child(
                            HfButton::element(HfButtonProps {
                                href: Prop::default(),
                                size: Prop::from(Some(ButtonSize::Md)),
                                variant: Prop::from(Some(ButtonVariant::Secondary)),
                                full_width: Prop::from(false),
                                disabled: Prop::from(false),
                                aria_label: Prop::from(None),
                                class: Prop::default(),
                                children: Children::default(),
                            })
                            .child(text("View Positions")),
                        ),
                ),
            )
            .child(
                HfStatusChip::element(HfStatusChipProps {
                    status: Prop::from("Market data live".to_string()),
                    variant: Prop::from(Some(BadgeVariant::Success)),
                }),
            ),
    )
    .child(
        text(
            "Select a workstation tile to begin. Market data is delayed 15 minutes unless a live feed subscription is active.",
        )
        .text_muted()
        .text_size(px(12.0)),
    )
}
