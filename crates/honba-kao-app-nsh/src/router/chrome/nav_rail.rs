//! The workstation's left navigation rail: a 54 pt icon-only column of
//! the route clusters (`docs/workstation-chrome-instruction.md` Step 1,
//! replacing the pool's 264 pt demo sidebar). The active cluster is the
//! one holding the entry the workstation chrome (`super::app_shell`)
//! shows; a press opens the cluster's first page through
//! [`super::app_shell::select_entry`].

use std::ptr;

use nsh_core::{
    element::{AccessibilityRole, AccessibilityState, AnyElement},
    prelude::*,
};

use honba_kao_ui_core::icons::canvas_icons::{IconName, icon};
use honba_kao_ui_core::templates::shell::{HfShellEntry, HfShellGroup};
use honba_kao_ui_core::theme::tokens::{colors, radius};

use super::app_shell::{SUISSE, select_entry, workstation_entry, workstation_groups};

/// The rail's cluster icons, one per workstation group in order (the
/// spec's Step 1 mapping: System, Trading, Derivatives, Portfolio,
/// Wealth, Quant, Screener, Research, Risk & Ops).
const RAIL_ICONS: [IconName; 9] = [
    IconName::System,
    IconName::Stocks,
    IconName::Fno,
    IconName::Core,
    IconName::Wealth,
    IconName::Quant,
    IconName::Search,
    IconName::Data,
    IconName::Bell,
];

/// The rail: the amber brand mark, the cluster buttons, the account
/// button at the bottom.
pub(crate) fn nav_rail() -> AnyElement {
    let buttons: Vec<AnyElement> = workstation_groups()
        .iter()
        .zip(RAIL_ICONS)
        .map(|(group, cluster_icon)| rail_button(group, cluster_icon, &group.entries[0]))
        .collect();
    view()
        .w(px(54.0))
        .flex_col()
        .items_center()
        .bg(colors::INSET)
        .border_r_1()
        .border_r_color(colors::LINE)
        .child(view().pt(px(11.0)).child(brand_mark()))
        .child(
            view()
                .flex_1()
                .flex_col()
                .items_center()
                .pt(px(10.0))
                .gap(px(4.0))
                .children(buttons),
        )
        .child(view().pb(px(11.0)).child(account_button()))
        .into_element()
}

/// A cluster's rail button: a 40 pt icon tile with an accent pill on
/// the rail's left edge while the workstation shows one of its pages.
fn rail_button(
    group: &'static HfShellGroup,
    cluster_icon: IconName,
    default: &'static HfShellEntry,
) -> AnyElement {
    let selected = workstation_entry();
    let active = move || selected.with(|s| group.entries.iter().any(|entry| ptr::eq(entry, s.0)));
    view()
        .w(px(54.0))
        .h(px(40.0))
        .flex_row()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .role(AccessibilityRole::Button)
        .label(group.title)
        .accessibility_state(move || AccessibilityState {
            selected: active(),
            ..AccessibilityState::default()
        })
        .on_press(move || select_entry(default))
        .child(
            view()
                .absolute()
                .left_0()
                .top(px(11.0))
                .w(px(3.0))
                .h(px(18.0))
                .rounded(px(1.5))
                .bg(colors::ACCENT)
                .opacity(move || if active() { 1.0_f32 } else { 0.0 }),
        )
        .child(
            view()
                .size(px(40.0))
                .items_center()
                .justify_center()
                .rounded(px(radius::CONTROL))
                .style_when(active, |s| s.bg(colors::RAISED))
                .hover(|s| {
                    s.bg(move || {
                        if active() {
                            colors::RAISED
                        } else {
                            colors::LINE
                        }
                    })
                })
                .active(|s| s.bg(colors::RAISED))
                .child(icon().name(cluster_icon).size(18.0).tint(move || {
                    if active() {
                        colors::ACCENT
                    } else {
                        colors::MUTED
                    }
                })),
        )
        .into_element()
}

/// The amber "H" brand mark tile (the spec's Step 1 top slot).
fn brand_mark() -> AnyElement {
    view()
        .size(px(32.0))
        .rounded(px(8.0))
        .bg(colors::ACCENT_FILL)
        .items_center()
        .justify_center()
        .child(
            text("H")
                .text_size(px(19.0))
                .font_family(SUISSE)
                .font_weight(FontWeight::EXTRA_BOLD)
                .text_color(colors::ON_ACCENT),
        )
        .into_element()
}

/// The bottom user/account button: the workstation's Account entry
/// (the System group's).
fn account_button() -> AnyElement {
    let groups = workstation_groups();
    let account = groups[0]
        .entries
        .iter()
        .find(|entry| entry.name == "Account");
    let Some(account) = account else {
        return view().into_element();
    };
    let selected = workstation_entry();
    let active = move || selected.with(|s| ptr::eq(account, s.0));
    view()
        .w(px(54.0))
        .h(px(40.0))
        .flex_row()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .role(AccessibilityRole::Button)
        .label("Account")
        .accessibility_state(move || AccessibilityState {
            selected: active(),
            ..AccessibilityState::default()
        })
        .on_press(move || select_entry(account))
        .child(
            view()
                .size(px(40.0))
                .items_center()
                .justify_center()
                .rounded(px(radius::CONTROL))
                .style_when(active, |s| s.bg(colors::RAISED))
                .hover(|s| {
                    s.bg(move || {
                        if active() {
                            colors::RAISED
                        } else {
                            colors::LINE
                        }
                    })
                })
                .active(|s| s.bg(colors::RAISED))
                .child(icon().name(IconName::Storage).size(18.0).tint(move || {
                    if active() {
                        colors::ACCENT
                    } else {
                        colors::MUTED
                    }
                })),
        )
        .into_element()
}
