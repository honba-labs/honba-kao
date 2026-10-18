//! KPI metric tile (label, value, optional trend delta).
//!
//! The value is a `children` block so it can be a static string or a reactive
//! closure over live account data.

use leptos::prelude::*;

#[component]
pub fn Kpi(
    label: &'static str,
    #[prop(optional, into)] delta: String,
    #[prop(default = true)] up: bool,
    children: Children,
) -> impl IntoView {
    let delta_class = if up {
        "kpi__delta up"
    } else {
        "kpi__delta down"
    };
    let has_delta = !delta.is_empty();

    view! {
        <div class="kpi">
            <span class="kpi__label">{label}</span>
            <span class="kpi__value tnum">{children()}</span>
            {has_delta.then(move || view! { <span class=delta_class>{delta}</span> })}
        </div>
    }
}
