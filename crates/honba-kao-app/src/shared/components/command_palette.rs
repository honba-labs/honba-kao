//! CommandPalette: the modal command runner (`Ctrl+K`).
//!
//! Port of OpenTerminalUI `layout/CommandPalette.tsx`: `Ctrl+K` toggles the
//! modal, typing filters NAV / ASSET / FUNC / CMD rows, arrows move the
//! highlight, `Enter` runs the highlighted row, `Escape` closes. Rows are
//! built by [`crate::shared::command::palette::build_items`].

use honba_kao_ui_core::composed::command_bar::{CommandSuggestion, HfCommandBarInput, HfCommandSuggestions};
use honba_kao_ui_core::composed::modal::HfModal;
use leptos::html;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;

use crate::shared::command::commanding::{CommandOutcome, parse_command, route_for};
use crate::shared::command::palette::{PaletteItem, PaletteTarget, build_items};

#[component]
pub fn CommandPalette() -> impl IntoView {
    let navigate = use_navigate();
    let open = RwSignal::new(false);
    let query = RwSignal::new(String::new());
    let selected = RwSignal::new(0usize);
    let feedback = RwSignal::new(None::<String>);
    let input_ref = NodeRef::<html::Input>::new();

    let items = Memo::new(move |_| build_items(&query.get()));
    let views = Signal::derive(move || {
        items
            .get()
            .into_iter()
            .map(|item: PaletteItem| CommandSuggestion {
                key: item.key,
                title: item.label,
                subtitle: item.description,
                badge: item.badge,
            })
            .collect::<Vec<_>>()
    });

    Effect::new(move |_| {
        let _ = items.get();
        selected.set(0);
    });

    // Focus the query field each time the palette opens.
    Effect::new(move |_| {
        if open.get() {
            if let Some(el) = input_ref.get() {
                let _ = el.focus();
            }
        }
    });

    let close = Callback::new(move |_: ()| {
        open.set(false);
        feedback.set(None);
    });

    let run = Callback::new(move |idx: usize| {
        let Some(item) = items.get_untracked().get(idx).cloned() else { return };
        let outcome = match &item.target {
            PaletteTarget::Route(route) => CommandOutcome::Navigate(route.clone()),
            PaletteTarget::Command(text) => route_for(&parse_command(text)),
        };
        match outcome {
            CommandOutcome::Navigate(route) => {
                navigate(&route, NavigateOptions::default());
                open.set(false);
                feedback.set(None);
            }
            CommandOutcome::Unsupported(message) => feedback.set(Some(message)),
        }
    });

    // `Ctrl+K` toggles; `Escape` closes. Installed once for the chrome's lifetime.
    let handle = window_event_listener(leptos::ev::keydown, move |ev: web_sys::KeyboardEvent| {
        let key = ev.key();
        if (ev.ctrl_key() || ev.meta_key()) && key.eq_ignore_ascii_case("k") {
            ev.prevent_default();
            if open.get_untracked() {
                close.run(());
            } else {
                query.set(String::new());
                selected.set(0);
                open.set(true);
            }
        } else if key == "Escape" && open.get_untracked() {
            close.run(());
        }
    });
    on_cleanup(move || handle.remove());

    let on_keydown = Callback::new(move |ev: web_sys::KeyboardEvent| match ev.key().as_str() {
        "ArrowDown" => {
            ev.prevent_default();
            let len = items.get_untracked().len();
            if len > 0 {
                selected.update(|i| *i = (*i + 1).min(len - 1));
            }
        }
        "ArrowUp" => {
            ev.prevent_default();
            selected.update(|i| *i = i.saturating_sub(1));
        }
        "Enter" => {
            ev.prevent_default();
            run.run(selected.get_untracked());
        }
        _ => {}
    });

    let on_pick = Callback::new(move |idx: usize| run.run(idx));

    view! {
        <Show when=move || open.get()>
            <HfModal title="Command palette" on_close=close class="app-command-palette">
                <HfCommandBarInput
                    value=query
                    node_ref=input_ref
                    placeholder="Search routes, commands, symbols..."
                    on_keydown=on_keydown
                />
                <HfCommandSuggestions
                    items=views
                    selected=Signal::derive(move || selected.get())
                    on_pick=on_pick
                    heading=Signal::derive(|| "Results (Ctrl+K)".to_string())
                />
                {move || feedback.get().map(|msg| view! { <div class="app-command-bar__feedback">{msg}</div> })}
            </HfModal>
        </Show>
    }
}
