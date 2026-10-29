//! `HfCollateralHaircutTile` — asset-level collateral contribution with haircut
//! ratio. Ported from `secured-finance-app` atoms `CollateralInformation` /
//! `CollateralInformationTable`.
//!
//! `balance`/`collateral_value` are money (Zero Floats Policy): they cross as
//! caller-pre-formatted `Prop<String>` and are parsed to `Decimal` here only
//! for display formatting. `haircut_pct` is only compared and displayed,
//! never fed into money math, so it stays a plain `f64`.

use std::str::FromStr;

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;
use rust_decimal::Decimal;

use crate::primitive::currency::HfAssetBadgeProps;
use crate::primitive::{Currency, HfAssetBadge};
use crate::theme::tokens::HonbaStyle;

/// Props for [`HfCollateralHaircutTile`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfCollateralHaircutTileProps {
    pub asset: Prop<String>,
    /// Pre-formatted by the caller; parsed to `Decimal` for display.
    pub balance: Prop<String>,
    pub haircut_pct: Prop<f64>,
    /// Pre-formatted by the caller; parsed to `Decimal` for display.
    pub collateral_value: Prop<String>,
}

/// Asset-level collateral contribution: balance, haircut ratio, and the
/// resulting collateral value. The haircut row dims when the asset is no
/// longer collateral-eligible (`haircut_pct >= 100`).
#[component]
pub fn HfCollateralHaircutTile(
    props: HfCollateralHaircutTileProps,
    cx: Scope<NoEvent>,
) -> impl IntoElement {
    let _ = cx;
    let asset = props.asset.get_untracked();
    let balance = Decimal::from_str(&props.balance.get_untracked()).unwrap_or_default();
    let haircut_pct = props.haircut_pct.get_untracked();
    let collateral_value =
        Decimal::from_str(&props.collateral_value.get_untracked()).unwrap_or_default();
    let eligible = haircut_pct < 100.0;

    let haircut_row = if eligible {
        text(format!("Haircut: {haircut_pct:.0}%")).text_high()
    } else {
        text(format!("Haircut: {haircut_pct:.0}%")).text_muted()
    };

    view()
        .flex_row()
        .justify_between()
        .items_start()
        .px(px(12.0))
        .py(px(10.0))
        .rounded(px(4.0))
        .border_soft()
        .surface_base()
        .text_size(px(12.0))
        .app_class(Some(SharedString::from("hf-collateral-tile")))
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(4.0))
                .child(HfAssetBadge::element(HfAssetBadgeProps {
                    currency: Prop::from(Currency::default()),
                    symbol: Prop::from(Some(asset)),
                })),
        )
        .child(
            view()
                .flex_col()
                .items_end()
                .gap(px(2.0))
                .font_family("monospace")
                .text_high()
                .child(text(format!("Balance: {balance:.2}")))
                .child(haircut_row)
                .child(text(format!("Collateral Value: {collateral_value:.2}")).text_positive()),
        )
}
