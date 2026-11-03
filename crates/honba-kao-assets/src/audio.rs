//! Institutional OMS audio cues: low-latency PCM/WAV sound bites for the
//! trading lifecycle (44.1 kHz, 16-bit PCM), compiled in via
//! `include_bytes!`.
//!
//! [`SoundCue`] is a strongly-typed event enum — order routing sites never
//! name a file, they dispatch a cue:
//!
//! ```ignore
//! match report.status {
//!     OrderStatus::Filled => SoundCue::OrderFilled.play(),
//!     OrderStatus::Rejected => SoundCue::OrderRejected.play(),
//!     OrderStatus::Cancelled => SoundCue::OrderCancelled.play(),
//!     _ => {}
//! }
//! ```

/// Strongly-typed sound events across the trading lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundCue {
    /// Dispatched immediately upon order transmit (user responsiveness confirmation).
    OrderSubmitted,
    /// Dispatched when an execution report indicates partial or full fill.
    OrderFilled,
    /// Dispatched when an order is cancelled by user or system.
    OrderCancelled,
    /// Dispatched when an order is rejected by exchange risk controls or broker RMS.
    OrderRejected,
    /// Urgent warning when margin usage breaches liquidation threshold.
    MarginWarning,
    /// Dispatched when a price trigger fires on a tracked instrument.
    PriceAlert,
    /// Dispatched when WebSocket/SSE stream reconnects.
    SystemConnected,
}

impl SoundCue {
    /// Returns the raw PCM WAV bytes included at compile time.
    pub fn wav_bytes(&self) -> &'static [u8] {
        match self {
            Self::OrderSubmitted => include_bytes!("../assets/audio/order_submitted.wav"),
            Self::OrderFilled => include_bytes!("../assets/audio/order_filled.wav"),
            Self::OrderCancelled => include_bytes!("../assets/audio/order_cancelled.wav"),
            Self::OrderRejected => include_bytes!("../assets/audio/order_rejected.wav"),
            Self::MarginWarning => include_bytes!("../assets/audio/margin_warning.wav"),
            Self::PriceAlert => include_bytes!("../assets/audio/price_alert.wav"),
            Self::SystemConnected => include_bytes!("../assets/audio/ping_connected.wav"),
        }
    }

    /// Plays the sound cue using the platform's native audio output (non-blocking).
    #[cfg(feature = "audio")]
    pub fn play(&self) {
        let bytes = self.wav_bytes();
        std::thread::spawn(move || {
            if let Ok((_stream, handle)) = rodio::OutputStream::try_default() {
                let cursor = std::io::Cursor::new(bytes);
                if let Ok(source) = rodio::Decoder::new(cursor) {
                    // `_sink` (not `let _ =`): the Sink must outlive the
                    // sleep below — dropping it cancels the queued voice.
                    let _sink = handle.play_raw(rodio::Source::convert_samples(source));
                    std::thread::sleep(std::time::Duration::from_millis(1500));
                }
            }
        });
    }

    #[cfg(not(feature = "audio"))]
    pub fn play(&self) {
        // No-op on native desktop without the `audio` feature. On wasm
        // targets this is forwarded to the browser Web Audio layer (the
        // §4.6 web asset layer) in a follow-up slice.
    }
}
