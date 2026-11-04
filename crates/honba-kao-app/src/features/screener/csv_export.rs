//! CSV export (screener-honba-leptos.md §6.4): pure row/header builders
//! testable on the host, plus the WASM Blob + object-URL anchor download.
//! Every cell is rendered through the shared `record_cell` formatters so the
//! export is byte-identical to what the grid shows.

use crate::features::screener::filters::record_cell;
use crate::features::screener::types::{ScreenerColumn, ScreenerRecord};

/// Header labels for the visible columns; the star toggle column never exports.
pub fn csv_headers(columns: &[ScreenerColumn]) -> Vec<String> {
    columns
        .iter()
        .filter(|col| col.key != "star")
        .map(|col| col.label.to_string())
        .collect()
}

/// Data rows for the visible columns; the star toggle column never exports.
pub fn csv_rows(records: &[ScreenerRecord], columns: &[ScreenerColumn]) -> Vec<Vec<String>> {
    let keys: Vec<&'static str> = columns
        .iter()
        .filter(|col| col.key != "star")
        .map(|col| col.key)
        .collect();
    records
        .iter()
        .map(|record| {
            keys.iter()
                .map(|&key| match key {
                    "symbol" => record.symbol().to_string(),
                    "name" => record.name().to_string(),
                    other => record_cell(record, other),
                })
                .collect::<Vec<String>>()
        })
        .collect()
}

/// CSV field escaping: quote fields containing comma, quote, CR or LF and
/// double any embedded quote. Plain fields pass through untouched.
pub fn csv_escape(field: &str) -> String {
    let needs_quoting =
        field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r');
    if !needs_quoting {
        return field.to_string();
    }
    format!("\"{}\"", field.replace('"', "\"\""))
}

/// Assembles the CSV document: header line, one line per row, LF endings,
/// trailing newline.
pub fn csv_document(headers: &[String], rows: &[Vec<String>]) -> String {
    let mut out = String::new();
    out.push_str(
        &headers
            .iter()
            .map(|h| csv_escape(h))
            .collect::<Vec<_>>()
            .join(","),
    );
    out.push('\n');
    for row in rows {
        out.push_str(
            &row.iter()
                .map(|r| csv_escape(r))
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push('\n');
    }
    out
}

/// Browser download: `text/csv` Blob -> object URL -> temporary anchor click
/// -> revoke (doc §6.4). WASM only; native builds have no download surface.
#[cfg(target_arch = "wasm32")]
pub fn export_screener_csv(filename: &str, document: &str) {
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Blob, BlobPropertyBag, HtmlElement, Url};

    let array = js_sys::Array::new();
    array.push(&JsValue::from_str(document));

    let mut bag = BlobPropertyBag::new();
    bag.set_type("text/csv;charset=utf-8;");
    if let Ok(blob) = Blob::new_with_str_sequence_and_options(&array, &bag) {
        if let Ok(url) = Url::create_object_url_with_blob(&blob) {
            if let Some(window) = web_sys::window() {
                if let Some(doc) = window.document() {
                    if let Ok(elem) = doc.create_element("a") {
                        let _ = elem.set_attribute("href", &url);
                        let _ = elem.set_attribute("download", filename);
                        if let Some(html_elem) = elem.dyn_ref::<HtmlElement>() {
                            html_elem.click();
                        }
                    }
                }
            }
            let _ = Url::revoke_object_url(&url);
        }
    }
}
