//! HfToggle & HfToggleGroup — two-state press button(s) (`aria-pressed`).

use leptos::prelude::*;

#[component]
pub fn HfToggle(
    #[prop(optional)] pressed: Option<bool>,
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_change: Option<Callback<bool>>,
    children: Children,
) -> impl IntoView {
    let state = RwSignal::new(pressed.unwrap_or(false));
    let disabled = disabled.unwrap_or(false);

    view! {
        <button
            type="button"
            disabled=disabled
            aria-pressed=move || state.get().to_string()
            data-state=move || if state.get() { "on" } else { "off" }
            class=move || {
                format!(
                    "hf-toggle {} {class}",
                    if state.get() { "hf-toggle--on" } else { "" }
                )
                .trim()
                .to_string()
            }
            on:click=move |_| {
                if !disabled {
                    let next = !state.get_untracked();
                    state.set(next);
                    if let Some(cb) = on_change {
                        cb.run(next);
                    }
                }
            }
        >
            {children()}
        </button>
    }
}

#[derive(Clone, Copy)]
struct ToggleGroupActive(RwSignal<String>);

#[component]
pub fn HfToggleGroup(
    #[prop(optional)] default_value: Option<&'static str>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let active = RwSignal::new(default_value.unwrap_or("").to_string());
    provide_context(ToggleGroupActive(active));

    view! {
        <div
            role="group"
            class=format!("hf-toggle-group {class}").trim().to_string()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn HfToggleGroupItem(
    value: &'static str,
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_select: Option<Callback<&'static str>>,
    children: Children,
) -> impl IntoView {
    let group = use_context::<ToggleGroupActive>().map(|g| g.0);
    let disabled = disabled.unwrap_or(false);
    let is_on = move || group.map(|g| g.get() == value).unwrap_or(false);

    view! {
        <button
            type="button"
            disabled=disabled
            aria-pressed=move || is_on().to_string()
            data-state=move || if is_on() { "on" } else { "off" }
            data-value=value
            class=move || {
                format!(
                    "hf-toggle-group__item {} {class}",
                    if is_on() { "hf-toggle-group__item--on" } else { "" }
                )
                .trim()
                .to_string()
            }
            on:click=move |_| {
                if !disabled {
                    if let Some(g) = group {
                        g.set(value.to_string());
                    }
                    if let Some(cb) = on_select {
                        cb.run(value);
                    }
                }
            }
        >
            {children()}
        </button>
    }
}
