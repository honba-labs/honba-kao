//! `HfAlertBanner` and `HfAlertBannerStack` — contextual notification banners
//! positioned above workspace content (Secured-Finance multi-tier alert hierarchy).

use nsh_core::prelude::*;

use crate::primitive::base::button::{HfButtonEvent, HfCloseButton, HfCloseButtonProps};
use crate::theme::tokens::HonbaStyle;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PropEnum)]
pub enum AlertSeverity {
    #[default]
    Info,
    Warning,
    Error,
    Success,
}

impl AlertSeverity {
    /// Tailwind background / border / text classes of the severity.
    fn class(self) -> &'static str {
        match self {
            AlertSeverity::Info => "bg-blue-600/20 border-blue-500/30 text-blue-400",
            AlertSeverity::Warning => "bg-amber-600/20 border-amber-500/30 text-amber-400",
            AlertSeverity::Error => "bg-rose-600/20 border-rose-500/30 text-rose-400",
            AlertSeverity::Success => "bg-emerald-600/20 border-emerald-500/30 text-emerald-400",
        }
    }

    /// The severity's glyph (plain Unicode text, no icon font).
    fn icon(self) -> &'static str {
        match self {
            AlertSeverity::Info => "ℹ",
            AlertSeverity::Warning => "⚠",
            AlertSeverity::Error => "✖",
            AlertSeverity::Success => "✔",
        }
    }
}

/// One entry of [`HfAlertBannerStack`].
#[derive(Clone, Debug, Default, PartialEq, JsObject)]
pub struct AlertStackItem {
    pub severity: AlertSeverity,
    pub message: String,
}

/// The events of [`HfAlertBanner`].
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfAlertBannerEvent {
    /// The dismiss button was pressed.
    Dismiss,
}

/// Props for [`HfAlertBanner`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfAlertBannerProps {
    /// Defaults to `Info` when `None`.
    #[prop(default)]
    pub severity: Prop<Option<AlertSeverity>>,
    pub message: Prop<String>,
    #[prop(default = false)]
    pub dismissible: Prop<bool>,
}

/// A single notification banner, optionally with a dismiss button.
#[component]
pub fn HfAlertBanner(props: HfAlertBannerProps, cx: Scope<HfAlertBannerEvent>) -> impl IntoElement {
    let severity = props.severity;
    let message = props.message;
    let dismissible = props.dismissible.get_untracked();

    let root = view()
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .px(px(12.0))
        .py(px(8.0))
        .rounded(px(4.0))
        .app_class(move || {
            Some(SharedString::from(format!(
                "hf-alert-banner border {}",
                severity.get().unwrap_or_default().class()
            )))
        })
        .child(
            text(move || severity.get().unwrap_or_default().icon().to_string()).text_size(px(11.0)),
        )
        .child(text(move || message.get()).text_high().text_size(px(12.0)));

    if dismissible {
        let close = HfCloseButton::element(HfCloseButtonProps {
            aria_label: Prop::from(Some("Dismiss".to_string())),
            ..Default::default()
        })
        .on_event(move |event: &HfButtonEvent| {
            if matches!(event, HfButtonEvent::Press) {
                cx.emit(HfAlertBannerEvent::Dismiss);
            }
        });
        root.child(
            view()
                .app_class(Some(SharedString::from("ml-auto")))
                .child(close),
        )
    } else {
        root
    }
}

/// The events of [`HfAlertBannerStack`].
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfAlertBannerStackEvent {
    /// The banner at this index was dismissed.
    Dismiss(usize),
}

/// Props for [`HfAlertBannerStack`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfAlertBannerStackProps {
    pub banners: Prop<Vec<AlertStackItem>>,
}

/// Structural notification header stack positioned above workspace content.
#[component]
pub fn HfAlertBannerStack(
    props: HfAlertBannerStackProps,
    cx: Scope<HfAlertBannerStackEvent>,
) -> impl IntoElement {
    let banners = props.banners;

    view()
        .flex_col()
        .gap(px(4.0))
        .mb(px(12.0))
        .app_class(Some(SharedString::from("hf-alert-banner-stack")))
        .child(each(
            move || banners.get().into_iter().enumerate().collect::<Vec<_>>(),
            |(index, _): &(usize, AlertStackItem)| *index,
            move |entry: ReadSignal<(usize, AlertStackItem)>, _row: ReadSignal<usize>| {
                let cx = cx.clone();
                HfAlertBanner::element(HfAlertBannerProps {
                    severity: Prop::bind(move || Some(entry.get().1.severity)),
                    message: Prop::bind(move || entry.get().1.message),
                    dismissible: Prop::from(true),
                    ..Default::default()
                })
                .on_event(move |event: &HfAlertBannerEvent| match event {
                    HfAlertBannerEvent::Dismiss => {
                        cx.emit(HfAlertBannerStackEvent::Dismiss(entry.get_untracked().0));
                    }
                })
            },
        ))
}
