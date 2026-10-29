//! `HfCollateralSnapshot` — summary card of account health: total collateral
//! value, borrow capacity, and utilization. Ported from
//! `secured-finance-app` molecule `CollateralSnapshot`.
//!
//! `total_collateral_value`/`borrow_capacity` are money (Zero Floats
//! Policy): they cross as caller-pre-formatted `Prop<String>`. Unlike every
//! other percent prop in this port, `margin_utilization_pct` *also* crosses
//! as `Prop<String>` rather than `f64`: it is multiplied against
//! `total_collateral_value` to compute `headroom`, itself a money value, so
//! that multiplication must stay in `Decimal` throughout — doing it in
//! `f64` would violate the zero-floats policy.

use std::str::FromStr;

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;
use rust_decimal::Decimal;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfCollateralSnapshot`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfCollateralSnapshotProps {
    /// Pre-formatted by the caller; parsed to `Decimal`.
    pub total_collateral_value: Prop<String>,
    /// Pre-formatted by the caller; parsed to `Decimal`.
    pub borrow_capacity: Prop<String>,
    /// Pre-formatted by the caller; parsed to `Decimal`. Feeds the
    /// `headroom` money calculation below, so it is a `Decimal`, not an
    /// `f64`.
    pub margin_utilization_pct: Prop<String>,
}

/// One `label` / `value` metric tile of the 2-column grid.
fn metric(label: &'static str, value: String) -> impl IntoElement {
    view()
        .flex_col()
        .gap(px(2.0))
        .p(px(6.0))
        .rounded(px(4.0))
        .surface_muted()
        .border_soft()
        .child(text(label).text_muted().text_size(px(10.0)))
        .child(text(value).text_high().font_family("monospace"))
}

/// Summary card of account collateral health: total collateral value,
/// borrow capacity, margin utilization (tri-tier tinted), and the resulting
/// headroom.
#[component]
pub fn HfCollateralSnapshot(
    props: HfCollateralSnapshotProps,
    cx: Scope<NoEvent>,
) -> impl IntoElement {
    let _ = cx;
    let total_collateral_value =
        Decimal::from_str(&props.total_collateral_value.get_untracked()).unwrap_or_default();
    let borrow_capacity =
        Decimal::from_str(&props.borrow_capacity.get_untracked()).unwrap_or_default();
    let margin_utilization_pct =
        Decimal::from_str(&props.margin_utilization_pct.get_untracked()).unwrap_or_default();

    let headroom = (borrow_capacity
        - total_collateral_value * margin_utilization_pct / Decimal::from(100))
    .max(Decimal::ZERO);

    let zone_value = if margin_utilization_pct < Decimal::new(60, 0) {
        text(format!("{margin_utilization_pct:.1}%")).text_positive()
    } else if margin_utilization_pct < Decimal::new(80, 0) {
        text(format!("{margin_utilization_pct:.1}%")).text_warning()
    } else {
        text(format!("{margin_utilization_pct:.1}%")).text_negative()
    };

    view()
        .flex_col()
        .gap(px(8.0))
        .rounded(px(8.0))
        .border_soft()
        .surface_base()
        .p(px(12.0))
        .text_size(px(12.0))
        .app_class(Some(SharedString::from("hf-collateral-snapshot")))
        .child(
            text("Collateral Snapshot")
                .text_muted()
                .font_weight(FontWeight::SEMIBOLD)
                .uppercase()
                .letter_spacing(px(0.5))
                .text_size(px(10.0)),
        )
        .child(
            view()
                .grid()
                .grid_cols(2)
                .gap(px(8.0))
                .child(metric(
                    "Total Collateral",
                    format!("₹{total_collateral_value:.2}"),
                ))
                .child(metric("Borrow Capacity", format!("₹{borrow_capacity:.2}"))),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(6.0))
                        .child(text("Margin Utilization").text_muted().text_size(px(10.0)))
                        .child(zone_value.font_family("monospace")),
                )
                .child(
                    text(format!("Headroom: ₹{headroom:.2}"))
                        .text_muted()
                        .text_size(px(10.0)),
                ),
        )
}
