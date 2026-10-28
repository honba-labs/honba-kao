//! `use_click_outside` — fires a callback when the user clicks outside a ref'd element.
//! Used by dropdowns, popovers and the Go-bar backdrop to dismiss on outside click.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlElement, Node};

/// Calls `on_click_outside` when a click lands outside the element referenced by `node_ref`.
pub fn use_click_outside(node_ref: NodeRef<leptos::html::Div>, on_click_outside: Callback<()>) {
    Effect::new(move |_| {
        let node_ref = node_ref;
        let on_click_outside = on_click_outside;
        if let Some(window) = web_sys::window() {
            let closure =
                Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |ev: web_sys::MouseEvent| {
                    let Some(target) = ev.target() else {
                        return;
                    };
                    let target_node: Option<Node> = target.dyn_into().ok();
                    let Some(element) = node_ref.get() else {
                        return;
                    };
                    let html_elem: &HtmlElement = &element;
                    let outside = target_node
                        .map(|node| !html_elem.contains(Some(&node)))
                        .unwrap_or(true);
                    if outside {
                        on_click_outside.run(());
                    }
                });
            let _ =
                window.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
            closure.forget();
        }
    });
}
