//! HfDataTable — virtualized table over `HfTable` / `HfTableHeader` / `HfTableBody`.
//!
//! Only the rows inside the viewport (plus overscan) are rendered; the rows
//! above and below are represented by spacer rows so the scrollbar reflects
//! the full list. Every row must be exactly `row_height` px tall.

use leptos::html::Div;
use leptos::prelude::*;

use crate::hooks::use_virtual_rows;
use crate::primitive::table::{HfTable, HfTableBody, HfTableHeader, HfTableHeaderCell, HfTableRow};

/// Virtualized data table. `render_row` turns one row's data into a
/// `HfTableRow` (with its `HfTableCell`s); `columns` are the header labels.
#[component]
pub fn HfDataTable<T>(
    /// Full row data (reactive).
    #[prop(into)]
    rows: Signal<Vec<T>>,
    /// Fixed row height in px.
    row_height: f64,
    /// Header labels, one per column.
    columns: Vec<&'static str>,
    /// Renders one row. Must return a `HfTableRow`.
    render_row: Callback<T, AnyView>,
    /// Extra rows rendered above and below the viewport.
    #[prop(optional)]
    overscan: Option<usize>,
    /// Extra CSS classes for the scroll container.
    #[prop(optional, into)]
    class: String,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
{
    let container = NodeRef::<Div>::new();
    let total = Signal::derive(move || rows.with(Vec::len));
    let virtual_rows = use_virtual_rows(container, total, row_height, overscan.unwrap_or(5));
    let column_count = columns.len().max(1);

    let window = move || {
        let range = virtual_rows.range.get();
        rows.with(|rows| {
            let end = range.end.min(rows.len());
            rows[range.start.min(end)..end].to_vec()
        })
    };

    let header_cells = columns
        .into_iter()
        .map(|label| view! { <HfTableHeaderCell>{label}</HfTableHeaderCell> })
        .collect_view();

    view! {
        <div
            node_ref=container
            class=format!("hf-data-table {class}").trim().to_string()
            on:scroll=move |_| virtual_rows.sync()
        >
            <HfTable>
                <HfTableHeader>
                    <HfTableRow>{header_cells}</HfTableRow>
                </HfTableHeader>
                <HfTableBody>
                    // Spacers keep the scrollbar height equal to the full list.
                    // Raw <tr>/<td>: the spacer needs an inline height, which the
                    // HfTableRow/HfTableCell primitives do not expose.
                    <tr
                        class="hf-data-table__spacer"
                        aria-hidden="true"
                        style=move || format!("height: {}px", virtual_rows.space_before())
                    >
                        <td colspan=column_count />
                    </tr>
                    {move || window().into_iter().map(|row| render_row.run(row)).collect_view()}
                    <tr
                        class="hf-data-table__spacer"
                        aria-hidden="true"
                        style=move || format!("height: {}px", virtual_rows.space_after())
                    >
                        <td colspan=column_count />
                    </tr>
                </HfTableBody>
            </HfTable>
        </div>
    }
}
