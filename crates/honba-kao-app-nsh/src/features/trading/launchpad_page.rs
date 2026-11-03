//! Launchpad — one-press route tiles across the workstation (Stage 3
//! Phase A.3 of the gallery-rs porting guide, replacing the stub page).
//!
//! A grid of outline buttons; each press routes through the responsive
//! shell's entry opener ([`app_shell::open_path`]), so the tile works in
//! both the desktop sidebar layout and the phone nav stack.

use crate::router::chrome::app_shell::open_path;
use crate::router::paths;
use honba_kao_ui_core::primitive::base::button::{
    ButtonSize, ButtonVariant, HfButton, HfButtonEvent, HfButtonProps,
};
use honba_kao_ui_core::templates::page::{HfPage, HfPageProps};
use honba_kao_ui_core::theme::tokens::HonbaStyle;
use nsh_core::prelude::*;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct LaunchpadPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// One launch tile: an outline button whose press opens a shell entry.
fn tile(label: &str, path: &'static str) -> impl IntoElement {
    HfButton::element(HfButtonProps {
        href: Prop::default(),
        size: Prop::from(Some(ButtonSize::Md)),
        variant: Prop::from(Some(ButtonVariant::Outline)),
        full_width: Prop::from(true),
        disabled: Prop::from(false),
        aria_label: Prop::from(None),
        class: Prop::default(),
        children: Children::default(),
    })
    .child(text(label.to_string()))
    .on_event(move |event: &HfButtonEvent| {
        if *event == HfButtonEvent::Press {
            open_path(path);
        }
    })
}

/// A titled group of tiles (row-wrapped at fixed tile width).
fn group(title: &'static str, tiles: Vec<impl IntoElement>) -> impl IntoElement {
    view()
        .flex_col()
        .gap_2()
        .w_full()
        .child(
            text(title.to_string())
                .text_size(px(13.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_muted(),
        )
        .child(
            view()
                .flex_row()
                .flex_wrap()
                .gap(px(10.0))
                .children(tiles.into_iter().map(|t| view().w(px(210.0)).child(t))),
        )
}

/// Launchpad — quick routes across the trading workstation and beyond.
#[component]
pub fn LaunchpadPage(_props: LaunchpadPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    HfPage::element(HfPageProps {
        title: Prop::from(Some("Launchpad".to_string())),
        subtitle: Prop::from(Some(
            "One-press routes across the trading workstation and the app's other clusters"
                .to_string(),
        )),
        max_width_class: Prop::default(),
        children: Children::default(),
    })
    .child(group(
        "Trading",
        vec![
            tile("Trading Cockpit", paths::ROUTE_TRADING_COCKPIT),
            tile("Multi-Chart", paths::ROUTE_TRADING_MULTI_CHART),
            tile("DOM", paths::ROUTE_TRADING_DOM),
            tile("Tape", paths::ROUTE_TRADING_TAPE),
            tile("MTA", paths::ROUTE_TRADING_MTA),
            tile("Compare", paths::ROUTE_TRADING_COMPARE),
        ],
    ))
    .child(group(
        "Across the workstation",
        vec![
            tile("Option Chain", paths::ROUTE_DERIVATIVES_OPTION_CHAIN),
            tile("Strategy Builder", paths::ROUTE_DERIVATIVES_STRATEGY),
            tile("Portfolio", paths::ROUTE_PORTFOLIO),
            tile("Quant Simulator", paths::ROUTE_QUANT_SIMULATOR),
            tile("Backtesting", paths::ROUTE_QUANT_BACKTESTING),
            tile("Risk & Ops", paths::ROUTE_RISK),
        ],
    ))
}
