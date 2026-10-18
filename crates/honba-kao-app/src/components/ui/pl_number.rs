//! Profit & Loss formatted number modeled after bfx-hf-ui-core PLNumber.

use leptos::prelude::*;

#[component]
pub fn HfPLNumber(
    value: f64,
    #[prop(default = true)] is_currency: bool,
    #[prop(default = "")] suffix: &'static str,
) -> impl IntoView {
    let is_positive = value > 0.0001;
    let is_negative = value < -0.0001;

    let (color_class, arrow) = if is_positive {
        ("positive", "▲ ")
    } else if is_negative {
        ("negative", "▼ ")
    } else {
        ("neutral", "")
    };

    let formatted_val = if is_currency {
        let abs_val = value.abs();
        if value < 0.0 {
            format!("-₹{:.2}", abs_val)
        } else {
            format!("₹{:.2}", abs_val)
        }
    } else {
        format!("{:.2}{}", value, suffix)
    };

    view! {
        <span class=format!("hfui-plnumber {}", color_class)>
            <span>{arrow}</span>
            <span>{formatted_val}</span>
        </span>
    }
}
