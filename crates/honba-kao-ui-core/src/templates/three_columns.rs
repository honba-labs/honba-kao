//! Three-column trading cockpit layout — NSH port per nsh-port.md §4 Phase 4.
//!
//! TopBar + Left Charts + Center Depth/Tape + Right OrderTicket +
//! Spanning Bottom Dock. Renders placeholder content; consumers replace
//! the inner views with real organisms.

use nsh_core::prelude::*;

/// Props for [`HfThreeColumnTradingLayout`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfThreeColumnTradingLayoutProps {
    #[prop(default)]
    pub market_tape: Prop<Option<String>>,
}

/// Three-column pro trading cockpit layout.
#[component]
pub fn HfThreeColumnTradingLayout(
    props: HfThreeColumnTradingLayoutProps,
    _cx: Scope<NoEvent>,
) -> impl IntoElement {
    view()
        .flex_col()
        .gap(px(2.0))
        .w_full()
        .h_full()
        .child(
            view()
                .flex_row()
                .w_full()
                .child(view().flex_col().gap(px(2.0)).w_full().child(text(move || {
                    props.market_tape.get_untracked().unwrap_or_default()
                }))),
        )
        .child(
            view()
                .flex_row()
                .gap(px(2.0))
                .flex_1()
                .child(
                    view()
                        .flex_1()
                        .flex_col()
                        .min_h(px(320.0))
                        .border(px(1.0))
                        .border_color(rgb(0x334155))
                        .child(text("Visualizer (Chart / Payoff)")),
                )
                .child(
                    view()
                        .flex_1()
                        .flex_col()
                        .min_h(px(320.0))
                        .border(px(1.0))
                        .border_color(rgb(0x334155))
                        .child(text("Depth Ladder (L2 Order Book)")),
                )
                .child(
                    view()
                        .flex_1()
                        .flex_col()
                        .min_h(px(320.0))
                        .border(px(1.0))
                        .border_color(rgb(0x334155))
                        .child(text("Order Ticket Panel")),
                ),
        )
        .child(
            view()
                .flex_col()
                .min_h(px(200.0))
                .w_full()
                .border(px(1.0))
                .border_color(rgb(0x334155))
                .child(text("Bottom Dock (Positions / Orders / History)")),
        )
}
