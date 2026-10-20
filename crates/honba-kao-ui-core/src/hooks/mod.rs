//! Reusable hooks for honba-kao-ui-core components.

use leptos::html::ElementType;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// Clickaway handler — calls callback when clicking outside a reference element.
///
/// ```rust
/// use honba_kao_ui_core::hooks::use_click_outside;
///
/// let (show, set_show) = create_signal(false);
/// let menu_ref = NodeRef::<leptos::html::Div>::new();
///
/// use_click_outside(menu_ref, move |_| set_show(false));
/// ```
pub fn use_click_outside(
    node_ref: NodeRef<leptos::html::Div>,
    callback: impl FnMut() + 'static,
) {
    let callback = std::rc::Rc::new(std::cell::RefCell::new(callback));
    leptos::prelude::Effect::new(move |_| {
        if let Some(el) = node_ref.get() {
            let el: web_sys::Element = el.unchecked_into();
            let callback = callback.clone();
            let handler = move |ev: web_sys::MouseEvent| {
                let target_el = ev
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::Element>().ok());
                if !el.contains(target_el.as_ref().map(|e| &**e)) {
                    (callback.borrow_mut())();
                }
            };
            if let Some(window) = web_sys::window() {
                let _ = window.add_event_listener_with_callback(
                    "mousedown",
                    &wasm_bindgen::closure::Closure::wrap(Box::new(handler) as Box<dyn FnMut(web_sys::MouseEvent)>).into_js_value().unchecked_ref(),
                );
            }
        }
    });
}

/// Dismiss a popover menu when the pointer goes down outside `host`.
///
/// The listener is (re)installed while the menu is open and torn down by the
/// effect scope — Leptos' rough equivalent of the React `mousedown`
/// listeners labs attach in `useEffect`.
pub fn dismiss_on_outside_click<E>(open: RwSignal<bool>, host: NodeRef<E>)
where
    E: leptos::html::ElementType + 'static,
    <E as ElementType>::Output: JsCast + Clone + 'static,
{
    // Tracks only `host`, never `open`: a listener that reran (and dropped
    // its own wasm-bindgen Closure) every time `open` toggled could get torn
    // down mid-invocation by the very `open.set(false)` it had just called,
    // which wasm-bindgen reports as "closure invoked recursively or after
    // being dropped". One stable listener for the host's lifetime, checking
    // `open` fresh on each click, sidesteps that entirely.
    Effect::new(move |_| {
        let Some(element) = host.get() else {
            return;
        };
        // Cast to web_sys::Element for contains()
        let element: web_sys::Element = element.unchecked_into();
        let handle = window_event_listener_untyped("mousedown", move |event: web_sys::Event| {
            if !open.get_untracked() {
                return;
            }
            let target = event_target::<web_sys::Element>(&event);
            if !element.contains(Some(&target)) {
                open.set(false);
            }
        });
        on_cleanup(move || handle.remove());
    });
}

/// Escape key handler — calls callback when Escape is pressed.
pub fn use_escape_key(
    callback: impl FnMut() + 'static,
) {
    let callback = std::rc::Rc::new(std::cell::RefCell::new(callback));
    leptos::prelude::Effect::new(move |_| {
        if let Some(window) = web_sys::window() {
            let callback = callback.clone();
            let handler = move |ev: web_sys::KeyboardEvent| {
                if ev.key() == "Escape" {
                    (callback.borrow_mut())();
                }
            };
            let _ = window.add_event_listener_with_callback(
                "keydown",
                &wasm_bindgen::closure::Closure::wrap(Box::new(handler) as Box<dyn FnMut(web_sys::KeyboardEvent)>).into_js_value().unchecked_ref(),
            );
        }
    });
}

/// Focus trap — traps focus within a reference element when active.
pub fn use_focus_trap(
    node_ref: NodeRef<leptos::html::Div>,
    active: leptos::prelude::RwSignal<bool>,
) {
    leptos::prelude::Effect::new(move |_| {
        if active.get() {
            if let Some(el) = node_ref.get() {
                let _ = el.focus();
            }
        }
    });
}

/// Media query hook — returns true when the media query matches.
pub fn use_media_query(query: &'static str) -> leptos::prelude::RwSignal<bool> {
    let matches = leptos::prelude::RwSignal::new(false);
    if let Some(window) = web_sys::window() {
        if let Some(mql) = window.match_media(query).ok().flatten() {
            let m = matches.clone();
            let mql = std::rc::Rc::new(mql);
            let handler = {
                let m = m.clone();
                let mql = mql.clone();
                move |_: web_sys::Event| {
                    m.set(mql.matches());
                }
            };
            let handler_closure = wasm_bindgen::closure::Closure::wrap(Box::new(handler) as Box<dyn FnMut(web_sys::Event)>);
            let _ = mql.add_event_listener_with_callback("change", handler_closure.as_ref().unchecked_ref());
            handler_closure.forget();
            m.set(mql.matches());
        }
    }
    matches
}

/// Local storage hook — persists a signal value to localStorage.
pub fn use_local_storage<T: serde::Serialize + serde::de::DeserializeOwned + Clone + Send + Sync + 'static>(
    key: &'static str,
    default: T,
) -> leptos::prelude::RwSignal<T> {
    let value = leptos::prelude::RwSignal::new(default.clone());
    // Load from localStorage on init
    if let Some(window) = web_sys::window() {
        if let Some(storage) = window.local_storage().ok().flatten() {
            if let Ok(Some(saved)) = storage.get_item(key) {
                if let Ok(parsed) = serde_json::from_str::<T>(&saved) {
                    value.set(parsed);
                }
            }
        }
    }
    // Save to localStorage on change
    let key_owned = key.to_string();
    leptos::prelude::Effect::new(move |_| {
        let v = value.get();
        if let Ok(serialized) = serde_json::to_string(&v) {
            if let Some(window) = web_sys::window() {
                if let Some(storage) = window.local_storage().ok().flatten() {
                    let _ = storage.set_item(&key_owned, &serialized);
                }
            }
        }
    });
    value
}

/// Debounce hook — returns a debounced version of the input signal.
pub fn use_debounce<T: Clone + Send + Sync + 'static>(
    input: leptos::prelude::ReadSignal<T>,
    ms: u32,
) -> leptos::prelude::RwSignal<T> {
    let debounced = leptos::prelude::RwSignal::new(input.get());
    let pending = std::cell::RefCell::new(None::<i32>);

    leptos::prelude::Effect::new(move |_| {
        let val = input.get();
        let debounced = debounced.clone();
        let mut pending = pending.borrow_mut();
        if let Some(handle) = pending.take() {
            if let Some(window) = web_sys::window() {
                let _ = window.clear_timeout_with_handle(handle);
            }
        }
        if let Some(window) = web_sys::window() {
            let handle = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                &wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                    debounced.set(val.clone());
                }) as Box<dyn FnMut()>).into_js_value().unchecked_ref(),
                ms as i32,
            ).ok();
            *pending = handle;
        }
    });
    debounced
}

/// Controllable state hook — manages controlled vs uncontrolled state.
pub fn use_controllable_state<T: Clone + Send + Sync + 'static>(
    controlled: Option<leptos::prelude::ReadSignal<T>>,
    default: T,
) -> (leptos::prelude::RwSignal<T>, impl Fn(T)) {
    let internal = leptos::prelude::RwSignal::new(default.clone());
    let value = controlled.map(|c| c.get()).unwrap_or_else(|| default);
    let state = leptos::prelude::RwSignal::new(value);

    let set_value = move |new_val: T| {
        state.set(new_val.clone());
        internal.set(new_val);
    };

    leptos::prelude::Effect::new(move |_| {
        if let Some(c) = &controlled {
            state.set(c.get());
        }
    });

    (state, Box::new(set_value) as Box<dyn Fn(T)>)
}

/// WebSocket tick subscription hook.
pub fn use_tick_stream(
    url: &'static str,
    on_tick: impl Fn(serde_json::Value) + 'static,
) {
    let on_tick = std::rc::Rc::new(std::cell::RefCell::new(on_tick));
    leptos::prelude::Effect::new(move |_| {
        if let Ok(ws) = web_sys::WebSocket::new(url) {
            let on_tick_clone = on_tick.clone();
            let onmessage_cb = wasm_bindgen::closure::Closure::wrap(Box::new(move |ev: web_sys::MessageEvent| {
                if let Ok(text) = ev.data().dyn_into::<js_sys::JsString>() {
                    let text_str = text.as_string().unwrap_or_default();
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text_str) {
                        on_tick_clone.borrow()(val);
                    }
                }
            }) as Box<dyn FnMut(web_sys::MessageEvent)>);
            ws.set_onmessage(Some(onmessage_cb.as_ref().unchecked_ref()));
            onmessage_cb.forget();
        }
    });
}

/// NSE market phase computation hook.
pub fn use_market_phase() -> leptos::prelude::RwSignal<String> {
    let phase = leptos::prelude::RwSignal::new("Regular".to_string());
    // In a real implementation, this would compute based on NSE market hours
    phase
}

/// Trading state (HALTED/REDUCING/ACTIVE) sync hook.
pub fn use_trading_state() -> leptos::prelude::RwSignal<String> {
    let state = leptos::prelude::RwSignal::new("Active".to_string());
    state
}