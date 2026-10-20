//! HfPlaceholder — scaffold block for routes whose feature work has not
//! landed yet.

use leptos::prelude::*;

#[component]
pub fn HfPlaceholder(
    route: &'static str,
    #[prop(default = "")]
    note: &'static str,
) -> impl IntoView {
    view! {
        <div class="placeholder">
            <span class="placeholder__badge">"SCAFFOLD"</span>
            <h2 class="placeholder__title">{route}</h2>
            <p class="placeholder__note">{note}</p>
        </div>
    }
}
