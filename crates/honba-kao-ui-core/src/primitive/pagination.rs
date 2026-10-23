//! HfPagination — accessible page navigation (`<nav aria-label="pagination">`).

use leptos::prelude::*;

#[component]
pub fn HfPagination(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <nav
            role="navigation"
            aria-label="pagination"
            class=format!("hf-pagination {class}").trim().to_string()
        >
            <ul class="hf-pagination__content">
                {children()}
            </ul>
        </nav>
    }
}

#[component]
pub fn HfPaginationItem(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <li class=format!("hf-pagination__item {class}").trim().to_string()>
            {children()}
        </li>
    }
}

#[component]
pub fn HfPaginationLink(
    #[prop(optional)] active: Option<bool>,
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional, into)] href: String,
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_click: Option<Callback<web_sys::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let active = active.unwrap_or(false);
    let disabled = disabled.unwrap_or(false);
    let href = (!href.is_empty()).then_some(href);
    let link_class = format!(
        "hf-pagination__link {} {} {class}",
        if active {
            "hf-pagination__link--active"
        } else {
            ""
        },
        if disabled {
            "hf-pagination__link--disabled"
        } else {
            ""
        },
    )
    .trim()
    .to_string();

    view! {
        <a
            href=href
            class=link_class
            aria-current=active.then_some("page")
            aria-disabled=disabled.then_some("true")
            on:click=move |ev| {
                if !disabled {
                    if let Some(cb) = on_click {
                        cb.run(ev);
                    }
                }
            }
        >
            {children()}
        </a>
    }
}
