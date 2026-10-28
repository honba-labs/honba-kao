//! HfPanel and HfPanelHeader — titled surface modeled after bfx-hf-ui-core
//! `.hfui-panel`.

use leptos::prelude::*;

#[component]
pub fn HfPanel(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <section class=format!("hfui-panel {class}").trim().to_string()>
            {children()}
        </section>
    }
}

/// Panel header: label, optional counter chip and trailing actions.
#[component]
pub fn HfPanelHeader(
    title: &'static str,
    #[prop(optional)] counter: Option<usize>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <header class="hfui-panel__header">
            <div class="hfui-panel__header-left">
                <span class="hfui-panel__label">{title}</span>
                {counter.map(|c| view! {
                    <span class="hfui-panel__counter">{c}</span>
                })}
            </div>
            <div class="hfui-panel__header-actions">
                {children.map(|c| c())}
            </div>
        </header>
    }
}
