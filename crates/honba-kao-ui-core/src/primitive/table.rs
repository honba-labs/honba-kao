//! `HfTable`, `HfTableCell`, `HfTableHeader`, `HfTableRow`, `TableCellAlign` —
//! table primitive built on [`View`] composition.
//!
//! `nsh_base::Table` / `DataTable` are available via `primitive/base/` when
//! full table behavior is needed; this is a lightweight display table.

use nsh_core::prelude::*;

use crate::theme::tokens::HonbaStyle;

/// Cell alignment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PropEnum)]
pub enum TableCellAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Props for [`HfTableRow`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfTableRowProps {
    pub cells: Prop<Vec<String>>,
    #[prop(default)]
    pub align: Prop<Option<TableCellAlign>>,
}

/// A table row.
#[component]
pub fn HfTableRow(props: HfTableRowProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let align = props.align.get_untracked();
    let cell_vals = props.cells.get_untracked();

    view()
        .flex()
        .gap(px(4.0))
        .border_b_1()
        .border_color(rgb(0x232a3a))
        .children(cell_vals.into_iter().map(move |content| {
            HfTableCell::element(HfTableCellProps {
                content: Prop::from(content),
                align: Prop::from(align),
            })
        }))
}

/// Props for [`HfTableCell`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfTableCellProps {
    pub content: Prop<String>,
    #[prop(default)]
    pub align: Prop<Option<TableCellAlign>>,
}

/// A table cell.
#[component]
pub fn HfTableCell(props: HfTableCellProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let align = props.align.get_untracked().unwrap_or_default();
    let c = props.content.get_untracked();

    let cell = view().flex().flex_1().px(px(2.0)).py(px(1.0));
    let cell = match align {
        TableCellAlign::Left => cell.justify_start(),
        TableCellAlign::Center => cell.justify_center(),
        TableCellAlign::Right => cell.justify_end(),
    };

    cell.child(text(c).text_high().text_size(px(12.0)))
}

/// Props for [`HfTableHeader`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfTableHeaderProps {
    pub label: Prop<String>,
}

/// A table header.
#[component]
pub fn HfTableHeader(props: HfTableHeaderProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let l = props.label.get_untracked();

    view().px(px(2.0)).py(px(1.0)).border_b_1().child(
        text(l)
            .text_high()
            .font_weight(FontWeight::SEMIBOLD)
            .text_size(px(11.0)),
    )
}

/// Props for [`HfTable`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct HfTableProps {
    pub rows: Prop<Vec<Vec<String>>>,
    #[prop(default)]
    pub header: Prop<Option<Vec<String>>>,
    /// Alignment applied to every data cell (via [`HfTableRow`] /
    /// [`HfTableCell`]); the header row is always left-aligned.
    #[prop(default)]
    pub align: Prop<Option<TableCellAlign>>,
}

/// A table, composed from [`HfTableHeader`] and [`HfTableRow`] (itself
/// composed from [`HfTableCell`]) so alignment flows from `HfTable` down
/// to each rendered cell instead of being dropped.
#[component]
pub fn HfTable(props: HfTableProps, cx: Scope<NoEvent>) -> impl IntoElement {
    let _ = cx;
    let rows = props.rows.get_untracked();
    let header = props.header.get_untracked();
    let align = props.align.get_untracked();

    let base = view().flex().flex_col();
    let base = if let Some(cols) = header {
        base.child(
            view()
                .flex()
                .gap(px(4.0))
                .children(cols.into_iter().map(|label| {
                    HfTableHeader::element(HfTableHeaderProps {
                        label: Prop::from(label),
                    })
                })),
        )
    } else {
        base
    };

    base.children(rows.into_iter().map(move |cells| {
        HfTableRow::element(HfTableRowProps {
            cells: Prop::from(cells),
            align: Prop::from(align),
        })
    }))
}
