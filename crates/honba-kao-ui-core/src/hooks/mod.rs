//! Reusable hooks for honba-kao-ui-core components.

use leptos::html::ElementType;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// Clickaway handler — calls callback when clicking outside a reference element.
///
/// One `mousedown` listener is installed per mounted element; it is removed
/// before the effect reruns and when the calling owner is cleaned up.
///
/// ```rust
/// use leptos::prelude::*;
/// use honba_kao_ui_core::hooks::use_click_outside;
///
/// #[component]
/// fn Menu() -> impl IntoView {
///     let show = RwSignal::new(true);
///     let menu_ref = NodeRef::<leptos::html::Div>::new();
///     use_click_outside(menu_ref, move || show.set(false));
///     view! { <div node_ref=menu_ref>"Menu"</div> }
/// }
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
            let handle = window_event_listener(leptos::ev::mousedown, move |ev| {
                let target_el = ev
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::Element>().ok());
                if !el.contains(target_el.as_ref().map(|e| &**e)) {
                    (callback.borrow_mut())();
                }
            });
            on_cleanup(move || handle.remove());
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
///
/// The `keydown` listener is removed when the calling owner is cleaned up.
pub fn use_escape_key(
    callback: impl FnMut() + 'static,
) {
    let callback = std::cell::RefCell::new(callback);
    let handle = window_event_listener(leptos::ev::keydown, move |ev| {
        if ev.key() == "Escape" {
            (callback.borrow_mut())();
        }
    });
    on_cleanup(move || handle.remove());
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
            m.set(mql.matches());
            // The same closure that was added is the one removed on cleanup.
            let listener = StoredValue::new_local((mql, handler_closure));
            on_cleanup(move || {
                listener.with_value(|(mql, closure)| {
                    let _ = mql.remove_event_listener_with_callback(
                        "change",
                        closure.as_ref().unchecked_ref(),
                    );
                });
                listener.dispose();
            });
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
            // Detach the handler and close the socket when the owner is
            // cleaned up, before the closure itself is dropped.
            let socket = StoredValue::new_local((ws, onmessage_cb));
            on_cleanup(move || {
                socket.with_value(|(ws, _)| {
                    ws.set_onmessage(None);
                    let _ = ws.close();
                });
                socket.dispose();
            });
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
/// The rendered slice `[start, end)` of a fixed-row-height virtual list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VirtualRange {
    pub start: usize,
    pub end: usize,
}

impl VirtualRange {
    /// Rows intersecting `[scroll_top, scroll_top + viewport_height)` plus
    /// `overscan` rows on either side, clamped to `[0, total)`.
    ///
    /// A scroll offset past the end of the list (possible for one frame
    /// after the list shrinks, before the browser clamps `scrollTop`)
    /// yields the tail of the list rather than an empty window.
    pub fn compute(
        scroll_top: f64,
        viewport_height: f64,
        row_height: f64,
        total: usize,
        overscan: usize,
    ) -> Self {
        if total == 0 || row_height <= 0.0 {
            return Self::default();
        }
        let top = scroll_top.max(0.0);
        let first = (top / row_height).floor() as usize;
        let last = ((top + viewport_height.max(0.0)) / row_height).ceil() as usize;
        let end = last.saturating_add(overscan).min(total);
        let start = first
            .saturating_sub(overscan)
            .min(end.saturating_sub(last - first + overscan));
        Self { start, end }
    }
}

/// Viewport state of a virtualized list; see [`use_virtual_rows`].
#[derive(Clone, Copy)]
pub struct VirtualRows {
    /// The rows to render.
    pub range: Memo<VirtualRange>,
    container: NodeRef<leptos::html::Div>,
    scroll_top: RwSignal<f64>,
    viewport_height: RwSignal<f64>,
    total: Signal<usize>,
    row_height: f64,
}

impl VirtualRows {
    /// Re-read the container's scroll offset and height. Bind it to the
    /// container's `on:scroll`.
    pub fn sync(&self) {
        let Some(element) = self.container.get_untracked() else {
            return;
        };
        let top = f64::from(element.scroll_top());
        let height = f64::from(element.client_height());
        if self.scroll_top.get_untracked() != top {
            self.scroll_top.set(top);
        }
        if self.viewport_height.get_untracked() != height {
            self.viewport_height.set(height);
        }
    }

    /// Height in px of the unrendered rows above the window.
    pub fn space_before(&self) -> f64 {
        self.range.get().start as f64 * self.row_height
    }

    /// Height in px of the unrendered rows below the window.
    pub fn space_after(&self) -> f64 {
        let end = self.range.get().end;
        self.total.get().saturating_sub(end) as f64 * self.row_height
    }
}

/// Viewport virtualization for a scroll container holding `total` rows of
/// exactly `row_height` px each.
///
/// The caller binds [`VirtualRows::sync`] to the container's `on:scroll`
/// (tied to the element's lifetime, so nothing leaks), renders only
/// `range` and pads the skipped rows with [`VirtualRows::space_before`] /
/// [`VirtualRows::space_after`] so the scrollbar reflects the full list.
/// A `ResizeObserver` on the container keeps the viewport height current
/// (window resizes, sibling layout changes) and is disconnected on cleanup;
/// the offset is also re-read whenever `total` changes, since the browser
/// clamps `scrollTop` when the list shrinks.
pub fn use_virtual_rows(
    container: NodeRef<leptos::html::Div>,
    total: Signal<usize>,
    row_height: f64,
    overscan: usize,
) -> VirtualRows {
    // Until the container is measured, assume it is at most window-tall so
    // the first paint already fills the viewport.
    let initial_height = web_sys::window()
        .and_then(|window| window.inner_height().ok())
        .and_then(|height| height.as_f64())
        .unwrap_or(0.0);
    let scroll_top = RwSignal::new(0.0);
    let viewport_height = RwSignal::new(initial_height);
    let range = Memo::new(move |_| {
        VirtualRange::compute(
            scroll_top.get(),
            viewport_height.get(),
            row_height,
            total.get(),
            overscan,
        )
    });
    let rows = VirtualRows {
        range,
        container,
        scroll_top,
        viewport_height,
        total,
        row_height,
    };

    Effect::new(move |_| {
        total.track();
        rows.sync();
    });

    Effect::new(move |_| {
        let Some(element) = container.get() else {
            return;
        };
        let callback =
            wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || rows.sync());
        let Ok(observer) = web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()) else {
            return;
        };
        observer.observe(&element);
        let stored = StoredValue::new_local((observer, callback));
        on_cleanup(move || {
            stored.with_value(|(observer, _)| observer.disconnect());
            stored.dispose();
        });
    });

    rows
}

#[cfg(test)]
mod tests {
    use super::VirtualRange;

    fn range(scroll_top: f64, viewport: f64, total: usize) -> (usize, usize) {
        let r = VirtualRange::compute(scroll_top, viewport, 40.0, total, 5);
        (r.start, r.end)
    }

    #[test]
    fn empty_list_renders_nothing() {
        assert_eq!(range(0.0, 800.0, 0), (0, 0));
    }

    #[test]
    fn short_list_renders_everything() {
        assert_eq!(range(0.0, 800.0, 7), (0, 7));
    }

    #[test]
    fn top_of_list_clamps_overscan() {
        assert_eq!(range(0.0, 800.0, 250), (0, 25));
    }

    #[test]
    fn middle_of_list_adds_overscan_both_sides() {
        // Rows 50..=70 intersect [2010, 2810).
        assert_eq!(range(2010.0, 800.0, 250), (45, 76));
    }

    #[test]
    fn bottom_of_list_clamps_to_total() {
        // Max scroll for 250 rows in an 800px viewport.
        assert_eq!(range(9200.0, 800.0, 250), (225, 250));
    }

    #[test]
    fn stale_offset_past_end_yields_tail() {
        assert_eq!(range(9200.0, 800.0, 10), (0, 10));
        assert_eq!(range(9200.0, 800.0, 100), (75, 100));
    }
}
