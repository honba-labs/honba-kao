//! AiInsightCard — titled card listing AI-generated insight bullets.

use leptos::prelude::*;

/// Card with a title and a bullet list of insight strings.
#[component]
pub fn AiInsightCard(
    #[prop(into)] title: String,
    #[prop(into)] bullets: Vec<String>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let items = bullets
        .into_iter()
        .map(|bullet| view! { <li class="hf-ai-insight-card__item">{bullet}</li> })
        .collect_view();

    view! {
        <article class=format!("hf-ai-insight-card {class}").trim().to_string()>
            <h3 class="hf-ai-insight-card__title">{title}</h3>
            <ul class="hf-ai-insight-card__list">{items}</ul>
        </article>
    }
}
