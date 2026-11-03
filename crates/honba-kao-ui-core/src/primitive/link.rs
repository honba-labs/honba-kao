//! `HfLink` — accessible link element with external indicator icon and hover states.
//! Ported from `secured-finance-app` atom `TextLink`.

use leptos::prelude::*;
use crate::icons::SvgExternalLink;

#[component]
pub fn HfLink(
    #[prop(into)] href: String,
    #[prop(into, optional)] external: Option<bool>,
    #[prop(into, optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let external = external.unwrap_or(false);
    let base = "hf-link text-emerald-400 hover:text-emerald-300 hover:underline transition-colors";
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    view! {
        <a
            href=href
            target=if external { Some("_blank".to_string()) } else { None }
            rel=if external { Some("noopener noreferrer".to_string()) } else { None }
            class=classes
        >
            {children()}
            {external.then(|| view! {
                <span class="hf-icon hf-icon-external ml-1" aria_hidden="true">
                    <SvgExternalLink />
                </span>
            }).into_view()}
        </a>
    }
}
