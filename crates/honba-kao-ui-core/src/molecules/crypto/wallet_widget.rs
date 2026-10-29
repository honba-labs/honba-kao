//! `HfWalletSummaryWidget` — full account balance widget (cash, margin
//! consumed, buying power, collateral ratio). Ported from
//! `secured-finance-app` `MyWalletWidget` / `MyWalletCard`.
//!
//! `cash`/`buying_power`/`margin_consumed` are money (Zero Floats Policy):
//! they cross as caller-pre-formatted `Prop<String>` and are parsed to
//! `Decimal` here only for display formatting. `collateral_ratio` is only
//! ever displayed, so it stays a plain `f64`.

use std::str::FromStr;

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;
use rust_decimal::Decimal;

use crate::primitive::HfStatusLed;
use crate::primitive::prompt::{HfStatusLedProps, StatusTone};
use crate::theme::tokens::HonbaStyle;

/// Props for [`HfWalletSummaryWidget`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfWalletSummaryWidgetProps {
    /// Pre-formatted by the caller; parsed to `Decimal` for display.
    pub cash: Prop<String>,
    /// Pre-formatted by the caller; parsed to `Decimal` for display.
    pub buying_power: Prop<String>,
    /// Pre-formatted by the caller; parsed to `Decimal` for display.
    pub margin_consumed: Prop<String>,
    pub collateral_ratio: Prop<f64>,
}

/// One `label` / `value` metric tile of the 2x2 grid.
fn metric(label: &'static str, value: String) -> impl IntoElement {
    view()
        .flex_col()
        .gap(px(2.0))
        .p(px(8.0))
        .rounded(px(4.0))
        .surface_muted()
        .border_soft()
        .child(text(label).text_muted().text_size(px(10.0)))
        .child(text(value).text_high().font_family("monospace"))
}

/// Full account balance widget: a "Live" status header over a 2x2 grid of
/// cash, buying power, margin consumed, and collateral ratio.
#[component]
pub fn HfWalletSummaryWidget(
    props: HfWalletSummaryWidgetProps,
    cx: Scope<NoEvent>,
) -> impl IntoElement {
    let _ = cx;
    let cash = Decimal::from_str(&props.cash.get_untracked()).unwrap_or_default();
    let buying_power = Decimal::from_str(&props.buying_power.get_untracked()).unwrap_or_default();
    let margin_consumed =
        Decimal::from_str(&props.margin_consumed.get_untracked()).unwrap_or_default();
    let collateral_ratio = props.collateral_ratio.get_untracked();

    let status = view()
        .flex_row()
        .items_center()
        .gap(px(4.0))
        .child(HfStatusLed::element(HfStatusLedProps {
            tone: Prop::from(Some(StatusTone::Ok)),
        }))
        .child(text("Live").text_muted().text_size(px(10.0)));

    view()
        .flex_col()
        .gap(px(8.0))
        .rounded(px(8.0))
        .border_soft()
        .surface_base()
        .p(px(12.0))
        .app_class(Some(SharedString::from("hf-wallet-widget")))
        .child(
            view()
                .flex_row()
                .justify_between()
                .items_center()
                .child(
                    text("Account")
                        .text_high()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_size(px(11.0))
                        .uppercase()
                        .letter_spacing(px(0.5)),
                )
                .child(status),
        )
        .child(
            view()
                .grid()
                .grid_cols(2)
                .gap(px(8.0))
                .child(metric("Cash", format!("{cash:.2}")))
                .child(metric("Buying Power", format!("{buying_power:.2}")))
                .child(metric("Margin Consumed", format!("{margin_consumed:.2}")))
                .child(metric("Collateral Ratio", format!("{collateral_ratio:.2}"))),
        )
}
