//! Trading-mode store — the operator's PAPER / LIVE book selector.
//!
//! One reactive store provided at the app root so every surface agrees on the
//! current mode: the header indicator (`HfTradingModeToggle`) writes it, the
//! order ticket / drawer / simulator read it. The choice persists in
//! `localStorage` and defaults to paper (orders never touch a real broker
//! unless the operator explicitly switches on live).

use leptos::prelude::*;

/// Which book orders execute against.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TradingMode {
    /// Simulated fills — the safe default.
    #[default]
    Paper,
    /// Live broker routing (requires an explicit operator switch).
    Live,
}

impl TradingMode {
    /// Short label shown in the header indicator.
    pub const fn label(self) -> &'static str {
        match self {
            TradingMode::Paper => "PAPER",
            TradingMode::Live => "LIVE",
        }
    }

    /// `localStorage` key value (stable wire spelling).
    const fn as_id(self) -> &'static str {
        match self {
            TradingMode::Paper => "paper",
            TradingMode::Live => "live",
        }
    }

    fn from_id(id: &str) -> Self {
        if id == "live" {
            TradingMode::Live
        } else {
            TradingMode::Paper
        }
    }
}

const STORAGE_KEY: &str = "honba.trading_mode";

/// The shared trading-mode state.
#[derive(Copy, Clone)]
pub struct TradingModeStore {
    mode: RwSignal<TradingMode>,
}

impl TradingModeStore {
    /// Restores the persisted choice, falling back to paper.
    pub fn new() -> Self {
        let restored = web_sys::window()
            .and_then(|window| {
                window
                    .local_storage()
                    .ok()
                    .flatten()
                    .and_then(|storage| storage.get_item(STORAGE_KEY).ok().flatten())
            })
            .map(|deref| TradingMode::from_id(&deref))
            .unwrap_or_default();

        Self {
            mode: RwSignal::new(restored),
        }
    }

    /// Read-only view of the current mode.
    pub fn mode(&self) -> Signal<TradingMode> {
        self.mode.into()
    }

    /// Reads *paper vs live* directly (e.g. for the Settings checkbox).
    pub fn is_paper(self) -> Signal<bool> {
        Signal::derive(move || self.mode.get() == TradingMode::Paper)
    }

    /// Switches the active book; writes through to `LocalStorage` and
    /// reacts anywhere the mode is read.
    pub fn set(&self, mode: TradingMode) {
        self.mode.set(mode);
        if let Some(storage) =
            web_sys::window().and_then(|window| window.local_storage().ok().flatten())
        {
            let _ = storage.set_item(STORAGE_KEY, mode.as_id());
        }
    }

    /// Convenience toggle used by keyboard shortcuts.
    pub fn toggle(&self) {
        let next = match self.mode.get() {
            TradingMode::Paper => TradingMode::Live,
            TradingMode::Live => TradingMode::Paper,
        };
        self.set(next);
    }
}

/// Access the app-provided trading-mode store.
pub fn use_trading_mode() -> TradingModeStore {
    expect_context::<TradingModeStore>()
}
