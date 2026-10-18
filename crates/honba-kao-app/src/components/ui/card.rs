//! Card container: a titled surface used to group page content.

use leptos::prelude::*;

#[component]
pub fn HfCard(#[prop(default = "")] title: &'static str, children: Children) -> impl IntoView {
    view! {
        <section class="card">
            {(!title.is_empty()).then(|| view! {
                <header class="card__header">{title}</header>
            })}
            <div class="card__body">{children()}</div>
        </section>
    }
}
