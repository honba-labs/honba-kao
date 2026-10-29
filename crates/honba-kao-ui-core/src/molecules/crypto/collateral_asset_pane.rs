//! `HfCollateralAssetPane` — left panel listing deposited collateral asset
//! positions. Ported from `secured-finance-app` molecule
//! `CollateralTabLeftPane`.
//!
//! `balance`/`collateral_value` are money (Zero Floats Policy): they cross
//! as caller-pre-formatted `String` fields of [`CollateralAsset`] and are
//! parsed to `Decimal` here only for display formatting and the running
//! total — never fed into `f64` math. `haircut_pct` is only compared and
//! displayed, never fed into money math, so it stays a plain `f64`, same as
//! `collateral_tile.rs`'s identical field.

use std::str::FromStr;

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;
use rust_decimal::Decimal;

use crate::primitive::currency::HfAssetBadgeProps;
use crate::primitive::{Currency, HfAssetBadge};
use crate::theme::tokens::HonbaStyle;

/// A single deposited collateral asset row of [`HfCollateralAssetPane`].
#[derive(Clone, Debug, Default, PartialEq, JsObject)]
pub struct CollateralAsset {
    pub symbol: String,
    /// Pre-formatted by the caller; parsed to `Decimal` for display.
    pub balance: String,
    pub haircut_pct: f64,
    /// Pre-formatted by the caller; parsed to `Decimal` for display and the
    /// running total.
    pub collateral_value: String,
    pub eligible: bool,
}

/// Props for [`HfCollateralAssetPane`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfCollateralAssetPaneProps {
    pub assets: Prop<Vec<CollateralAsset>>,
}

/// Left panel of the collateral manager listing deposited asset positions,
/// each rendered as a flex row (no table primitive available — same
/// "build flex rows by hand" treatment as `option_chain_row`, see
/// `.agents/HANDOFF.md` §11), with a running total above the list.
///
/// nsh has no scroll-container primitive (no `max-height`/`overflow-y-auto`
/// — confirmed, only `overflow_hidden()` exists), so the original's
/// `max-h-[300px] overflow-y-auto` is dropped: a long asset list simply
/// grows the panel, a documented gap like `go_bar`'s scroll-into-view.
#[component]
pub fn HfCollateralAssetPane(
    props: HfCollateralAssetPaneProps,
    cx: Scope<NoEvent>,
) -> impl IntoElement {
    let _ = cx;
    let assets = props.assets;
    let total_text = move || {
        let total: Decimal = assets
            .get()
            .iter()
            .map(|a| Decimal::from_str(&a.collateral_value).unwrap_or_default())
            .sum();
        format!("Total: ₹{total:.2}")
    };

    view()
        .flex_col()
        .gap(px(8.0))
        .p(px(12.0))
        .rounded(px(4.0))
        .border_soft()
        .surface_base()
        .text_size(px(12.0))
        .app_class(Some(SharedString::from("hf-collateral-asset-pane")))
        .child(
            view()
                .flex_row()
                .justify_between()
                .items_center()
                .app_class(Some(SharedString::from("hf-pane-header")))
                .child(
                    text("Deposited Assets")
                        .text_high()
                        .font_weight(FontWeight::SEMIBOLD)
                        .uppercase()
                        .letter_spacing(px(0.5))
                        .text_size(px(10.0)),
                )
                .child(text(total_text).text_high().font_family("monospace")),
        )
        .child(
            view()
                .flex_col()
                .gap(px(4.0))
                .app_class(Some(SharedString::from("hf-asset-list")))
                .child(each(
                    move || assets.get().into_iter().enumerate().collect::<Vec<_>>(),
                    |(index, _): &(usize, CollateralAsset)| *index,
                    move |entry: ReadSignal<(usize, CollateralAsset)>, _row: ReadSignal<usize>| {
                        let asset = entry.get_untracked().1;
                        let balance = Decimal::from_str(&asset.balance).unwrap_or_default();
                        let collateral_value =
                            Decimal::from_str(&asset.collateral_value).unwrap_or_default();
                        let haircut_row = if asset.eligible {
                            text(format!("haircut {:.0}%", asset.haircut_pct)).text_high()
                        } else {
                            text(format!("haircut {:.0}%", asset.haircut_pct)).text_muted()
                        };

                        view()
                            .flex_row()
                            .justify_between()
                            .items_center()
                            .px(px(8.0))
                            .py(px(6.0))
                            .rounded(px(4.0))
                            .border_soft()
                            .surface_muted()
                            .app_class(Some(SharedString::from("hf-asset-row")))
                            .child(
                                view()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(4.0))
                                    .app_class(Some(SharedString::from("hf-asset-info")))
                                    .child(HfAssetBadge::element(HfAssetBadgeProps {
                                        currency: Prop::from(Currency::default()),
                                        symbol: Prop::from(Some(asset.symbol.clone())),
                                    })),
                            )
                            .child(
                                view()
                                    .flex_col()
                                    .items_end()
                                    .gap(px(2.0))
                                    .font_family("monospace")
                                    .text_high()
                                    .app_class(Some(SharedString::from("hf-asset-values")))
                                    .child(text(format!("{balance:.2}")))
                                    .child(haircut_row)
                                    .child(text(format!("₹{collateral_value:.2}")).text_positive()),
                            )
                    },
                )),
        )
}
