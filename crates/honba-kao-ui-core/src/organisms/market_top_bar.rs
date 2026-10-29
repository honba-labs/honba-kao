//! Market top bar organism — NSH port per nsh-port.md §4 Phase 3.
//!
//! A top strip of market status pills with the desk/currency context.
//! Built from `nsh_core` primitives; honba styling applied via `.class()`/`.style()`.

use nsh_core::prelude::*;

/// Props for [`HfMarketTopBar`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfMarketTopBarProps {
    /// Pill labels (e.g. "NIFTY 50").
    pub labels: Prop<Vec<String>>,
    /// Pill values (e.g. "22,450.50").
    pub values: Prop<Vec<String>>,
    /// Desk name shown at the left.
    #[prop(default)]
    pub desk: Prop<Option<String>>,
    /// Currency shown at the right.
    #[prop(default)]
    pub currency: Prop<Option<String>>,
}

/// Top bar showing market pulses and desk context.
#[component]
pub fn HfMarketTopBar(props: HfMarketTopBarProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let labels = props.labels;
    let values = props.values;

    view()
        .flex_row()
        .gap(px(8.0))
        .px_4()
        .py_2()
        .bg(rgb(0x0f172a))
        .text_color(rgb(0xe2e8f0))
        .child(
            view()
                .flex_1()
                .child(text(move || {
                    props
                        .desk
                        .get_untracked()
                        .unwrap_or_else(|| "Desk".to_string())
                }))
                .text_size(px(12.0))
                .font_weight(FontWeight::MEDIUM),
        )
        .child(
            view().flex_1().justify_end().child(
                each(
                    labels,
                    move |label: &String| label.clone(),
                    move |label_signal, _| {
                        let value_signal = values.clone();
                        let label_val = label_signal.get();
                        let idx = labels
                            .get_untracked()
                            .iter()
                            .position(|x| x == &label_val)
                            .unwrap_or(0);
                        view()
                            .flex_row()
                            .gap(px(6.0))
                            .px_3()
                            .py_1()
                            .rounded(px(6.0))
                            .bg(rgb(0x334155))
                            .text_color(rgb(0xffffff))
                            .text_size(px(11.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(text(label_val))
                            .child(text(format!(
                                " {}",
                                value_signal.get().get(idx).cloned().unwrap_or_default()
                            )))
                    },
                )
                .into_element(),
            ),
        )
        .child(
            view()
                .flex_1()
                .justify_end()
                .child(text(move || {
                    props
                        .currency
                        .get_untracked()
                        .unwrap_or_else(|| "INR".to_string())
                }))
                .text_size(px(12.0))
                .font_weight(FontWeight::MEDIUM),
        )
}
