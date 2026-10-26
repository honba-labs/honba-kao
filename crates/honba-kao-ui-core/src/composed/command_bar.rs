//! Presentational pieces of the command bar (`Ctrl+G`) and palette (`Ctrl+K`).
//!
//! Layout and classes follow OpenTerminalUI `layout/CommandBar.tsx` and
//! `layout/CommandPalette.tsx`. All state and command routing live in the
//! app crate (`shared::components::command_bar`); these components only render
//! what they are handed and report input back through callbacks.

use leptos::html;
use leptos::prelude::*;

use crate::trading::{HfMiniSparkline, HfProvenanceChip, ProvenanceQuality};

/// One suggestion row. `badge` is the small tag shown at the row end
/// (`FUNC`, `ASSET`, `NAV`, `CMD`, `HIST`), matching the palette's badges.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandSuggestion {
    pub key: String,
    pub title: String,
    pub subtitle: String,
    pub badge: &'static str,
}

/// The bar container. `aria-label` mirrors OpenTerminalUI's "Command palette".
#[component]
pub fn HfCommandBar(
    #[prop(optional, into)] class: String,
    /// True while the suggestion list is open (drives `role="dialog"`).
    #[prop(optional)]
    open: Option<Signal<bool>>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=format!("hf-command-bar {class}").trim().to_string()
            role="combobox"
            aria-label="Command palette"
            aria-expanded=move || open.map(|o| o.get()).unwrap_or(false).to_string()
        >
            {children()}
        </div>
    }
}

/// The text input. The value is controlled by `value`; keystrokes go to
/// `on_keydown` so the app can handle arrows, Enter and Escape.
#[component]
pub fn HfCommandBarInput(
    value: RwSignal<String>,
    node_ref: NodeRef<html::Input>,
    #[prop(optional)] placeholder: Option<&'static str>,
    #[prop(optional)] on_keydown: Option<Callback<web_sys::KeyboardEvent>>,
    #[prop(optional)] on_focus: Option<Callback<web_sys::FocusEvent>>,
    #[prop(optional)] on_blur: Option<Callback<web_sys::FocusEvent>>,
) -> impl IntoView {
    view! {
        <input
            node_ref=node_ref
            type="text"
            class="hf-command-bar__input"
            autocomplete="off"
            spellcheck="false"
            aria-autocomplete="list"
            placeholder=placeholder.unwrap_or("Type ticker, command, or search...")
            prop:value=move || value.get()
            on:input=move |ev| value.set(event_target_value(&ev))
            on:keydown=move |ev| {
                if let Some(cb) = on_keydown {
                    cb.run(ev);
                }
            }
            on:focus=move |ev| {
                if let Some(cb) = on_focus {
                    cb.run(ev);
                }
            }
            on:blur=move |ev| {
                if let Some(cb) = on_blur {
                    cb.run(ev);
                }
            }
        />
    }
}

/// The suggestion list. `selected` is the highlighted index; `on_pick` fires
/// with the index of a clicked row.
#[component]
pub fn HfCommandSuggestions(
    items: Signal<Vec<CommandSuggestion>>,
    selected: Signal<usize>,
    #[prop(optional)] on_pick: Option<Callback<usize>>,
    /// Heading shown above the list, e.g. "Suggestions" or "Reverse History Search (Ctrl+R)".
    #[prop(into)]
    heading: Signal<String>,
) -> impl IntoView {
    let rows = move || {
        let list = items.get();
        if list.is_empty() {
            return view! { <div class="hf-command-suggestions__empty">"No matches"</div> }.into_any();
        }
        list.into_iter()
            .enumerate()
            .map(|(idx, item)| {
                let row_class = move || {
                    if selected.get() == idx {
                        "hf-command-suggestions__row hf-command-suggestions__row--active"
                    } else {
                        "hf-command-suggestions__row"
                    }
                };
                view! {
                    <div
                        role="option"
                        class=row_class
                        aria-selected=move || (selected.get() == idx).to_string()
                        on:mousedown=move |ev: web_sys::MouseEvent| {
                            // Keep focus in the input so picking a row does not blur the bar.
                            ev.prevent_default();
                            if let Some(cb) = on_pick {
                                cb.run(idx);
                            }
                        }
                    >
                        <span class="hf-command-suggestions__title">{item.title}</span>
                        <span class="hf-command-suggestions__subtitle">{item.subtitle}</span>
                        <span class="hf-command-suggestions__badge">{item.badge}</span>
                    </div>
                }
            })
            .collect_view()
            .into_any()
    };

    view! {
        <div class="hf-command-suggestions" role="listbox">
            <div class="hf-command-suggestions__heading">{move || heading.get()}</div>
            {rows}
        </div>
    }
}

/// Hover/selection preview card: LTP, change, asset class, exchange and a sparkline.
#[component]
pub fn HfAssetPreviewCard(
    ticker: String,
    name: String,
    exchange: String,
    asset_class: String,
    ltp: String,
    change_pct: String,
    series: Vec<f64>,
    #[prop(optional)] quality: Option<ProvenanceQuality>,
) -> impl IntoView {
    let up = !change_pct.trim_start().starts_with('-');
    let change_class = if up { "hf-asset-preview__change hf-asset-preview__change--up" } else { "hf-asset-preview__change hf-asset-preview__change--down" };
    let arrow = if up { "▲" } else { "▼" };
    view! {
        <div class="hf-asset-preview">
            <div class="hf-asset-preview__head">
                <span class="hf-asset-preview__ticker">{ticker}</span>
                <span class="hf-asset-preview__exchange">{exchange}</span>
                <span class="hf-asset-preview__class">{asset_class}</span>
            </div>
            <div class="hf-asset-preview__name">{name}</div>
            <div class="hf-asset-preview__quote">
                <span class="hf-asset-preview__ltp">{ltp}</span>
                <span class=change_class>{format!("{arrow} {change_pct}%")}</span>
                {quality.map(|q| view! { <HfProvenanceChip quality=q compact=true /> })}
            </div>
            <HfMiniSparkline values=series width=160.0 height=28.0 class="hf-asset-preview__spark" />
        </div>
    }
}
