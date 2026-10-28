//! `HfTable`, `HfTableHeader`, `HfTableRow`, `HfTableCell` — accessible table primitives.
//! Ported from `secured-finance-app` atoms `HorizontalListItem`, `TableHeader`, `TableContractCell`.

use leptos::prelude::*;

#[component]
pub fn HfTable(
    #[prop(into, optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let base = "hf-table w-full border-collapse text-left text-xs";
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    view! {
        <table class=classes>{children()}</table>
    }
}

#[component]
pub fn HfTableHeader(
    #[prop(into, optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let base = "hf-table-header";
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    view! {
        <thead class=classes>{children()}</thead>
    }
}

#[component]
pub fn HfTableRow(
    #[prop(into, optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let base = "hf-table-row border-b border-border-soft";
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    view! {
        <tr class=classes>{children()}</tr>
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TableCellAlign {
    Left,
    Center,
    Right,
    #[default]
    Inherit,
}

#[component]
pub fn HfTableCell(
    #[prop(into, optional)] class: Option<String>,
    #[prop(into, optional)] align: Option<TableCellAlign>,
    #[prop(into, optional)] numeric: Option<bool>,
    children: Children,
) -> impl IntoView {
    let numeric = numeric.unwrap_or(false);
    let align_class = match align.unwrap_or_default() {
        TableCellAlign::Left => "text-left",
        TableCellAlign::Center => "text-center",
        TableCellAlign::Right => "text-right",
        TableCellAlign::Inherit => "",
    };
    let numeric_class = if numeric { "font-mono tabular-nums" } else { "" };
    let base = "hf-table-cell px-3 py-1.5 border-b border-border-soft text-text-high";
    let classes = match class {
        Some(c) => format!("{} {} {} {}", base, align_class, numeric_class, c),
        None => format!("{} {} {}", base, align_class, numeric_class),
    };
    view! {
        <td class=classes>{children()}</td>
    }
}
