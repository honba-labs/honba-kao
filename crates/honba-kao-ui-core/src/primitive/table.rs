//! HfTable — semantic table primitives (`<table>`, `<thead>`, `<tbody>`, `<tr>`, `<th>`, `<td>`, `<caption>`).
//!
//! Domain components must use these instead of raw `<table>` elements.

use leptos::prelude::*;

#[component]
pub fn HfTable(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <div class="hf-table__wrapper">
            <table class=format!("hf-table {class}").trim().to_string()>
                {children()}
            </table>
        </div>
    }
}

#[component]
pub fn HfTableHeader(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <thead class=format!("hf-table__header {class}").trim().to_string()>
            {children()}
        </thead>
    }
}

#[component]
pub fn HfTableBody(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <tbody class=format!("hf-table__body {class}").trim().to_string()>
            {children()}
        </tbody>
    }
}

#[component]
pub fn HfTableRow(
    #[prop(optional)] selected: Option<bool>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_click: Option<Callback<web_sys::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let selected = selected.unwrap_or(false);
    let row_class = format!(
        "hf-table__row {} {class}",
        if selected {
            "hf-table__row--selected"
        } else {
            ""
        }
    )
    .trim()
    .to_string();

    view! {
        <tr
            class=row_class
            aria-selected=selected.then_some("true")
            on:click=move |ev| {
                if let Some(cb) = on_click {
                    cb.run(ev);
                }
            }
        >
            {children()}
        </tr>
    }
}

#[component]
pub fn HfTableHeaderCell(
    #[prop(optional, into)] class: String,
    #[prop(optional)] colspan: Option<u32>,
    #[prop(optional)] on_click: Option<Callback<web_sys::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <th
            scope="col"
            colspan=colspan
            class=format!("hf-table__head {class}").trim().to_string()
            on:click=move |ev| {
                if let Some(cb) = on_click {
                    cb.run(ev);
                }
            }
        >
            {children()}
        </th>
    }
}

#[component]
pub fn HfTableCell(
    #[prop(optional, into)] class: String,
    #[prop(optional)] colspan: Option<u32>,
    children: Children,
) -> impl IntoView {
    view! {
        <td
            colspan=colspan
            class=format!("hf-table__cell {class}").trim().to_string()
        >
            {children()}
        </td>
    }
}

#[component]
pub fn HfTableCaption(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <caption class=format!("hf-table__caption {class}").trim().to_string()>
            {children()}
        </caption>
    }
}
