//! `use_media_query` — reactive boolean signal matching a CSS media query string.
//! Enables responsive layout swapping without SSR hydration mismatch.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

/// Returns an `RwSignal<bool>` indicating whether the media query currently matches.
pub fn use_media_query(query: &str) -> RwSignal<bool> {
    let matches = RwSignal::new(false);
    let query_owned = query.to_string();

    Effect::new(move |_| {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(mql)) = window.match_media(&query_owned) {
                matches.set(mql.matches());
                let mql_clone = mql.clone();
                let closure = Closure::<dyn FnMut()>::new(move || {
                    matches.set(mql_clone.matches());
                });
                let _ = mql
                    .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref());
                closure.forget();
            }
        }
    });

    matches
}
