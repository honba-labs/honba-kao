//! Palette: draggable node entries from the catalogue of whitelisted
//! NSE indicators + `transform` family ops. Drag with HTML5 MIME
//! `application/honba-node` (port fe's).
//! 
//! Each entry shows the indicator name, a tiny glowing chip,
//! and a `data-honba-node` attribute so the canvas drag logic can
//! identify the kind.

use leptos::prelude::*;

/// Whitelisted indicator set (Phase 2): exactly 9 from `honba-me-indicators`.
/// Any other kind is *unverified-provisional* (badged, evaluated in Tier 1
/// only, cannot enter `goal_met` until server coverage exists).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Indicator {
    Sma,
    Ema,
    Rsi,
    Macd,
    Bollinger,
    Donchian,
    Attr,
    Stochastic,
    WilliamsR,
}

impl Indicator {
    fn token(&self) -> &'static str {
        match self {
            Indicator::Sma => "SMA",
            Indicator::Ema => "EMA",
            Indicator::Rsi => "RSI",
            Indicator::Macd => "MACD",
            Indicator::Bollinger => "BOLL",
            Indicator::Donchian => "DONX",
            Indicator::Attr => "ATR",
            Indicator::Stochastic => "STOCH",
            Indicator::WilliamsR => "WILLR",
        }
    }

    fn chip_class(&self) -> &'static str {
        "indicator-chip"
    }
}

/// One palette entry.
#[component]
pub fn IndicatorChip {
    /// The indicator to display.
    #[prop(into)]
    indicator: Indicator,
}

let ind = indicator;
let token = ind.token();

view! {
    <div
        class=format!("indicator-chip {}", ind.chip_class())
        data-honba-node=format!("{}", ind)
        style="margin: 2px; padding: 4px 8px; font-size: 12px; background: var(--c-indicator); color: var(--c-indicator-foreground);"
    >
        {ind.token()}
    </div>
}
