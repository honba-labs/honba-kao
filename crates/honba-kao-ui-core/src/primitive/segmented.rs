//! `HfSegmentSelector`, `HfSegmentedControl`, `HfTraderTabGroup` — bespoke
//! View/Text segmented controls (no `nsh_base::ToggleGroup` in this release).

use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Props for [`HfSegmentSelector`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfSegmentSelectorProps {
    /// The segment's value; also its displayed label.
    pub value: Prop<String>,
    #[prop(default = false)]
    pub selected: Prop<bool>,
}

/// A single segment / toggle item.
#[component]
pub fn HfSegmentSelector(props: HfSegmentSelectorProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let val = props.value.get_untracked();
    let sel = props.selected.get_untracked();

    if sel {
        view()
            .px(px(2.0))
            .py(px(1.0))
            .rounded(px(6.0))
            .bg(rgb(0x10b981))
            .child(
                text(val)
                    .text_color(rgb(0xffffff))
                    .font_weight(FontWeight::SEMIBOLD),
            )
    } else {
        view()
            .px(px(2.0))
            .py(px(1.0))
            .rounded(px(6.0))
            .surface_muted()
            .child(text(val).text_high().font_weight(FontWeight::MEDIUM))
    }
}

/// The events of a [`HfSegmentedControl`].
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfSegmentedControlEvent {
    /// A different segment was selected.
    #[args(value)]
    Change(String),
}

/// Props for [`HfSegmentedControl`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfSegmentedControlProps {
    pub value: Prop<String>,
    pub options: Prop<Vec<String>>,
}

/// A segmented control (toggle group of segments).
#[component]
pub fn HfSegmentedControl(
    props: HfSegmentedControlProps,
    cx: Scope<HfSegmentedControlEvent>,
) -> impl IntoElement {
    let opts = props.options.get_untracked();
    let cur = props.value.get_untracked();

    // `HfSegmentSelector` is `Scope<NoEvent>` (purely presentational), so
    // its own `ComponentElement` can never carry a press listener
    // (`Interactive::on_press` is implemented for `View`/`Text`/..., not
    // for `ComponentElement<C>`). Wrap each segment in a pressable `View`
    // instead and emit the control's own `Change` from there.
    view()
        .flex()
        .gap(px(1.0))
        .children(opts.into_iter().map(move |opt| {
            let is_sel = opt == cur;
            let segment_value = opt.clone();
            view()
                .on_press(move |_: &()| {
                    cx.emit(HfSegmentedControlEvent::Change(segment_value.clone()))
                })
                .child(HfSegmentSelector::element(HfSegmentSelectorProps {
                    value: Prop::from(opt),
                    selected: Prop::from(is_sel),
                }))
        }))
}

/// The events of a [`HfTraderTabGroup`].
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfTraderTabGroupEvent {
    /// A different tab was selected.
    #[args(value)]
    Change(String),
}

/// One tab of a [`HfTraderTabGroup`].
#[derive(Clone, Debug, Default, PartialEq, JsObject)]
pub struct HfTraderTab {
    pub label: String,
    pub key_badge: String,
}

/// Props for [`HfTraderTabGroup`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfTraderTabGroupProps {
    pub value: Prop<String>,
    pub tabs: Prop<Vec<HfTraderTab>>,
}

/// Trader tab group with function-key badges.
#[component]
pub fn HfTraderTabGroup(
    props: HfTraderTabGroupProps,
    cx: Scope<HfTraderTabGroupEvent>,
) -> impl IntoElement {
    let tabs = props.tabs.get_untracked();
    let cur = props.value.get_untracked();

    view()
        .flex()
        .gap(px(2.0))
        .children(tabs.into_iter().map(move |tab| {
            let is_sel = tab.label == cur;
            let tab_value = tab.label.clone();

            let tab_view = if is_sel {
                view()
                    .flex()
                    .items_center()
                    .gap(px(1.0))
                    .px(px(2.0))
                    .py(px(1.0))
                    .rounded(px(6.0))
                    .bg(rgb(0x10b981))
                    .child(
                        text(tab.label)
                            .text_color(rgb(0xffffff))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_size(px(10.0)),
                    )
                    .child(
                        text(tab.key_badge)
                            .text_color(rgb(0xffffff))
                            .text_size(px(8.0)),
                    )
            } else {
                view()
                    .flex()
                    .items_center()
                    .gap(px(1.0))
                    .px(px(2.0))
                    .py(px(1.0))
                    .rounded(px(6.0))
                    .surface_muted()
                    .child(
                        text(tab.label)
                            .text_high()
                            .font_weight(FontWeight::MEDIUM)
                            .text_size(px(10.0)),
                    )
                    .child(text(tab.key_badge).text_muted().text_size(px(8.0)))
            };

            tab_view
                .on_press(move |_: &()| cx.emit(HfTraderTabGroupEvent::Change(tab_value.clone())))
        }))
}
