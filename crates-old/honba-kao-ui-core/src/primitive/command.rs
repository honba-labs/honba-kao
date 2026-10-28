//! HfCommand — command palette / filterable action list (`role="combobox"` + `role="listbox"`).

use leptos::prelude::*;

#[derive(Clone, Copy)]
struct CommandQuery(RwSignal<String>);

#[component]
pub fn HfCommand(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let query = RwSignal::new(String::new());
    provide_context(CommandQuery(query));

    view! {
        <div class=format!("hf-command {class}").trim().to_string()>
            {children()}
        </div>
    }
}

#[component]
pub fn HfCommandInput(
    #[prop(optional)] placeholder: Option<&'static str>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_input: Option<Callback<String>>,
) -> impl IntoView {
    let query = use_context::<CommandQuery>().map(|q| q.0);

    view! {
        <div class="hf-command__input-wrapper">
            <input
                type="text"
                role="combobox"
                aria-expanded="true"
                aria-autocomplete="list"
                placeholder=placeholder.unwrap_or("Type a command or search...")
                class=format!("hf-command__input {class}").trim().to_string()
                on:input=move |ev| {
                    let val = event_target_value(&ev);
                    if let Some(q) = query {
                        q.set(val.clone());
                    }
                    if let Some(cb) = on_input {
                        cb.run(val);
                    }
                }
            />
        </div>
    }
}

#[component]
pub fn HfCommandList(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <div
            role="listbox"
            class=format!("hf-command__list {class}").trim().to_string()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn HfCommandItem(
    #[prop(optional, into)] value: String,
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_select: Option<Callback<web_sys::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let query = use_context::<CommandQuery>().map(|q| q.0);
    let disabled = disabled.unwrap_or(false);
    let value_lower = value.to_lowercase();
    let visible = move || {
        let Some(q) = query else { return true };
        let needle = q.get().to_lowercase();
        needle.is_empty() || value_lower.is_empty() || value_lower.contains(&needle)
    };

    view! {
        <div
            role="option"
            aria-disabled=disabled.to_string()
            data-value=value
            class=move || {
                format!(
                    "hf-command__item {} {} {class}",
                    if disabled { "hf-command__item--disabled" } else { "" },
                    if visible() { "" } else { "hf-command__item--hidden" }
                )
                .trim()
                .to_string()
            }
            on:click=move |ev| {
                if !disabled
                    && let Some(cb) = on_select {
                        cb.run(ev);
                    }
            }
        >
            {children()}
        </div>
    }
}

#[component]
pub fn HfCommandSeparator(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <div role="separator" class=format!("hf-command__separator {class}").trim().to_string() />
    }
}

#[component]
pub fn HfCommandShortcut(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <kbd class=format!("hf-command__shortcut {class}").trim().to_string()>
            {children()}
        </kbd>
    }
}
