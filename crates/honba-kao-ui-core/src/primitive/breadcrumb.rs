//! HfBreadcrumb — hierarchical breadcrumb navigation (`<nav aria-label="breadcrumb">`).

use leptos::prelude::*;

#[component]
pub fn HfBreadcrumb(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <nav
            aria-label="breadcrumb"
            class=format!("hf-breadcrumb {class}").trim().to_string()
        >
            <ol class="hf-breadcrumb__list">
                {children()}
            </ol>
        </nav>
    }
}

#[component]
pub fn HfBreadcrumbItem(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <li class=format!("hf-breadcrumb__item {class}").trim().to_string()>
            {children()}
        </li>
    }
}

#[component]
pub fn HfBreadcrumbLink(
    #[prop(optional, into)] href: String,
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_click: Option<Callback<web_sys::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let href = (!href.is_empty()).then_some(href);

    view! {
        <a
            href=href
            class=format!("hf-breadcrumb__link {class}").trim().to_string()
            on:click=move |ev| {
                if let Some(cb) = on_click {
                    cb.run(ev);
                }
            }
        >
            {children()}
        </a>
    }
}

#[component]
pub fn HfBreadcrumbPage(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <span
            role="link"
            aria-disabled="true"
            aria-current="page"
            class=format!("hf-breadcrumb__page {class}").trim().to_string()
        >
            {children()}
        </span>
    }
}

#[component]
pub fn HfBreadcrumbSeparator(
    #[prop(optional, into)] class: String,
    #[prop(optional)] label: Option<&'static str>,
) -> impl IntoView {
    view! {
        <li
            role="presentation"
            aria-hidden="true"
            class=format!("hf-breadcrumb__separator {class}").trim().to_string()
        >
            {label.unwrap_or("/")}
        </li>
    }
}

#[component]
pub fn HfBreadcrumbEllipsis(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <span
            role="presentation"
            aria-hidden="true"
            class=format!("hf-breadcrumb__ellipsis {class}").trim().to_string()
        >
            "\u{2026}"
        </span>
    }
}
