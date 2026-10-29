//! `HfRollingReturnsVisualizer` — rolling return consistency vs benchmark.

use nsh_core::element::NoEvent;
use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Height of the bar well, in px; each bar's height is its percentage of this.
const BAR_WELL_PX: f32 = 96.0;

/// Props for [`HfRollingReturnsVisualizer`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfRollingReturnsVisualizerProps {
    pub fund: Prop<f64>,
    pub benchmark: Prop<f64>,
    /// Extra Tailwind classes, applied after the visualizer's own
    /// (`Element::app_class`).
    #[prop(default)]
    pub class: Prop<Option<String>>,
}

/// The fund's rolling return as a bar (percent of the well's height, clamped
/// to 0–100) with its value, and the benchmark's return beneath it.
#[component]
pub fn HfRollingReturnsVisualizer(
    props: HfRollingReturnsVisualizerProps,
    cx: Scope<NoEvent>,
) -> impl IntoElement {
    let _ = cx;
    let fund = props.fund.get_untracked();
    let benchmark = props.benchmark.get_untracked();
    let extra = props.class.get_untracked();

    let classes = match extra {
        Some(c) => format!("hf-rolling-returns {c}"),
        None => "hf-rolling-returns".to_string(),
    };

    let bar_px = (fund.clamp(0.0, 100.0) as f32) / 100.0 * BAR_WELL_PX;

    view()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .rounded_lg()
        .border_soft()
        .surface_base()
        .p(px(12.0))
        .text_size(px(12.0))
        .app_class(Some(SharedString::from(classes)))
        .child(
            text("Rolling Returns")
                .text_muted()
                .uppercase()
                .text_size(px(10.0))
                .letter_spacing(px(0.8)),
        )
        .child(
            view()
                .flex()
                .items_end()
                .gap(px(4.0))
                .h(px(BAR_WELL_PX))
                .mt(px(8.0))
                .child(
                    view()
                        .flex_grow(1.0)
                        .h(px(bar_px))
                        .rounded_t(px(4.0))
                        .bg(rgb(0x10b981).opacity(0.2)),
                )
                .child(
                    view()
                        .w(px(1.0))
                        .h(px(bar_px))
                        .rounded_t(px(4.0))
                        .bg(rgb(0x10b981)),
                ),
        )
        .child(
            text(format!("{fund:.1}%"))
                .font_family("monospace")
                .text_high(),
        )
        .child(
            text(format!("vs benchmark {benchmark:.1}%"))
                .text_muted()
                .text_size(px(10.0)),
        )
}
