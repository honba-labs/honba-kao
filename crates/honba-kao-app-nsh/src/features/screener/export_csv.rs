//! CSV export delivery (screener-design-doc.md §5.7): the browser gets a
//! real file download — a `text/csv` [`Blob`], one object URL, one
//! invisible anchor click — while the native host keeps the clipboard
//! copy (there is no browser harness there). Non-secure contexts and
//! headless runs that reject the clipboard fail silently into this
//! download instead of vanishing.

/// Delivers `csv_content` to the user as `filename`:
/// WASM — download, native — clipboard fallback (logged, no-op).
pub fn deliver_csv(filename: &str, csv_content: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        trigger_csv_download(filename, csv_content);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use nsh_core::prelude::set_clipboard;
        tracing::info!(
            filename,
            bytes = csv_content.len(),
            "screener export CSV (native: clipboard only, no file harness)"
        );
        set_clipboard(csv_content);
    }
}

/// The browser download path: Blob → object URL → invisible `<a download>`
/// → click → revoke. Never panics: every step falls through to a log on
/// failure.
#[cfg(target_arch = "wasm32")]
pub fn trigger_csv_download(filename: &str, csv_content: &str) {
    use wasm_bindgen::JsCast as _;
    use wasm_bindgen::JsValue;

    let Some(window) = web_sys::window() else {
        tracing::warn!("screener export CSV: no window");
        return;
    };
    let Some(document) = window.document() else {
        tracing::warn!("screener export CSV: no document");
        return;
    };

    // 1. Create the text/csv Blob.
    let parts = js_sys::Array::of1(&JsValue::from_str(csv_content));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("text/csv");
    let blob = match web_sys::Blob::new_with_str_sequence_and_options(&parts.into(), &options) {
        Ok(blob) => blob,
        Err(_) => {
            tracing::warn!("screener export CSV: Blob contract declined");
            return;
        }
    };

    // 2. Generate the object URL.
    let object_url = match web_sys::Url::create_object_url_with_blob(&blob) {
        Ok(url) => url,
        Err(_) => {
            tracing::warn!("screener export CSV: object URL denied");
            return;
        }
    };

    // 3. The invisible anchor, download attribute, one programmatic
    //    click, then the URL's revoke.
    match document.create_element("a") {
        Ok(element) => {
            let anchor: web_sys::HtmlAnchorElement = element.unchecked_into();
            anchor.set_href(&object_url);
            anchor.set_download(filename);
            anchor.style().set_property("display", "none").ok();
            let attached = document
                .body()
                .and_then(|body| body.append_child(&anchor).ok())
                .is_some();
            anchor.click();
            let _ = web_sys::Url::revoke_object_url(&object_url);
            if attached {
                tracing::info!(
                    filename,
                    bytes = csv_content.len(),
                    "screener CSV downloaded"
                );
            }
        }
        Err(_) => {
            let _ = web_sys::Url::revoke_object_url(&object_url);
            tracing::warn!("screener export CSV: anchor creation refused");
        }
    }
}
