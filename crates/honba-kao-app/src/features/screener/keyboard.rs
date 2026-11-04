//! §7 terminal shortcuts: one window keydown listener; typing targets (INPUT/TEXTAREA/SELECT) and IME composition are ignored.
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

pub enum ScreenerKeyAction {
    NextRow,
    PrevRow,
    ToggleStar,
    EnterRoute,
    OpenCatalog,
    SaveScreen,
    OpenFilters,
    OpenColumns,
    ToggleView,
}

pub fn use_screener_shortcuts(handler: Callback<ScreenerKeyAction>) {
    Effect::new(move |_| {
        let handler = handler;
        let Some(window) = web_sys::window() else {
            return;
        };
        let closure =
            Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |ev: web_sys::KeyboardEvent| {
                if ev.ctrl_key() || ev.meta_key() || ev.alt_key() {
                    return;
                }
                if ev.is_composing() {
                    return;
                }
                let Some(target) = ev
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                else {
                    return;
                };
                match target.tag_name().to_uppercase().as_str() {
                    "INPUT" | "TEXTAREA" | "SELECT" => return,
                    _ => {}
                }
                let action = match ev.key().as_str() {
                    "j" | "ArrowDown" => Some(ScreenerKeyAction::NextRow),
                    "k" | "ArrowUp" => Some(ScreenerKeyAction::PrevRow),
                    " " => Some(ScreenerKeyAction::ToggleStar),
                    "Enter" => Some(ScreenerKeyAction::EnterRoute),
                    "." => Some(ScreenerKeyAction::OpenCatalog),
                    "N" | "n" if ev.shift_key() => Some(ScreenerKeyAction::SaveScreen),
                    "f" => Some(ScreenerKeyAction::OpenFilters),
                    "c" => Some(ScreenerKeyAction::OpenColumns),
                    "h" => Some(ScreenerKeyAction::ToggleView),
                    _ => None,
                };
                if let Some(action) = action {
                    ev.prevent_default();
                    handler.run(action);
                }
            });
        let _ =
            window.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
        closure.forget();
    });
}
