//! `HfAgentConsole` — terminal console panel showing AI/quant research reasoning
//! steps and logs. Ported from the restructure spec (Section 7.6) and
//! OpenTerminalUI `AiVisuals`.

use leptos::prelude::*;
use crate::primitive::{HfThinkingOrb, HfKbdBadge};

#[derive(Clone, Debug, PartialEq)]
pub struct AgentThoughtStep {
    pub agent_name: String,
    pub step_type: String, // "thought", "tool_call", "artifact"
    pub content: String,
    pub timestamp: String,
}

#[component]
pub fn HfAgentConsole(
    #[prop(into)] open: RwSignal<bool>,
    #[prop(into)] thoughts: Signal<Vec<AgentThoughtStep>>,
    #[prop(into)] is_streaming: Signal<bool>,
    #[prop(into)] on_submit_prompt: Callback<String>,
) -> impl IntoView {
    let prompt_input = RwSignal::new(String::new());

    let send_message = move || {
        let text = prompt_input.get().trim().to_string();
        if !text.is_empty() {
            prompt_input.set(String::new());
            on_submit_prompt.run(text);
        }
    };

    view! {
        <div class=move || format!("hf-agent-drawer fixed top-0 right-0 h-full w-[440px] z-40 bg-surface-base border-l border-border-soft flex flex-col shadow-2xl transition-transform duration-300 {}", if open.get() { "translate-x-0" } else { "translate-x-full" })>
            <div class="hf-agent-header flex items-center justify-between p-3 border-b border-border-soft">
                <div class="hf-agent-title flex items-center gap-2">
                    <HfThinkingOrb active=is_streaming />
                    <span class="text-xs font-semibold tracking-wide uppercase text-text-high">"Honba Research Analyst"</span>
                </div>
                <div class="hf-agent-actions flex items-center gap-2">
                    <HfKbdBadge key="Ctrl+J" />
                    <button class="hf-close-btn text-text-muted hover:text-text-high" on:click=move |_| open.set(false)>"✕"</button>
                </div>
            </div>

            <div class="hf-agent-stream-body flex-grow overflow-y-auto p-3 flex flex-col gap-2.5">
                <For
                    each=move || thoughts.get()
                    key=|step| format!("{}-{}", step.timestamp, step.agent_name)
                    children=move |step| {
                        view! {
                            <div class="hf-thought-bubble p-2 rounded bg-surface-subtle border border-border-soft text-xs">
                                <div class="hf-bubble-header flex justify-between text-[10px] text-text-muted mb-1">
                                    <span class="font-mono font-bold text-accent-primary">{step.agent_name}</span>
                                    <span>{step.timestamp}</span>
                                </div>
                                <div class="hf-bubble-content font-sans text-text-high whitespace-pre-wrap">{step.content}</div>
                            </div>
                        }
                    }
                />
            </div>

            <div class="hf-agent-input-bar p-3 border-t border-border-soft flex items-center gap-2">
                <input
                    type="text"
                    class="hf-agent-prompt-box flex-grow p-2 rounded bg-surface-subtle text-xs text-text-high outline-none border border-border-soft focus:border-accent-primary"
                    placeholder="Ask analyst (e.g. 'Synthesize Nifty straddle Greeks vs realized vol')..."
                    prop:value=prompt_input
                    on:input=move |ev| prompt_input.set(event_target_value(&ev))
                    on:keydown=move |ev: web_sys::KeyboardEvent| {
                        if ev.key() == "Enter" {
                            send_message();
                        }
                    }
                />
                <button class="hf-send-btn px-3 py-2 rounded bg-accent-primary text-text-inverted text-xs font-semibold hover:opacity-90" on:click=move |_| send_message()>"Send"</button>
            </div>
        </div>
    }
}
