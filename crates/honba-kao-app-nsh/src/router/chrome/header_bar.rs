//! The workstation's top header bar: 44 pt of breadcrumb, the active
//! cluster's sub-navigation pills, the search ("Ctrl+G") and AI analyst
//! ("Ctrl+J") triggers, the OMS/FEED LEDs, the latency sample and the
//! OS tag (`docs/workstation-chrome-instruction.md` Step 2). The
//! openers arrive as the chrome's signals through
//! [`super::app_shell::WorkspaceHotkeys`]; the LEDs render the
//! [`super::app_shell::ChromeStatus`] global.

use std::ptr;

use nsh_core::{
    element::{AccessibilityRole, AnyElement},
    prelude::*,
};

use honba_kao_ui_core::icons::canvas_icons::{IconName, icon};
use honba_kao_ui_core::templates::shell::{HfShellEntry, entry_group};
use honba_kao_ui_core::theme::tokens::{MONO, colors, radius};

use super::app_shell::{
    ChromeStatus, SUISSE, select_entry, status_row, workspace_hotkeys, workstation_entry,
    workstation_groups,
};

/// The bar: the breadcrumb and sub-navigation at the left, the overlay
/// triggers and the connection state at the right.
pub(crate) fn header_bar() -> AnyElement {
    view()
        .h(px(44.0))
        .flex_row()
        .items_center()
        .gap(px(12.0))
        .px_3()
        .bg(colors::SURFACE)
        .border_b_1()
        .border_b_color(colors::LINE)
        .child(breadcrumb())
        .child(view().flex_1().child(dynamic(sub_navigation_pills)))
        .child(gobar_button())
        .child(analyst_button())
        .child(connection())
        .child(platform_tag())
        .into_element()
}

/// The left breadcrumb: `HONBA / GROUP / Page` over the workstation's
/// open entry.
fn breadcrumb() -> AnyElement {
    let selected = workstation_entry();
    view()
        .flex_row()
        .items_baseline()
        .gap(px(6.0))
        .px_1()
        .child(
            text("HONBA")
                .text_size(px(13.0))
                .font_family(SUISSE)
                .font_weight(FontWeight::EXTRA_BOLD)
                .text_color(colors::TEXT),
        )
        .child(sep())
        .child(
            text(move || {
                let entry = selected.get().0;
                entry_group(entry, workstation_groups()).to_uppercase()
            })
            .text_size(px(12.0))
            .font_family(MONO)
            .font_weight(FontWeight::MEDIUM)
            .text_color(colors::MUTED),
        )
        .child(sep())
        .child(
            text(move || selected.get().0.name)
                .number_of_lines(1)
                .text_size(px(14.0))
                .font_family(SUISSE)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors::TEXT),
        )
        .into_element()
}

/// A breadcrumb separator.
fn sep() -> AnyElement {
    text("/")
        .text_size(px(12.0))
        .font_family(MONO)
        .text_color(colors::FAINT)
        .into_element()
}

/// The active cluster's pills: one per entry of the open entry's group;
/// a press opens the page.
fn sub_navigation_pills() -> AnyElement {
    let open = workstation_entry().get_untracked().0;
    let group = entry_group(open, workstation_groups());
    let pills: Vec<AnyElement> = workstation_groups()
        .iter()
        .find(|g| g.title == group)
        .map(|g| g.entries.iter().map(pill).collect())
        .unwrap_or_default();
    view()
        .flex_row()
        .items_center()
        .gap(px(4.0))
        .children(pills)
        .into_element()
}

/// One pill of the sub-navigation: raised fill + accent text while its
/// entry is the open one, a hover fill otherwise.
fn pill(entry: &'static HfShellEntry) -> AnyElement {
    let selected = workstation_entry();
    let active = move || selected.with(|s| ptr::eq(entry, s.0));
    view()
        .h(px(26.0))
        .flex_row()
        .items_center()
        .px(px(10.0))
        .rounded(px(radius::CONTROL))
        .cursor_pointer()
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
        .on_press(move || select_entry(entry))
        .child(
            text(entry.name)
                .text_size(px(12.0))
                .font_family(SUISSE)
                .style_when(active, |s| s.font_weight(FontWeight::SEMIBOLD))
                .text_color(move || {
                    if active() {
                        colors::ACCENT
                    } else {
                        colors::MUTED
                    }
                }),
        )
        .into_element()
}

/// The GoBar trigger ("Ctrl+G"): the overlay's opener signal lives in
/// the [`super::app_shell::WorkspaceHotkeys`] global.
fn gobar_button() -> AnyElement {
    let hotkeys = workspace_hotkeys();
    overlay_button(IconName::Search, "Jump to…", "Ctrl+G", move || {
        if let Some(open) = hotkeys.with_untracked(|h| h.gobar) {
            open.update(|v| *v = !*v);
        }
    })
}

/// The AI analyst trigger ("Ctrl+J").
fn analyst_button() -> AnyElement {
    let hotkeys = workspace_hotkeys();
    overlay_button(IconName::Spark, "Analyst", "Ctrl+J", move || {
        if let Some(open) = hotkeys.with_untracked(|h| h.agent) {
            open.update(|v| *v = !*v);
        }
    })
}

/// A 30 pt control: icon, label and a mono shortcut hint.
fn overlay_button(
    name: IconName,
    label: &'static str,
    hint: &'static str,
    toggle: impl Fn() + 'static,
) -> AnyElement {
    view()
        .h(px(30.0))
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .px(px(10.0))
        .rounded(px(radius::CONTROL))
        .bg(colors::INSET)
        .border_1()
        .border_color(colors::BORDER)
        .cursor_pointer()
        .hover(|s| s.bg(colors::LINE))
        .on_press(toggle)
        .role(AccessibilityRole::Button)
        .label(label)
        .child(icon().name(name).size(14.0).tint(colors::MUTED))
        .child(
            text(label)
                .text_size(px(12.0))
                .font_family(SUISSE)
                .text_color(colors::TEXT),
        )
        .child(
            text(hint)
                .text_size(px(10.0))
                .font_family(MONO)
                .text_color(colors::FAINT),
        )
        .into_element()
}

/// The right connection cluster: the OMS and FEED LEDs beside the last
/// latency sample, from the [`super::app_shell::ChromeStatus`] global.
fn connection() -> AnyElement {
    let status = global::<ChromeStatus>();
    view()
        .flex_row()
        .items_center()
        .gap(px(10.0))
        .pl(px(10.0))
        .child(status_row("OMS", |status| status.oms))
        .child(status_row("FEED", |status| status.feed))
        .child(
            text(move || match status.get().latency {
                Some(ms) => format!("{ms} ms"),
                None => "-- ms".to_string(),
            })
            .text_size(px(11.0))
            .font_family(MONO)
            .text_color(colors::MUTED),
        )
        .into_element()
}

/// The OS tag ("Workspace · Linux" and so on).
fn platform_tag() -> AnyElement {
    view()
        .h(px(22.0))
        .flex_row()
        .items_center()
        .px(px(8.0))
        .rounded(px(radius::CONTROL))
        .bg(colors::INSET)
        .child(
            text(format!("Workspace · {}", os().name()))
                .text_size(px(10.0))
                .font_family(MONO)
                .text_color(colors::FAINT),
        )
        .into_element()
}
