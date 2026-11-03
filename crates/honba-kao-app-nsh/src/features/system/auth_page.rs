//! Auth — login form for the institutional workstation.
//! Real layout ported onto NSH primitives (Phase 5, system cluster).

use honba_kao_ui_core::primitive::base::button::HfButtonProps;
use honba_kao_ui_core::primitive::base::input::HfInputProps;
use honba_kao_ui_core::primitive::panel::HfPanelProps;
use honba_kao_ui_core::primitive::{ButtonSize, ButtonVariant, HfButton, HfCard, HfInput, HfPanel};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use honba_kao_ui_core::theme::tokens::HonbaStyle;
use nsh_core::prelude::*;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct AuthPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// Auth — email/password sign-in for the workstation.
#[component]
pub fn AuthPage(_props: AuthPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    HfPage::element(HfPageProps {
        title: Prop::from(Some("Sign In".to_string())),
        subtitle: Prop::from(Some(
            "Institutional access to the trading workstation".to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(
        view()
            .flex_col()
            .items_center()
            .gap(px(4.0))
            .w_full()
            .max_w(px(420.0))
            .child(
                HfCard::element(honba_kao_ui_core::primitive::card::HfCardProps {
                    outlined: Prop::from(false),
                    children: Children::default(),
                })
                .child(
                    view()
                        .flex_col()
                        .gap(px(3.0))
                        .px(px(6.0))
                        .py(px(6.0))
                        .w_full()
                        .child(
                            text("Sign In")
                                .text_high()
                                .text_size(px(20.0))
                                .font_weight(FontWeight::SEMIBOLD),
                        )
                        .child(
                            text("Enter your workstation credentials")
                                .text_muted()
                                .text_size(px(13.0)),
                        )
                        .child(
                            view()
                                .flex_col()
                                .gap(px(1.5))
                                .mt(px(4.0))
                                .child(
                                    text("Work Email")
                                        .text_muted()
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::SEMIBOLD),
                                )
                                .child(HfInput::element(HfInputProps {
                                    value: Prop::from(String::new()),
                                    placeholder: Prop::from(Some("name@firm.in".into())),
                                    disabled: Prop::from(false),
                                }))
                                .child(
                                    text("Password")
                                        .text_muted()
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::SEMIBOLD),
                                )
                                .child(HfInput::element(HfInputProps {
                                    value: Prop::from(String::new()),
                                    placeholder: Prop::from(Some("••••••••".into())),
                                    disabled: Prop::from(false),
                                })),
                        )
                        .child(
                            HfButton::element(HfButtonProps {
                                href: Prop::default(),
                                size: Prop::from(Some(ButtonSize::Md)),
                                variant: Prop::from(Some(ButtonVariant::Primary)),
                                full_width: Prop::from(true),
                                disabled: Prop::from(false),
                                aria_label: Prop::from(None),
                                class: Prop::default(),
                                children: Children::default(),
                            })
                            .child(text("Sign In")),
                        )
                        .child(
                            HfButton::element(HfButtonProps {
                                href: Prop::default(),
                                size: Prop::from(Some(ButtonSize::Sm)),
                                variant: Prop::from(Some(ButtonVariant::Ghost)),
                                full_width: Prop::from(true),
                                disabled: Prop::from(false),
                                aria_label: Prop::from(None),
                                class: Prop::default(),
                                children: Children::default(),
                            })
                            .child(text("Forgot password?")),
                        ),
                ),
            )
            .child(
                HfPanel::element(HfPanelProps {
                    header: Prop::from(None),
                    children: Children::default(),
                })
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(2.0))
                        .child(
                            text("New to the workstation?")
                                .text_muted()
                                .text_size(px(12.0)),
                        )
                        .child(
                            HfButton::element(HfButtonProps {
                                href: Prop::default(),
                                size: Prop::from(Some(ButtonSize::Sm)),
                                variant: Prop::from(Some(ButtonVariant::Secondary)),
                                full_width: Prop::from(false),
                                disabled: Prop::from(false),
                                aria_label: Prop::from(None),
                                class: Prop::default(),
                                children: Children::default(),
                            })
                            .child(text("Create organisation")),
                        ),
                ),
            ),
    )
}
