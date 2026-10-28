//! `use_escape_key` — calls a callback when the user presses Escape.
//! Essential for keyboard navigation across all modals, drawers, and the Go-bar.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

/// Dismisses modals, drawers, and the Go-bar on Escape.
pub fn use_escape_key(on_escape: Callback<()>) {
    Effect::new(move |_| {
        let on_escape = on_escape;
        if let Some(window) = web_sys::window() {
            let closure = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
                move |ev: web_sys::KeyboardEvent| {
                    if ev.key() == "Escape" {
                        on_escape.run(());
                    }
                },
            );
            let _ = window
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
            closure.forget();
        }
    });
}
