//! Panel primitives modeled after bfx-hf-ui-core .hfui-panel.

use leptos::prelude::*;

#[component]
pub fn HfPanel(#[prop(default = "")] class: &'static str, children: Children) -> impl IntoView {
    view! {
        <section class=format!("hfui-panel {}", class)>
            {children()}
        </section>
    }
}

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
                {move || counter.map(|c| view! {
                    <span class="hfui-panel__counter">{c}</span>
                })}
            </div>
            <div class="hfui-panel__header-actions">
                {children.map(|c| c())}
            </div>
        </header>
    }
}

#[component]
pub fn HfIconButton(
    icon: &'static str,
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(default = "")] title: &'static str,
) -> impl IntoView {
    let handle_click = move |_| {
        if let Some(cb) = on_click {
            cb.run(());
        }
    };

    view! {
        <button
            class="hfui-panel__icon-button"
            title=title
            on:click=handle_click
        >
            {icon}
        </button>
    }
}
