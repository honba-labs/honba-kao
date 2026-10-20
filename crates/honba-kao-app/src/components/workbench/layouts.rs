//! Workspace grid presets, ported from the labs `lib/chart/layouts.tsx`.
//!
//! Shapes themselves are CSS (`wb-grid--*` and `wb-layout-icon--*` classes in
//! `style/workbench.css` use named grid areas `a`–`d`), so a preset only
//! carries its identity, label and cell count here.

use leptos::prelude::*;

use crate::stores::layout_store::WorkbenchLayout;

/// One workspace grid preset: how many panes it shows and what it is called.
pub struct LayoutPreset {
    pub layout: WorkbenchLayout,
    pub label: &'static str,
    pub cell_count: usize,
}

/// In display order, as in the labs layout picker.
pub const LAYOUT_PRESETS: [LayoutPreset; 5] = [
    LayoutPreset {
        layout: WorkbenchLayout::Single,
        label: "Single",
        cell_count: 1,
    },
    LayoutPreset {
        layout: WorkbenchLayout::Cols2,
        label: "2 Columns",
        cell_count: 2,
    },
    LayoutPreset {
        layout: WorkbenchLayout::Rows2,
        label: "2 Rows",
        cell_count: 2,
    },
    LayoutPreset {
        layout: WorkbenchLayout::OneTwo,
        label: "1 + 2 (Master & Dual)",
        cell_count: 3,
    },
    LayoutPreset {
        layout: WorkbenchLayout::Grid4,
        label: "2 × 2 Quad",
        cell_count: 4,
    },
];

pub fn preset_for(layout: WorkbenchLayout) -> &'static LayoutPreset {
    LAYOUT_PRESETS
        .iter()
        .find(|preset| preset.layout == layout)
        .expect("every WorkbenchLayout has a preset")
}

/// Named grid area for a pane index (`a`–`d`), matching the CSS templates.
pub fn area_name(index: usize) -> &'static str {
    ["a", "b", "c", "d"]
        .get(index)
        .copied()
        .expect("the workspace grid has at most 4 panes")
}

/// The 16×16 mini-grid glyph for a preset (≈ labs `LayoutIcon`).
///
/// Cells for areas the preset does not use are simply never rendered, so the
/// icon shape follows `cell_count` while the template class gives the pose.
#[component]
pub fn LayoutIcon(layout: WorkbenchLayout) -> impl IntoView {
    let cells = preset_for(layout).cell_count;

    view! {
        <span class=format!("wb-layout-icon wb-layout-icon--{}", layout.as_id()) aria-hidden="true">
            {(0..cells)
                .map(|index| {
                    let class = format!("wb-layout-icon__cell wb-cell--{}", area_name(index));
                    view! { <span class=class /> }
                })
                .collect_view()}
        </span>
    }
}
