//! `HfAgentConsole` — terminal console panel showing AI/quant research reasoning
//! steps and logs. Ported from the restructure spec (Section 7.6) and
//! OpenTerminalUI `AiVisuals`.
//!
//! The prompt box is a raw [`nsh_core::element::input`] rather than
//! [`crate::primitive::base::input::HfInput`]: Enter-to-submit needs
//! `.on_submit`, which `HfInput` does not expose (it only reports
//! [`crate::primitive::base::input::HfInputEvent::Change`]), and adding a
//! `Submit` variant there would break the exhaustive matches of its three
//! existing callers.

use nsh_core::element::input;
use nsh_core::prelude::*;

use crate::primitive::base::button::{
    ButtonSize, ButtonVariant, HfButton, HfButtonEvent, HfButtonProps, HfCloseButton,
    HfCloseButtonProps,
};
use crate::primitive::kbd::{HfKbdBadge, HfKbdBadgeProps};
use crate::primitive::prompt::{HfThinkingOrb, HfThinkingOrbProps};
use crate::theme::tokens::HonbaStyle;

#[derive(Clone, Debug, Default, PartialEq, JsObject)]
pub struct AgentThoughtStep {
    pub agent_name: String,
    pub step_type: String, // "thought", "tool_call", "artifact"
    pub content: String,
    pub timestamp: String,
}

/// The events of [`HfAgentConsole`].
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfAgentConsoleEvent {
    /// The close button was pressed. The caller flips its own `open` state.
    Close,
    /// A non-empty, trimmed prompt was submitted (Enter or Send).
    SubmitPrompt(String),
}

/// Props for [`HfAgentConsole`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfAgentConsoleProps {
    pub open: Prop<bool>,
    pub thoughts: Prop<Vec<AgentThoughtStep>>,
    pub is_streaming: Prop<bool>,
}

/// Trim the draft, clear it and emit `SubmitPrompt`; empty drafts are ignored.
fn send_message(draft: Signal<String>, cx: &Scope<HfAgentConsoleEvent>) {
    let text = draft.get().trim().to_string();
    if !text.is_empty() {
        draft.set(String::new());
        cx.emit(HfAgentConsoleEvent::SubmitPrompt(text));
    }
}

/// One thought step: agent name and timestamp, then the content.
fn thought_bubble(entry: ReadSignal<(usize, AgentThoughtStep)>) -> View {
    view()
        .flex_col()
        .gap(px(4.0))
        .px(px(8.0))
        .py(px(8.0))
        .rounded(px(4.0))
        .border(px(1.0))
        .border_soft()
        .surface_muted()
        .child(
            view()
                .flex_row()
                .justify_between()
                .child(
                    text(move || entry.get().1.agent_name)
                        .text_size(px(10.0))
                        .font_family("monospace")
                        .font_weight(FontWeight::BOLD)
                        .text_positive(),
                )
                .child(
                    text(move || entry.get().1.timestamp)
                        .text_size(px(10.0))
                        .text_muted(),
                ),
        )
        .child(
            text(move || entry.get().1.content)
                .text_size(px(12.0))
                .text_high(),
        )
}

/// Right-side analyst drawer: header (orb, title, Ctrl+J badge, close), a
/// scrollable list of thought steps, and a prompt input with Send.
#[component]
pub fn HfAgentConsole(
    props: HfAgentConsoleProps,
    cx: Scope<HfAgentConsoleEvent>,
) -> impl IntoElement {
    let open = props.open;
    let thoughts = props.thoughts;
    let is_streaming = props.is_streaming;
    let draft = signal(String::new());

    let close_cx = cx.clone();
    let close = HfCloseButton::element(HfCloseButtonProps {
        aria_label: Prop::from(Some("Close analyst console".to_string())),
        ..Default::default()
    })
    .on_event(move |event: &HfButtonEvent| {
        if matches!(event, HfButtonEvent::Press) {
            close_cx.emit(HfAgentConsoleEvent::Close);
        }
    });

    let header = view()
        .flex_row()
        .items_center()
        .justify_between()
        .px(px(12.0))
        .py(px(12.0))
        .border_b_1()
        .border_soft()
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(8.0))
                .child(HfThinkingOrb::element(HfThinkingOrbProps {
                    size: Prop::from(16.0),
                }))
                .child(
                    text("HONBA RESEARCH ANALYST")
                        .text_size(px(11.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_high(),
                )
                .child(
                    text(move || {
                        if is_streaming.get() {
                            "streaming".to_string()
                        } else {
                            String::new()
                        }
                    })
                    .text_size(px(10.0))
                    .text_muted(),
                ),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(8.0))
                .child(HfKbdBadge::element(HfKbdBadgeProps {
                    key: Prop::from("Ctrl+J".to_string()),
                }))
                .child(close),
        );

    let body = scroll().flex_1().child(
        view()
            .flex_col()
            .gap(px(10.0))
            .px(px(12.0))
            .py(px(12.0))
            .child(each(
                move || thoughts.get().into_iter().enumerate().collect::<Vec<_>>(),
                |(index, _): &(usize, AgentThoughtStep)| *index,
                move |entry: ReadSignal<(usize, AgentThoughtStep)>, _row: ReadSignal<usize>| {
                    thought_bubble(entry)
                },
            )),
    );

    let submit_cx = cx.clone();
    let prompt = input()
        .value(move || draft.get())
        .placeholder(
            "Ask analyst (e.g. 'Synthesize Nifty straddle Greeks vs realized vol')...".to_string(),
        )
        .flex_1()
        .rounded(px(4.0))
        .border(px(1.0))
        .border_soft()
        .px(px(8.0))
        .py(px(8.0))
        .text_size(px(12.0))
        .on_change(move |s: &str| draft.set(s.to_string()))
        .on_submit(move |_: &str| send_message(draft, &submit_cx));

    let send_cx = cx.clone();
    let send = HfButton::element(HfButtonProps {
        variant: Prop::from(Some(ButtonVariant::Primary)),
        size: Prop::from(Some(ButtonSize::Sm)),
        aria_label: Prop::from(Some("Send".to_string())),
        ..Default::default()
    })
    .child(text("Send"))
    .on_event(move |event: &HfButtonEvent| {
        if matches!(event, HfButtonEvent::Press) {
            send_message(draft, &send_cx);
        }
    });

    let input_bar = view()
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .px(px(12.0))
        .py(px(12.0))
        .border_t(px(1.0))
        .border_soft()
        .child(prompt)
        .child(send);

    view()
        .flex_col()
        .w(px(440.0))
        .h_full()
        .border_l_1()
        .border_soft()
        .surface_base()
        .shadow_2xl()
        .app_class(move || {
            Some(SharedString::from(format!(
                "hf-agent-drawer fixed top-0 right-0 z-40 transition-transform duration-300 {}",
                if open.get() {
                    "translate-x-0"
                } else {
                    "translate-x-full"
                }
            )))
        })
        .child(header)
        .child(body)
        .child(input_bar)
}
