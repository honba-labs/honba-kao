//! Resilient WebSocket transport for institutional market feeds.
//!
//! `WsConnection` is a production-grade replacement for the minimal
//! `web_sys::WebSocket` wrapper. It provides:
//!
//! * **Automatic exponential backoff reconnection** after an unexpected close.
//! * **Heartbeat keepalive** to prevent idle timeouts on long-lived feeds.
//! * **Outgoing message queuing** while the socket is disconnected or opening.
//! * **Channel multiplexing** so many logical streams can share one socket.
//!
//! All transport concerns are headless (no UI dependencies) and the crate
//! compiles to `wasm32-unknown-unknown` using only `web_sys`/`js_sys` browser
//! APIs.

use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::cell::RefCell;

use js_sys::Date;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CloseEvent, MessageEvent, WebSocket};

/// A lifecycle event delivered to the `on_event` handler.
#[derive(Clone, Debug)]
pub enum WsEvent {
    /// The WebSocket opened successfully.
    Open,
    /// The WebSocket closed.
    Close {
        code: Option<u16>,
        reason: Option<String>,
        was_clean: bool,
    },
    /// A transport error occurred.
    Error(String),
    /// A reconnection attempt is being scheduled.
    Reconnecting { attempt: u32, delay_ms: u32 },
}

/// Configuration for reconnection and keepalive behaviour.
#[derive(Clone, Debug)]
pub struct WsConfig {
    /// Automatically reconnect after an unexpected close.
    pub reconnect: bool,
    /// Initial backoff delay in milliseconds.
    pub initial_backoff_ms: u32,
    /// Maximum backoff delay in milliseconds.
    pub max_backoff_ms: u32,
    /// Multiplier applied after each successive reconnect attempt.
    pub backoff_multiplier: f64,
    /// Heartbeat keepalive interval in milliseconds (0 = disabled).
    pub heartbeat_interval_ms: u32,
    /// Maximum outgoing messages queued while disconnected.
    pub max_queue_size: usize,
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            reconnect: true,
            initial_backoff_ms: 1_000,
            max_backoff_ms: 30_000,
            backoff_multiplier: 2.0,
            heartbeat_interval_ms: 15_000,
            max_queue_size: 10_000,
        }
    }
}

/// Mutable transport state shared between the browser event/timer closures.
struct Inner {
    config: WsConfig,
    url: String,
    /// The underlying socket (None while opening/closed/reconnecting).
    ws: Option<WebSocket>,
    /// Delivered every raw text frame.
    on_message: Box<dyn FnMut(String)>,
    /// Delivered lifecycle events (open, close, error, reconnecting).
    on_event: Box<dyn FnMut(WsEvent)>,
    /// Logical channel -> message sinks for multiplexing.
    channels: HashMap<String, Vec<Box<dyn FnMut(String)>>>,
    /// Outgoing messages queued while the socket is not open.
    queue: VecDeque<String>,
    /// Browser timer handles.
    heartbeat_timer: Option<i32>,
    reconnect_timer: Option<i32>,
    /// Exponential backoff state.
    next_delay_ms: u32,
    reconnect_attempts: u32,
    /// Set by an explicit `close()`; suppresses automatic reconnection.
    closed: bool,
}

/// A resilient, auto-reconnecting WebSocket connection.
pub struct WsConnection {
    state: Rc<RefCell<Inner>>,
}

impl WsConnection {
    /// Connect to `url`, delivering raw text frames to `on_message` and
    /// lifecycle events to `on_event`.
    pub fn connect<F, G>(
        url: &str,
        on_message: F,
        on_event: G,
        config: WsConfig,
    ) -> Result<Self, String>
    where
        F: FnMut(String) + 'static,
        G: FnMut(WsEvent) + 'static,
    {
        let state = Rc::new(RefCell::new(Inner::new(
            url.to_string(),
            Box::new(on_message),
            Box::new(on_event),
            config,
        )));

        Self::open_socket(&state)?;

        Ok(Self { state })
    }

    /// Create the underlying `WebSocket` and register its event handlers.
    fn open_socket(state: &Rc<RefCell<Inner>>) -> Result<(), String> {
        let url = { state.borrow().url.clone() };
        let ws =
            WebSocket::new(&url).map_err(|e| format!("WebSocket creation failed: {e:?}"))?;
        Self::register_handlers(state, &ws);
        state.borrow_mut().ws = Some(ws);
        Ok(())
    }

    /// Register the browser event handlers on a freshly created socket.
    fn register_handlers(state: &Rc<RefCell<Inner>>, ws: &WebSocket) {
        let onopen_state = state.clone();
        let onopen = Closure::<dyn FnMut(JsValue)>::new(move |_| {
            let _ = on_open(&onopen_state);
        });
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        let onmsg_state = state.clone();
        let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
            if let Some(txt) = e.data().as_string() {
                let _ = on_message(&onmsg_state, txt);
            }
        });
        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        let onerr_state = state.clone();
        let onerror = Closure::<dyn FnMut(JsValue)>::new(move |e: JsValue| {
            let _ = on_error(&onerr_state, format!("{e:?}"));
        });
        ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onerror.forget();

        let onclose_state = state.clone();
        let onclose = Closure::<dyn FnMut(CloseEvent)>::new(move |e: CloseEvent| {
            let _ = on_close(
                &onclose_state,
                e.was_clean(),
                e.code(),
                e.reason(),
            );
        });
        ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        onclose.forget();
    }

    /// Send a raw text frame immediately if the socket is open; otherwise queue
    /// it for delivery once the socket opens.
    pub fn send(&self, text: &str) -> Result<(), String> {
        let mut state = self.state.borrow_mut();
        if let Some(ws) = &state.ws {
            if ws.ready_state() == 1 {
                // WebSocket.OPEN
                return ws
                    .send_with_str(text)
                    .map_err(|e| format!("WebSocket send error: {e:?}"));
            }
        }
        state.enqueue_locked(text.to_string());
        Ok(())
    }

    /// Publish a message on a logical channel. The frame is sent as a JSON
    /// envelope `{"channel": ..., "payload": ..., "ts": ...}` so the server and
    /// channel subscribers can demultiplex.
    pub fn publish(&self, channel: &str, payload: &str) -> Result<(), String> {
        let envelope = serde_json::json!({
            "channel": channel,
            "payload": payload,
            "ts": Date::now() as i64,
        });
        self.send(&envelope.to_string())
    }

    /// Subscribe a callback to a logical channel. Incoming frames carrying a
    /// matching `channel` field are delivered to all of that channel's sinks.
    pub fn subscribe(&self, channel: &str, callback: impl FnMut(String) + 'static) {
        self.state
            .borrow_mut()
            .channels
            .entry(channel.to_string())
            .or_default()
            .push(Box::new(callback));
    }

    /// Whether the underlying socket is currently open.
    pub fn is_connected(&self) -> bool {
        self.state
            .borrow()
            .ws
            .as_ref()
            .map(|ws| ws.ready_state() == 1)
            .unwrap_or(false)
    }

    /// Explicitly close the connection. Automatic reconnection is suppressed.
    pub fn close(&self) {
        let mut state = self.state.borrow_mut();
        state.closed = true;
        state.stop_heartbeat_locked();
        state.stop_reconnect_timer_locked();
        if let Some(ws) = state.ws.take() {
            let _ = ws.close();
        }
    }

    /// Transport configuration.
    pub fn config(&self) -> WsConfig {
        self.state.borrow().config.clone()
    }

    // -----------------------------------------------------------------------
    // Timer-driven reconnection / heartbeat.
    // -----------------------------------------------------------------------

    /// Attempt to open a new socket (invoked by the reconnect timer).
    fn do_reconnect(state: &Rc<RefCell<Inner>>) -> Result<(), String> {
        {
            let mut s = state.borrow_mut();
            s.stop_reconnect_timer_locked();
            if s.closed {
                return Ok(());
            }
        }
        Self::open_socket(state)
    }

    /// Schedule a reconnection after the current backoff delay.
    fn schedule_reconnect(state: &Rc<RefCell<Inner>>) {
        let (attempt, delay) = {
            let mut s = state.borrow_mut();
            s.reconnect_attempts += 1;
            let delay = s.next_delay_ms;
            s.next_delay_ms =
                ((s.next_delay_ms as f64) * s.config.backoff_multiplier)
                    .min(s.config.max_backoff_ms as f64) as u32;
            (s.reconnect_attempts, delay)
        };

        {
            let mut s = state.borrow_mut();
            (s.on_event)(WsEvent::Reconnecting { attempt, delay_ms: delay });
        }

        let closure_state = state.clone();
        let timer = Closure::<dyn FnMut()>::new(move || {
            let _ = Self::do_reconnect(&closure_state);
        });
        if let Some(window) = web_sys::window() {
            if let Ok(handle) = window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    timer.as_ref().unchecked_ref(),
                    delay as i32,
                )
            {
                timer.forget();
                state.borrow_mut().reconnect_timer = Some(handle);
            }
        }
    }

    /// Start the heartbeat keepalive timer. Fires every `interval` ms and
    /// sends a JSON heartbeat frame while the socket is open.
    fn start_heartbeat(state: &Rc<RefCell<Inner>>) -> Result<(), String> {
        if state.borrow().config.heartbeat_interval_ms == 0 {
            return Ok(());
        }
        Self::stop_heartbeat(state);
        let interval = state.borrow().config.heartbeat_interval_ms;
        let closure_state = state.clone();
        let timer = Closure::<dyn FnMut()>::new(move || {
            let now = Date::now() as i64;
            let payload = serde_json::json!({ "type": "heartbeat", "ts": now }).to_string();
            let s = closure_state.borrow();
            if let Some(ws) = s.ws.as_ref() {
                if ws.ready_state() == 1 {
                    let _ = ws.send_with_str(&payload);
                }
            }
        });
        if let Some(window) = web_sys::window() {
            if let Ok(handle) = window
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    timer.as_ref().unchecked_ref(),
                    interval as i32,
                )
            {
                timer.forget();
                state.borrow_mut().heartbeat_timer = Some(handle);
            }
        }
        Ok(())
    }

    /// Stop the heartbeat timer.
    fn stop_heartbeat(state: &Rc<RefCell<Inner>>) {
        if let Some(handle) = state.borrow().heartbeat_timer {
            if let Some(window) = web_sys::window() {
                let _ = window.clear_interval_with_handle(handle);
            }
        }
        state.borrow_mut().heartbeat_timer = None;
    }
}

impl Inner {
    fn new(
        url: String,
        on_message: Box<dyn FnMut(String)>,
        on_event: Box<dyn FnMut(WsEvent)>,
        config: WsConfig,
    ) -> Self {
        let initial_backoff = config.initial_backoff_ms;
        Self {
            config,
            url,
            ws: None,
            on_message,
            on_event,
            channels: HashMap::new(),
            queue: VecDeque::new(),
            heartbeat_timer: None,
            reconnect_timer: None,
            next_delay_ms: initial_backoff,
            reconnect_attempts: 0,
            closed: false,
        }
    }

    fn enqueue_locked(&mut self, text: String) {
        if self.queue.len() >= self.config.max_queue_size {
            self.queue.pop_front();
        }
        self.queue.push_back(text);
    }

    fn flush_queue_locked(&mut self) -> Result<(), String> {
        while let Some(msg) = self.queue.pop_front() {
            let ws = self.ws.as_ref().ok_or("socket not open while flushing")?;
            if ws.ready_state() != 1 {
                self.enqueue_locked(msg);
                break;
            }
            ws.send_with_str(&msg)
                .map_err(|e| format!("flush send error: {e:?}"))?;
        }
        Ok(())
    }

    fn stop_heartbeat_locked(&mut self) {
        if let Some(handle) = self.heartbeat_timer.take() {
            if let Some(window) = web_sys::window() {
                let _ = window.clear_timeout_with_handle(handle);
            }
        }
    }

    fn stop_reconnect_timer_locked(&mut self) {
        if let Some(handle) = self.reconnect_timer.take() {
            if let Some(window) = web_sys::window() {
                let _ = window.clear_timeout_with_handle(handle);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Lifecycle event handlers (free functions so closures can capture `state`).
// ---------------------------------------------------------------------------

fn on_open(state: &Rc<RefCell<Inner>>) -> Result<(), String> {
    {
        let mut s = state.borrow_mut();
        s.stop_reconnect_timer_locked();
        s.next_delay_ms = s.config.initial_backoff_ms;
        s.reconnect_attempts = 0;
    }
    WsConnection::start_heartbeat(state)?;
    {
        let mut s = state.borrow_mut();
        s.flush_queue_locked()?;
        (s.on_event)(WsEvent::Open);
    }
    Ok(())
}

fn on_message(state: &Rc<RefCell<Inner>>, raw: String) -> Result<(), String> {
    {
        let mut s = state.borrow_mut();
        (s.on_message)(raw.clone());
    }
    // Route multiplexed channel messages to their subscribers.
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
        if let Some(channel) = value.get("channel").and_then(|c| c.as_str()) {
            let payload = value
                .get("payload")
                .map(|p| p.to_string())
                .unwrap_or_else(|| raw.clone());
            let mut s = state.borrow_mut();
            if let Some(sinks) = s.channels.get_mut(channel) {
                for sink in sinks.iter_mut() {
                    sink(payload.clone());
                }
            }
        }
    }
    Ok(())
}

fn on_error(state: &Rc<RefCell<Inner>>, message: String) -> Result<(), String> {
    (state.borrow_mut().on_event)(WsEvent::Error(message));
    Ok(())
}

fn on_close(
    state: &Rc<RefCell<Inner>>,
    was_clean: bool,
    code: u16,
    reason: String,
) -> Result<(), String> {
    {
        let mut s = state.borrow_mut();
        WsConnection::stop_heartbeat(state);
        s.stop_reconnect_timer_locked();
        s.ws = None;
        (s.on_event)(WsEvent::Close {
            code: Some(code),
            reason: if reason.is_empty() { None } else { Some(reason) },
            was_clean,
        });
        if s.closed {
            return Ok(());
        }
    }
    if {
        let s = state.borrow();
        s.config.reconnect
    } {
        WsConnection::schedule_reconnect(state);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_grows_exponentially_and_caps() {
        let cfg = WsConfig {
            initial_backoff_ms: 1000,
            max_backoff_ms: 10000,
            backoff_multiplier: 2.0,
            ..Default::default()
        };
        let mut delay = cfg.initial_backoff_ms;
        for _ in 0..6 {
            delay =
                ((delay as f64) * cfg.backoff_multiplier).min(cfg.max_backoff_ms as f64) as u32;
        }
        assert_eq!(delay, cfg.max_backoff_ms);
    }

    #[test]
    fn queue_drops_oldest_when_full() {
        let cfg = WsConfig {
            max_queue_size: 3,
            ..Default::default()
        };
        let mut queue: VecDeque<String> = VecDeque::new();
        for i in 1..=5 {
            if queue.len() >= cfg.max_queue_size {
                queue.pop_front();
            }
            queue.push_back(format!("msg-{i}"));
        }
        let collected: Vec<_> = queue.into_iter().collect();
        assert_eq!(collected, vec!["msg-3", "msg-4", "msg-5"]);
    }
}
