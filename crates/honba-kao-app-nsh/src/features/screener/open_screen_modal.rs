//! Open Screen modal (screener-design-doc.md §5.2): the `.` hotkey's
//! dialog — the built-in catalog grouped Popular / Fundamental /
//! Valuation / Technicals, the user's saved screens beneath, each row
//! applying on press (customs also deletable), and a Save sheet for the
//! Shift+N snapshot of the page's live state. Applies through the same
//! page-owned signals the filter modal writes (`ScreenerFilterModal`
//! pattern — no callback plumbing).

use nsh_core::prelude::*;

use honba_kao_ui_core::primitive::base::button::{
    ButtonSize, ButtonVariant, HfButton, HfButtonEvent, HfButtonProps,
};
use honba_kao_ui_core::primitive::base::input::{HfInput, HfInputEvent, HfInputProps};
use honba_kao_ui_core::theme::tokens::{HonbaStyle, colors, radius, rgb_a};

use crate::features::screener::screen_catalog::{
    CustomScreen, builtin_groups, delete_screen, load_custom_screens, snapshot_screen,
    upsert_screen,
};
use crate::features::screener::table_grid::default_visible_columns_for;
use crate::features::screener::types::{
    FilterCriteria, QuickPreset, ScreenerAssetType, ScreenerViewTab,
};

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

/// One catalog row: name + description, press applies the screen into
/// the page's signals; customs gain a delete chip that also refreshes
/// the working list.
#[allow(clippy::too_many_arguments)] // a row builder's flat signal list
fn screen_row(
    screen: CustomScreen,
    mine: bool,
    refresh: Signal<Vec<CustomScreen>>,
    open: Signal<bool>,
    asset: Signal<ScreenerAssetType>,
    tab: Signal<ScreenerViewTab>,
    preset: Signal<QuickPreset>,
    criteria: Signal<FilterCriteria>,
    visible_columns: Signal<Vec<String>>,
) -> AnyElement {
    let name = screen.name.clone();
    let description = screen.description.clone();
    let press_screen_name = screen.name.clone();

    let row = view()
        .flex_col()
        .gap(px(1.0))
        .px(px(8.0))
        .py(px(6.0))
        .rounded(px(radius::CONTROL))
        .bg(colors::INSET)
        // A press applies the whole screen into the page's signals and
        // dismisses the modal.
        .on_press(move || {
            let target = &press_screen_name;
            let _ = target;
            asset.set(screen.asset_type);
            tab.set(screen.tab);
            if let Some(arm) = screen.preset {
                preset.set(arm);
            }
            criteria.set(screen.criteria.clone());
            let defaults = default_visible_columns_for(screen.asset_type, screen.tab);
            let columns = if screen.visible_columns.is_empty() {
                defaults
            } else {
                screen.visible_columns.clone()
            };
            visible_columns.set(columns);
            open.set(false);
        })
        .child(
            text(name)
                .text_size(px(12.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_high(),
        )
        .child(text(description).text_size(px(10.0)).text_muted());

    if !mine {
        return row.into_element();
    }

    let delete_id = screen.id.clone();
    let row = row.child(
        view().mt(px(2.0)).child(
            HfButton::element(ghost_button_props())
                .child(text("Delete").text_color(colors::DOWN))
                .on_event(move |event: &HfButtonEvent| {
                    if *event == HfButtonEvent::Press {
                        delete_screen(&delete_id.clone());
                        refresh.set(load_custom_screens());
                    }
                }),
        ),
    );
    row.into_element()
}

/// The Open Screen modal panel: grouped catalog, user screens, and the
/// save sheet that snapshots the page's live state (Shift+N's flow).
#[allow(non_snake_case)]
pub fn ScreenerOpenScreenModal(
    open: Signal<bool>,
    save_open: Signal<bool>,
    asset: Signal<ScreenerAssetType>,
    tab: Signal<ScreenerViewTab>,
    preset: Signal<QuickPreset>,
    criteria: Signal<FilterCriteria>,
    visible_columns: Signal<Vec<String>>,
) -> View {
    // Customs snapshot once per presentation; deletes refresh the list.
    // The save sheet starts open when Shift+N summoned the modal.
    let customs = signal(load_custom_screens());
    let saving = signal(save_open.get_untracked());
    let name_draft = signal(String::new());
    let description_draft = signal(String::new());

    let catalog =
        view()
            .flex_col()
            .gap(px(10.0))
            .w_full()
            .children(builtin_groups().into_iter().map(|(group, screens)| {
                view()
                    .flex_col()
                    .gap(px(3.0))
                    .w_full()
                    .child(
                        text(group)
                            .text_size(px(10.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_muted(),
                    )
                    .child(
                        view()
                            .flex_col()
                            .gap(px(2.0))
                            .children(screens.into_iter().map(move |screen| {
                                screen_row(
                                    screen,
                                    false,
                                    customs,
                                    open,
                                    asset,
                                    tab,
                                    preset,
                                    criteria,
                                    visible_columns,
                                )
                            })),
                    )
            }));

    let customs_signal = customs;
    let custom_list =
        view()
            .flex_col()
            .gap(px(2.0))
            .w_full()
            .children(customs_signal.get().into_iter().map(move |screen| {
                screen_row(
                    screen,
                    true,
                    customs_signal,
                    open,
                    asset,
                    tab,
                    preset,
                    criteria,
                    visible_columns,
                )
            }));

    let save_sheet = view()
        .flex_col()
        .gap(px(6.0))
        .w_full()
        .child(
            HfButton::element(ghost_button_props())
                .child(text("▸ Save current view"))
                .on_event({
                    move |event: &HfButtonEvent| {
                        if *event == HfButtonEvent::Press {
                            saving.update(|is_open| *is_open = !*is_open);
                        }
                    }
                }),
        )
        .child(when(saving, move || {
            dynamic(move || {
                view()
                    .flex_col()
                    .gap(px(6.0))
                    .w_full()
                    .child(
                        HfInput::element(HfInputProps {
                            value: Prop::bind(move || name_draft.get()),
                            placeholder: Prop::from(Some("Screen name…".to_string())),
                            disabled: Prop::from(false),
                        })
                        .on_event({
                            let draft = name_draft;
                            move |event: &HfInputEvent| {
                                let HfInputEvent::Change(v) = event;
                                draft.set(v.clone());
                            }
                        }),
                    )
                    .child(
                        HfInput::element(HfInputProps {
                            value: Prop::bind(move || description_draft.get()),
                            placeholder: Prop::from(Some("What this screen isolates…".to_string())),
                            disabled: Prop::from(false),
                        })
                        .on_event({
                            let draft = description_draft;
                            move |event: &HfInputEvent| {
                                let HfInputEvent::Change(v) = event;
                                draft.set(v.clone());
                            }
                        }),
                    )
                    .child(
                        view()
                            .px(px(14.0))
                            .py(px(6.0))
                            .rounded(px(radius::CONTROL))
                            .bg(move || {
                                if name_draft.get().trim().is_empty() {
                                    colors::INSET
                                } else {
                                    colors::ACCENT
                                }
                            })
                            .on_press(move || {
                                let name = name_draft.get().trim().to_string();
                                if name.is_empty() {
                                    return;
                                }
                                let screen = snapshot_screen(
                                    &name,
                                    &description_draft.get(),
                                    asset.get(),
                                    tab.get(),
                                    Some(preset.get()),
                                    criteria.get(),
                                    visible_columns.get(),
                                );
                                upsert_screen(screen);
                                customs.set(load_custom_screens());
                                name_draft.set(String::new());
                                description_draft.set(String::new());
                                saving.set(false);
                            })
                            .child(
                                text("Save screen")
                                    .text_size(px(11.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(colors::ON_ACCENT),
                            ),
                    )
            })
        }));

    let panel = view()
        .flex_col()
        .w(px(520.0))
        .h(px(640.0))
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
                    text("Open Screen")
                        .text_size(px(14.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::TEXT),
                )
                .child(
                    text("Load a catalog screen or save the current view (Shift+N)")
                        .text_size(px(11.0))
                        .text_color(colors::MUTED),
                ),
        )
        .child(
            scroll()
                .flex_1()
                .flex_col()
                .gap(px(10.0))
                .child(catalog)
                .child(
                    view().flex_col().gap(px(3.0)).w_full().child(
                        text("Your screens")
                            .text_size(px(10.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_muted(),
                    ),
                )
                .child(custom_list),
        )
        .child(save_sheet);

    // Backdrop-press closes; the panel sits in a press-swallowing wrapper.
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
