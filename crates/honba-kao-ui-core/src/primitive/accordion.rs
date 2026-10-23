//! HfAccordion — vertically stacked collapsible sections.

use leptos::prelude::*;

#[component]
pub fn HfAccordion(
    #[prop(optional)] default_value: Option<&'static str>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let active = RwSignal::new(default_value.unwrap_or("").to_string());
    provide_context(active);

    view! {
        <div
            class=format!("hf-accordion {class}").trim().to_string()
            data-active=move || active.get()
        >
            {children()}
        </div>
    }
}

#[derive(Clone)]
struct AccordionItemKey(String);

#[component]
pub fn HfAccordionItem(
    value: &'static str,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    provide_context(AccordionItemKey(value.to_string()));
    let active = use_context::<RwSignal<String>>().unwrap_or_else(|| RwSignal::new(String::new()));
    let is_open = move || active.get() == value;

    view! {
        <div
            class=move || {
                format!(
                    "hf-accordion__item {} {class}",
                    if is_open() { "hf-accordion__item--open" } else { "" }
                )
                .trim()
                .to_string()
            }
            data-state=move || if is_open() { "open" } else { "closed" }
            data-value=value
        >
            {children()}
        </div>
    }
}

#[component]
pub fn HfAccordionTrigger(
    #[prop(optional)] value: Option<&'static str>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let active = use_context::<RwSignal<String>>().unwrap_or_else(|| RwSignal::new(String::new()));
    let item_key = use_context::<AccordionItemKey>()
        .map(|k| k.0)
        .unwrap_or_else(|| value.unwrap_or("").to_string());
    let key_for_check = item_key.clone();
    let is_open = move || active.get() == key_for_check;

    view! {
        <button
            type="button"
            class=format!("hf-accordion__trigger {class}").trim().to_string()
            aria-expanded=move || is_open().to_string()
            on:click=move |_| {
                let k = item_key.clone();
                active.update(|cur| {
                    if *cur == k {
                        cur.clear();
                    } else {
                        *cur = k;
                    }
                });
            }
        >
            {children()}
        </button>
    }
}

#[component]
pub fn HfAccordionContent(
    #[prop(optional)] value: Option<&'static str>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let active = use_context::<RwSignal<String>>().unwrap_or_else(|| RwSignal::new(String::new()));
    let item_key = use_context::<AccordionItemKey>()
        .map(|k| k.0)
        .unwrap_or_else(|| value.unwrap_or("").to_string());
    let is_open = move || active.get() == item_key;

    view! {
        <div
            role="region"
            class=move || {
                format!(
                    "hf-accordion__content {} {class}",
                    if is_open() { "hf-accordion__content--open" } else { "hf-accordion__content--hidden" }
                )
                .trim()
                .to_string()
            }
            aria-hidden=move || (!is_open()).to_string()
        >
            {children()}
        </div>
    }
}
