//! `HfGoBar` — fast `Ctrl+G` modal search input with symbol and route auto-complete.
//! Ported from the restructure spec (Section 7.6) and OpenTerminalUI `CommandBar`.

use leptos::prelude::*;
use leptos::html::Input;
use crate::primitive::HfPromptSymbol;

#[component]
pub fn HfGoBar(
    #[prop(into)] open: RwSignal<bool>,
    #[prop(into)] on_navigate: Callback<String>,
) -> impl IntoView {
    let input_ref: NodeRef<Input> = NodeRef::new();
    let query = RwSignal::new(String::new());

    Effect::new(move || {
        if open.get() {
            if let Some(elem) = input_ref.get() {
                let _ = elem.focus();
            }
        }
    });

    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" {
            open.set(false);
        } else if ev.key() == "Enter" {
            let q = query.get().trim().to_uppercase();
            if !q.is_empty() {
                open.set(false);
                on_navigate.run(q);
            }
        }
    };

    view! {
        <Show when=move || open.get()>
            <div class="hf-gobar-backdrop fixed inset-0 z-50 bg-black/60 flex items-start justify-center pt-24" on:click=move |_| open.set(false)>
                <div class="hf-gobar-modal w-[600px] max-w-full rounded-lg bg-surface-base border border-border-strong shadow-2xl p-3" on:click=|ev| ev.stop_propagation()>
                    <div class="hf-gobar-input-row flex items-center gap-2">
                        <HfPromptSymbol />
                        <input
                            node_ref=input_ref
                            type="text"
                            class="hf-gobar-field flex-grow bg-transparent font-mono text-sm text-text-high outline-none"
                            placeholder="GO: Enter symbol (e.g. INFY, BTC), command (/dom, /fno), or action..."
                            prop:value=query
                            on:input=move |ev| query.set(event_target_value(&ev))
                            on:keydown=on_keydown
                        />
                        <span class="hf-kbd-hint text-[10px] text-text-muted border border-border-soft px-1.5 py-0.5 rounded font-mono">"ESC"</span>
                    </div>
                </div>
            </div>
        </Show>
    }
}
