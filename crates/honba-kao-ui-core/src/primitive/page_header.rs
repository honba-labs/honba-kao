//! HfPageHeader — page title, optional subtitle and trailing actions.

use leptos::prelude::*;

#[component]
pub fn HfPageHeader(
    title: &'static str,
    #[prop(default = "")]
    subtitle: &'static str,
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="page-header">
            <div>
                <h1 class="page-header__title">{title}</h1>
                {(!subtitle.is_empty()).then(|| view! {
                    <p class="page-header__sub">{subtitle}</p>
                })}
            </div>
            <div class="page-header__actions">
                {children.map(|c| c())}
            </div>
        </div>
    }
}
