//! Collateral organism — NSH port per nsh-port.md §4 Phase 3.
//!
//! A view over collateral assets and haircuts. Built from `nsh_core`
//! primitives; honba styling via `.class()`/`.style()`.

use nsh_core::prelude::*;

/// One collateral row. Amounts as strings (Decimal formatted by caller).
#[derive(Debug, Clone, PartialEq, Props, JsObject)]
pub struct CollateralRow {
    pub asset: String,
    pub value: String,
    pub haircut: String,
    pub net_value: String,
}

/// Props for [`HfCollateralOrganism`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfCollateralOrganismProps {
    pub rows: Prop<Vec<CollateralRow>>,
    pub total_collateral: Prop<String>,
    pub total_haircut: Prop<String>,
}

/// Collateral summary and asset breakdown.
#[component]
pub fn HfCollateralOrganism(
    props: HfCollateralOrganismProps,
    _cx: Scope<NoEvent>,
) -> impl IntoElement {
    let rows = props.rows;

    view()
        .flex_col()
        .gap(px(8.0))
        .px_4()
        .py_2()
        .bg(rgb(0x0f172a))
        .text_color(rgb(0xe2e8f0))
        .child(
            view()
                .flex_row()
                .justify_between()
                .child(text("Total Collateral"))
                .child(text(move || props.total_collateral.get())),
        )
        .child(
            view()
                .flex_row()
                .justify_between()
                .child(text("Total Haircut"))
                .child(text(move || props.total_haircut.get())),
        )
        .child(
            view().flex_col().gap(px(4.0)).child(
                each(
                    rows,
                    move |r: &CollateralRow| r.asset.clone(),
                    move |row_signal, _| {
                        let row = row_signal.get();
                        view()
                            .flex_row()
                            .justify_between()
                            .px_2()
                            .py_1()
                            .child(text(row.asset))
                            .child(text(row.value))
                            .child(text(row.haircut))
                            .child(text(row.net_value))
                    },
                )
                .into_element(),
            ),
        )
}
