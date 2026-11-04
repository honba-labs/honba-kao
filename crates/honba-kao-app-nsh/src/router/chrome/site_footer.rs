//! `SiteFooter` — persistent status bar: OMS connection, latency, feed
//! status, and version info.

use crate::shared::contexts::HonbaClientContext;
use honba_kao_ui_core::theme::tokens::HonbaStyle;
use nsh_core::prelude::*;

/// Props for [`SiteFooter`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct SiteFooterProps {}

/// The persistent bottom status bar.
#[component]
pub fn SiteFooter(_props: SiteFooterProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let client = use_context::<HonbaClientContext>().unwrap_or_else(fallback_client_context);
    let oms_connected = client.oms_connected;
    let feed_connected = client.feed_connected;
    let latency_ms = client.latency_ms;

    let status_dot = view().w(px(6.0)).h(px(6.0)).rounded(px(999.0)).bg(move || {
        if oms_connected.get() {
            AdaptiveColor::rgb(0x34d399, 0x34d399)
        } else {
            AdaptiveColor::rgb(0xf43f5e, 0xf43f5e)
        }
    });

    let oms_status = text(move || {
        if oms_connected.get() {
            "OMS: CONNECTED".to_string()
        } else {
            "OMS: DISCONNECTED".to_string()
        }
    })
    .font_family("monospace")
    .text_size(px(11.0))
    .text_color(move || {
        if oms_connected.get() {
            AdaptiveColor::rgb(0x10b981, 0x34d399)
        } else {
            AdaptiveColor::rgb(0xef4444, 0xf87171)
        }
    });

    let latency = text(move || match latency_ms.get() {
        Some(ms) => format!("LATENCY: {ms}ms"),
        None => "LATENCY: --".to_string(),
    })
    .font_family("monospace")
    .text_size(px(11.0))
    .text_muted();

    let feed_ticks = text(move || {
        if feed_connected.get() {
            "FEED: LIVE".to_string()
        } else {
            "FEED: IDLE".to_string()
        }
    })
    .font_family("monospace")
    .text_size(px(11.0))
    .text_muted();

    let divider = || text("|").text_size(px(11.0)).text_muted();

    view()
        .flex_row()
        .items_center()
        .justify_between()
        .px(px(16.0))
        .py(px(6.0))
        .surface_muted()
        .border_t(px(1.0))
        .border_soft()
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(12.0))
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(6.0))
                        .child(status_dot)
                        .child(oms_status),
                )
                .child(divider())
                .child(latency)
                .child(divider())
                .child(feed_ticks),
        )
        .child(
            view().flex_row().items_center().gap(px(12.0)).child(
                text("HONBA-KAO v3.0 Institutional")
                    .font_family("monospace")
                    .text_size(px(11.0))
                    .text_muted(),
            ),
        )
}

/// Fallback client context when `HonbaClientContext` has not been provided
/// (e.g. a standalone preview of the footer): the context's own offline
/// mirror.
fn fallback_client_context() -> HonbaClientContext {
    HonbaClientContext::offline()
}
