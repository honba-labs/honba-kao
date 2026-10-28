//! `HfGoBar` — fast `Ctrl+G` modal search input with symbol and route
//! auto-complete. Ported from the restructure spec (Section 7.6) and
//! OpenTerminalUI `CommandBar`.

use leptos::prelude::*;
use leptos::html::{Input, Ul};
use crate::primitive::HfPromptSymbol;

#[component]
pub fn HfGoBar(
    #[prop(into)] open: RwSignal<bool>,
    #[prop(into)] on_navigate: Callback<String>,
    #[prop(into, optional)] autocomplete: Option<Vec<String>>,
) -> impl IntoView {
    let input_ref: NodeRef<Input> = NodeRef::new();
    let query = RwSignal::new(String::new());
    let suggestions = RwSignal::new(Vec::<String>::new());
    let highlighted = RwSignal::new(0_usize);
    let list_ref: NodeRef<Ul> = NodeRef::new();

    let opts = autocomplete.unwrap_or_default();

    Effect::new(move |_| {
        if open.get() {
            query.set(String::new());
            suggestions.set(Vec::new());
            highlighted.set(0);
            if let Some(elem) = input_ref.get() {
                let _ = elem.focus();
            }
        }
    });

    let filtered = Signal::derive(move || {
        let q = query.get().trim().to_uppercase();
        if q.is_empty() {
            Vec::<String>::new()
        } else {
            opts.iter()
                .filter(|s| s.to_uppercase().contains(&q))
                .cloned()
                .collect()
        }
    });

    // Keep highlight inside the visible suggestion list.
    Effect::new(move |_| {
        let n = filtered.get().len();
        if n == 0 {
            highlighted.set(0);
        } else if highlighted.get() >= n {
            highlighted.set(n.saturating_sub(1));
        }
    });

    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        let n = filtered.get().len();
        match ev.key().as_str() {
            "Escape" => open.set(false),
            "Enter" => {
                let sel = highlighted.get();
                if n > 0 && sel < n {
                    let choice = filtered.get()[sel].clone();
                    open.set(false);
                    on_navigate.run(choice);
                } else {
                    let q = query.get().trim().to_uppercase();
                    if !q.is_empty() {
                        open.set(false);
                        on_navigate.run(q);
                    }
                }
            }
            "ArrowDown" => {
                ev.prevent_default();
                highlighted.set((highlighted.get() + 1).max(1).min(n.saturating_sub(1).max(1)));
                if let Some(list) = list_ref.get() {
                    let _ = list.scroll_into_view();
                }
            }
            "ArrowUp" => {
                ev.prevent_default();
                highlighted.set(highlighted.get().saturating_sub(1).min(n.saturating_sub(1)));
            }
            _ => {}
        }
    };

    let set_query = move |ev| query.set(event_target_value(&ev));

    view! {
        <Show when=move || open.get()>
            <div class="hf-gobar-backdrop fixed inset-0 z-50 bg-black/60 flex items-start justify-center pt-24" on:click=move |_| open.set(false)>
                <div class="hf-gobar-modal w-[600px] max-w-full rounded-lg bg-surface-base border border-border-strong shadow-2xl p-3 flex flex-col" on:click=|ev| ev.stop_propagation()>
                    <div class="hf-gobar-input-row flex items-center gap-2 mb-2">
                        <HfPromptSymbol />
                        <input
                            node_ref=input_ref
                            type="text"
                            class="hf-gobar-field flex-grow bg-transparent font-mono text-sm text-text-high outline-none"
                            placeholder="GO: Enter symbol (e.g. INFY, BTC), command (/dom, /fno), or action..."
                            prop:value=query
                            on:input=set_query
                            on:keydown=on_keydown
                        />
                        <span class="hf-kbd-hint text-[10px] text-text-muted border border-border-soft px-1.5 py-0.5 rounded font-mono">"ESC"</span>
                    </div>
                    <Show when=move || !filtered.get().is_empty()>
                        <ul
                            node_ref=list_ref
                            class="hf-gobar-suggestions max-h-[200px] overflow-y-auto border border-border-soft bg-surface-base rounded mb-2 z-10"
                            on:keydown=on_keydown
                        >
                            {move || {
                                filtered.get().into_iter().enumerate().map(|(i, suggestion)| {
                                    let active = highlighted.get() == i;
                                    let suggestion_clone = suggestion.clone();
                                    view! {
                                        <li
                                            class=move || format!(
                                                "hf-gobar-suggestion px-3 py-1.5 text-xs cursor-pointer text-text-high border-b border-border-soft/50 last:border-0 {}",
                                                if active { "bg-emerald-600/20 text-emerald-400 font-bold" } else { "hover:bg-surface-subtle" }
                                            )
                                            on:click=move |_| {
                                                open.set(false);
                                                on_navigate.run(suggestion_clone.clone());
                                            }
                                        >
                                            {suggestion}
                                        </li>
                                    }
                                }).collect::<Vec<_>>()
                            }}
                        </ul>
                    </Show>
                    <span class="hf-kbd-hint text-[10px] text-text-muted">
                        {move || {
                            let n = filtered.get().len();
                            if n > 0 {
                                format!("↑/↓ navigate · Enter select · {} match{}", n, if n == 1 { "" } else { "es" })
                            } else {
                                "↑/↓ navigate · Enter submit".to_string()
                            }
                        }}
                    </span>
                </div>
            </div>
        </Show>
    }
}
