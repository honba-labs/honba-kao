//! §7 terminal shortcuts: one window keydown listener; typing targets (INPUT/TEXTAREA/SELECT) and IME composition are ignored.
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

/// Screener workstation actions (doc §7 table).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Dismiss,
}

/// Pure key → action mapping behind the window keydown listener: guard-free
/// of DOM details (only flags + focused-element tag), so the §7 table is
/// native-testable. Editable focus targets swallow every key except Escape
/// (modals and the drawer dismiss from anywhere).
pub fn map_key_event(
    key: &str,
    shift: bool,
    ctrl: bool,
    meta: bool,
    alt: bool,
    target_tag: &str,
) -> Option<ScreenerKeyAction> {
    if ctrl || meta || alt {
        return None;
    }
    if matches!(
        target_tag.to_uppercase().as_str(),
        "INPUT" | "TEXTAREA" | "SELECT"
    ) && key != "Escape"
    {
        return None;
    }
    match key {
        "j" | "ArrowDown" => Some(ScreenerKeyAction::NextRow),
        "k" | "ArrowUp" => Some(ScreenerKeyAction::PrevRow),
        " " => Some(ScreenerKeyAction::ToggleStar),
        "Enter" => Some(ScreenerKeyAction::EnterRoute),
        "." => Some(ScreenerKeyAction::OpenCatalog),
        "N" | "n" if shift => Some(ScreenerKeyAction::SaveScreen),
        "f" => Some(ScreenerKeyAction::OpenFilters),
        "c" => Some(ScreenerKeyAction::OpenColumns),
        "h" => Some(ScreenerKeyAction::ToggleView),
        "Escape" => Some(ScreenerKeyAction::Dismiss),
        _ => None,
    }
}

pub fn use_screener_shortcuts(handler: Callback<ScreenerKeyAction>) {
    Effect::new(move |_| {
        let handler = handler;
        let Some(window) = web_sys::window() else {
            return;
        };
        let closure =
            Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |ev: web_sys::KeyboardEvent| {
                if ev.is_composing() {
                    return;
                }
                let tag = ev
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                    .map(|el| el.tag_name())
                    .unwrap_or_default();
                let Some(action) = map_key_event(
                    &ev.key(),
                    ev.shift_key(),
                    ev.ctrl_key(),
                    ev.meta_key(),
                    ev.alt_key(),
                    &tag,
                ) else {
                    return;
                };
                ev.prevent_default();
                handler.run(action);
            });
        let _ =
            window.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
        closure.forget();
    });
}
