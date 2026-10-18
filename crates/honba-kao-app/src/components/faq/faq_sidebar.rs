//! FAQ category sidebar + search. Mirrors
//! `web/src/components/faq/FAQSidebar.tsx` and `FAQSearchBar.tsx`.

use leptos::prelude::*;

use crate::data::faq_data::faq_categories;

#[component]
pub fn FaqSidebar(active: RwSignal<String>) -> impl IntoView {
    view! {
        <aside class="faq-sidebar">
            <nav class="faq-sidebar__cats">
                {faq_categories()
                    .into_iter()
                    .map(|cat| {
                        let value = cat.to_string();
                        let is_active = move || active.get() == value;
                        let label = cat;
                        view! {
                            <button
                                class=move || if is_active() { "faq-cat faq-cat--active" } else { "faq-cat" }
                                on:click=move |_| active.set(cat.to_string())
                            >
                                {label}
                            </button>
                        }
                    })
                    .collect_view()}
                <button
                    class=move || if active.get().is_empty() { "faq-cat faq-cat--active" } else { "faq-cat" }
                    on:click=move |_| active.set(String::new())
                >
                    "All"
                </button>
            </nav>
        </aside>
    }
}
