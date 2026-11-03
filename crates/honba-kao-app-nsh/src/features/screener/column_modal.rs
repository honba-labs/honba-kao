//! Screener column-visibility modal (screener-design-doc.md §5.2).

use honba_kao_ui_core::molecules::quant::filter_pill::{
    HfFilterPill, HfFilterPillEvent, HfFilterPillProps,
};
use honba_kao_ui_core::primitive::base::button::{
    ButtonSize, ButtonVariant, HfButton, HfButtonEvent, HfButtonProps,
};
use honba_kao_ui_core::theme::tokens::{colors, radius, rgb_a};
use nsh_core::prelude::*;

use crate::features::screener::table_grid::default_visible_columns;
use crate::features::screener::types::{ScreenerColumn, ScreenerViewTab, tab_columns};

/// One list row: the column label plus a toggle chip whose pressed
/// state tracks membership in the visible-id set.
fn column_row(col: ScreenerColumn, visible: Signal<Vec<String>>) -> AnyElement {
    let col_id = col.id;
    view()
        .flex_row()
        .items_center()
        .justify_between()
        .py(px(6.0))
        .w_full()
        .child(text(col.label).text_size(px(12.0)).text_color(colors::TEXT))
        .child(
            HfFilterPill::element(HfFilterPillProps {
                label: Prop::from(col.label.to_string()),
                active: Prop::bind(move || visible.get().iter().any(|id| id == col_id)),
            })
            .on_event(move |event: &HfFilterPillEvent| {
                if *event == HfFilterPillEvent::Toggle {
                    visible.update(|ids| {
                        if ids.iter().any(|id| id == col_id) {
                            ids.retain(|s| s != col_id);
                        } else {
                            ids.push(col_id.to_string());
                        }
                    });
                }
            }),
        )
        .into_element()
}

/// Shared chrome for the footer's ghost button.
fn ghost_button_props() -> HfButtonProps {
    HfButtonProps {
        href: Prop::default(),
        size: Prop::from(Some(ButtonSize::Sm)),
        variant: Prop::from(Some(ButtonVariant::Ghost)),
        full_width: Prop::from(false),
        disabled: Prop::from(false),
        aria_label: Prop::from(None),
        class: Prop::default(),
        children: Children::default(),
    }
}

#[allow(non_snake_case)]
pub fn ScreenerColumnModal(
    tab: Signal<ScreenerViewTab>,
    visible: Signal<Vec<String>>,
    open: Signal<bool>,
) -> View {
    // The tab is fixed while the modal is presented; the rows are the
    // tab's column set snapshotted at construction.
    let columns = tab_columns(tab.get());

    let list = view()
        .flex_col()
        .gap(px(2.0))
        .w_full()
        .children(columns.iter().copied().map(|col| column_row(col, visible)));

    let footer = view()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .child(
            HfButton::element(ghost_button_props())
                .child(text("Reset to default"))
                .on_event(move |event: &HfButtonEvent| {
                    if *event == HfButtonEvent::Press {
                        visible.set(default_visible_columns(tab.get()));
                    }
                }),
        )
        .child(
            view()
                .px(px(14.0))
                .py(px(6.0))
                .rounded(px(radius::CONTROL))
                .bg(colors::ACCENT)
                .child(
                    text("Done")
                        .text_size(px(11.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors::ON_ACCENT),
                )
                .on_press(move || open.set(false)),
        );

    let panel = view()
        .flex_col()
        .w(px(420.0))
        .gap(px(radius::INSET))
        .px(px(radius::CARD))
        .py(px(radius::CARD))
        .rounded(px(radius::CARD))
        .bg(colors::SURFACE)
        .border_1()
        .border_color(colors::BORDER)
        .child(
            view()
                .flex_col()
                .gap(px(2.0))
                .w_full()
                .child(
                    text("Columns")
                        .text_size(px(14.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::TEXT),
                )
                .child(
                    text(move || {
                        format!(
                            "{} of {} shown for {}",
                            visible.get().len(),
                            tab_columns(tab.get()).len(),
                            tab.get().label()
                        )
                    })
                    .text_size(px(11.0))
                    .text_color(colors::MUTED),
                ),
        )
        .child(list)
        .child(footer);

    // Backdrop press closes (discarding nothing — the visible set is
    // mutated live); the panel sits in a press-swallowing wrapper.
    view()
        .w_full()
        .h_full()
        .flex_row()
        .items_center()
        .justify_center()
        .bg(rgb_a(0x0b0d12, 0.72))
        .on_press(move || open.set(false))
        .child(view().on_press(move || {}).child(panel))
}
