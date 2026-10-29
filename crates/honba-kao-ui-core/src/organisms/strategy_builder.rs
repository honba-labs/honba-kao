//! Strategy builder organism — NSH port per nsh-port.md §4 Phase 3.
//!
//! A strategy builder: panes for legs and a canvas for the payoff chart.
//! Built from `nsh_core` primitives.

use nsh_core::prelude::*;

/// Props for [`HfStrategyBuilder`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfStrategyBuilderProps {
    #[prop(lazy)]
    pub children: Prop<Children>,
}

/// A strategy builder: panes for legs and a canvas for the payoff chart.
#[component]
pub fn HfStrategyBuilder(props: HfStrategyBuilderProps, _cx: Scope<NoEvent>) -> impl IntoElement {
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
                .flex_col()
                .gap(px(4.0))
                .child(text("Legs"))
                .child(props.children),
        )
        .child(
            view()
                .flex_1()
                .flex_col()
                .gap(px(4.0))
                .child(text("Payoff"))
                .child(canvas(move || Painter::default())),
        )
}
