//! `HfGreeksGroup` — monospace display of Delta, Gamma, Theta, Vega, Rho.

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Option Greeks. Display values, not tradeable quantities, so `f64`.
#[derive(Clone, Copy, Debug, Default, PartialEq, JsObject)]
pub struct Greeks {
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub rho: f64,
}

/// Props for [`HfGreeksGroup`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfGreeksGroupProps {
    pub greeks: Prop<Greeks>,
}

/// One `label  value` pair of the grid.
fn greek_cell(label: &'static str, value: impl Fn() -> String + 'static) -> impl IntoElement {
    view()
        .flex_row()
        .justify_between()
        .items_center()
        .child(text(label).text_muted())
        .child(text(value).text_high())
}

/// Delta, Gamma, Theta, Vega and Rho in a two-column monospace grid. Theta is
/// tinted by sign: positive emerald, negative rose.
#[component]
pub fn HfGreeksGroup(props: HfGreeksGroupProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let greeks = props.greeks;
    let theta_class = move || {
        if greeks.get().theta >= 0.0 {
            Some(SharedString::from("text-emerald-400"))
        } else {
            Some(SharedString::from("text-rose-400"))
        }
    };

    view()
        .grid()
        .grid_cols(2)
        .gap(px(8.0))
        .text_size(px(10.0))
        .font_family("monospace")
        .app_class(Some(SharedString::from("hf-greeks-grid")))
        .child(greek_cell("Δ", move || {
            format!("{:.3}", greeks.get().delta)
        }))
        .child(greek_cell("Γ", move || {
            format!("{:.3}", greeks.get().gamma)
        }))
        .child(
            view()
                .flex_row()
                .justify_between()
                .items_center()
                .child(text("Θ").text_muted())
                .child(text(move || format!("{:.3}", greeks.get().theta)).app_class(theta_class)),
        )
        .child(greek_cell("ν", move || {
            format!("{:.3}", greeks.get().vega)
        }))
        .child(greek_cell("ρ", move || format!("{:.3}", greeks.get().rho)))
}
