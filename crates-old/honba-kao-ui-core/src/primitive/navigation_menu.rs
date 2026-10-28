//! HfNavigationMenu — top-level site/workbench navigation bar (`<nav>`).

use leptos::prelude::*;

#[component]
pub fn HfNavigationMenu(
    #[prop(optional, into)] aria_label: String,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let aria_label = if aria_label.is_empty() {
        "Main".to_string()
    } else {
        aria_label
    };

    view! {
        <nav
            aria-label=aria_label
            class=format!("hf-navigation-menu {class}").trim().to_string()
        >
            {children()}
        </nav>
    }
}

#[component]
pub fn HfNavigationMenuList(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <ul class=format!("hf-navigation-menu__list {class}").trim().to_string()>
            {children()}
        </ul>
    }
}

#[component]
pub fn HfNavigationMenuItem(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <li class=format!("hf-navigation-menu__item {class}").trim().to_string()>
            {children()}
        </li>
    }
}

#[component]
pub fn HfNavigationMenuLink(
    #[prop(optional, into)] href: String,
    #[prop(optional)] active: Option<bool>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_click: Option<Callback<web_sys::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let active = active.unwrap_or(false);
    let href = (!href.is_empty()).then_some(href);
    let link_class = format!(
        "hf-navigation-menu__link {} {class}",
        if active {
            "hf-navigation-menu__link--active"
        } else {
            ""
        }
    )
    .trim()
    .to_string();

    view! {
        <a
            href=href
            aria-current=active.then_some("page")
            class=link_class
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
pub fn HfNavigationMenuIndicator(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <span
            aria-hidden="true"
            class=format!("hf-navigation-menu__indicator {class}").trim().to_string()
        />
    }
}
