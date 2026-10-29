//! Top-level `App` shell component — NSH port per nsh-port.md §4 Phase 4.
//!
//! This is the root `#[component]` that composes organisms into the full
//! trading terminal layout. Templates are inlined here since `Children`
//! props are framework-managed (JSX) rather than constructible in pure Rust.
//! Routing is intentionally minimal: the routing story is an open question
//! per nsh-port.md §6 risk #5.

use crate::organisms::*;
use crate::templates::*;
use nsh_core::prelude::*;

/// Props for [`App`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct AppProps {
    #[prop(default)]
    pub user_name: Prop<Option<String>>,
}

/// The root trading terminal shell.
#[component]
pub fn App(props: AppProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    view()
        .flex_col()
        .gap(px(2.0))
        .w_full()
        .h_full()
        .bg(rgb(0x0f172a))
        .text_color(rgb(0xe2e8f0))
        .child(HfMarketTopBar::element(HfMarketTopBarProps {
            labels: Prop::new(vec![
                "NIFTY 50".to_string(),
                "SENSEX".to_string(),
                "BANKNIFTY".to_string(),
                "NASDAQ".to_string(),
            ]),
            values: Prop::new(vec![
                "22,450.50".to_string(),
                "73,200.15".to_string(),
                "48,550.30".to_string(),
                "18,450.20".to_string(),
            ]),
            desk: props.user_name,
            currency: Prop::new(Some("INR".to_string())),
        }))
        .child(
            view()
                .flex_col()
                .gap(px(2.0))
                .w_full()
                .flex_1()
                .child(text("Toolbar"))
                .child(HfThreeColumnTradingLayout::element(
                    HfThreeColumnTradingLayoutProps {
                        market_tape: Prop::from(Some(
                            "NIFTY 50  22,450.50  SENSEX  73,200.15".to_string(),
                        )),
                    },
                )),
        )
}
