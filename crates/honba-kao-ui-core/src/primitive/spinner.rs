//! `HfSpinner` — monospace / SVG loading indicator for asynchronous data streams.
//! Ported from `secured-finance-app` atom `Spinner`.

use leptos::prelude::*;
use crate::icons::SvgSpinner;

#[component]
pub fn HfSpinner(
    #[prop(into, optional)] size: Option<&'static str>,
    #[prop(into, optional)] class: Option<String>,
) -> impl IntoView {
    let size = size.unwrap_or("w-5 h-5");
    let base = "hf-spinner animate-spin text-text-muted";
    let classes = match class {
        Some(c) => format!("{} {} {}", base, size, c),
        None => format!("{} {}", base, size),
    };
    view! {
        <div class=classes>
            <SvgSpinner />
        </div>
    }
}
