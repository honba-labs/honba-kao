//! FAQ entries with search + category filter. Mirrors
//! `web/src/components/faq/FAQContent.tsx`.

use leptos::prelude::*;

use crate::data::faq_data::FAQ_ENTRIES;

#[component]
pub fn FaqContent(active: RwSignal<String>) -> impl IntoView {
    let query = RwSignal::new(String::new());

    let filtered = move || {
        let q = query.get().to_lowercase();
        let cat = active.get();
        FAQ_ENTRIES
            .iter()
            .filter(|e| cat.is_empty() || e.category == cat)
            .filter(|e| {
                q.is_empty()
                    || e.question.to_lowercase().contains(&q)
                    || e.answer.to_lowercase().contains(&q)
            })
            .copied()
            .collect::<Vec<_>>()
    };

    view! {
        <section class="faq-content">
            <input
                class="faq-search"
                type="search"
                placeholder="Search the FAQ…"
                prop:value=move || query.get()
                on:input=move |ev| query.set(event_target_value(&ev))
            />

            <div class="faq-entries">
                {move || filtered()
                    .into_iter()
                    .map(|entry| {
                        view! {
                            <details class="faq-entry">
                                <summary>
                                    <span class="faq-entry__q">{entry.question}</span>
                                    <span class="faq-entry__cat">{entry.category}</span>
                                </summary>
                                <p class="faq-entry__a">{entry.answer}</p>
                            </details>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}
