//! `HfFilterPill` — multi-select factor filter pill for screeners.

use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// The events of [`HfFilterPill`].
#[derive(Debug, Clone, PartialEq, Event)]
pub enum HfFilterPillEvent {
    /// The pill was pressed. The caller flips `active` and passes the new
    /// value back down; the pill never changes its own state.
    Toggle,
}

/// Props for [`HfFilterPill`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfFilterPillProps {
    pub label: Prop<String>,
    #[prop(default = false)]
    pub active: Prop<bool>,
}

/// Pill-shaped toggle: emerald filled when active, muted outline when not.
#[component]
pub fn HfFilterPill(props: HfFilterPillProps, cx: Scope<HfFilterPillEvent>) -> impl IntoElement {
    // `active` is read once, as `HfChip` does: the caller re-renders the
    // pill with the new value after a `Toggle`.
    let active_val = props.active.get_untracked();
    let label = props.label;

    let el = view()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .px(px(10.0))
        .py(px(2.0))
        .rounded(px(6.0))
        .border(px(1.0))
        .app_class(Some(SharedString::from("hf-filter-pill cursor-pointer")));

    let el = if active_val {
        el.bg(AdaptiveColor::rgb(0x10b981, 0x10b981))
            .border_color(AdaptiveColor::rgb(0x10b981, 0x10b981))
            .text_color(AdaptiveColor::rgb(0xffffff, 0xffffff))
    } else {
        el.surface_muted()
            .border_color(AdaptiveColor::rgb(0x232a3a, 0x232a3a))
            .text_muted()
    };

    el.child(
        text(move || label.get())
            .text_size(px(12.0))
            .font_weight(FontWeight::MEDIUM),
    )
    .on_press(move |_: &()| cx.emit(HfFilterPillEvent::Toggle))
}
