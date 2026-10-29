//! `HfStatsBar` — horizontal strip of labelled stats, coloured by sign.

use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// One labelled value in [`HfStatsBar`]. `is_positive` colours the value:
/// `Some(true)` emerald, `Some(false)` rose, `None` neutral.
#[derive(Clone, Debug, Default, PartialEq, JsObject)]
pub struct StatItem {
    pub label: String,
    pub value: String,
    pub is_positive: Option<bool>,
}

/// Props for [`HfStatsBar`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfStatsBarProps {
    pub stats: Prop<Vec<StatItem>>,
}

/// Tailwind classes of a stat value, by its sign.
fn value_class(is_positive: Option<bool>) -> &'static str {
    match is_positive {
        Some(true) => "text-emerald-400 font-mono font-semibold",
        Some(false) => "text-rose-400 font-mono font-semibold",
        None => "text-text-high font-mono font-semibold",
    }
}

/// A horizontal strip of stats, one `label: value` pair per item.
#[component]
pub fn HfStatsBar(props: HfStatsBarProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let stats = props.stats;

    view()
        .flex_row()
        .items_center()
        .gap(px(16.0))
        .px(px(12.0))
        .py(px(6.0))
        .surface_muted()
        .border_soft()
        .text_size(px(12.0))
        .app_class(Some(SharedString::from(
            "hf-stats-bar border-b overflow-x-auto",
        )))
        .child(each(
            move || stats.get(),
            |item: &StatItem| item.label.clone(),
            move |item: ReadSignal<StatItem>, _index: ReadSignal<usize>| {
                view()
                    .flex_row()
                    .items_center()
                    .gap(px(6.0))
                    .app_class(Some(SharedString::from("shrink-0")))
                    .child(
                        text(move || format!("{}:", item.get().label))
                            .text_muted()
                            .text_size(px(12.0)),
                    )
                    .child(
                        view()
                            .app_class(move || {
                                Some(SharedString::from(value_class(item.get().is_positive)))
                            })
                            .child(text(move || item.get().value).text_size(px(12.0))),
                    )
            },
        ))
}
