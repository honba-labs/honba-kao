//! `honba-kao-api-wasm` — Pure Math & Compute Engine.
//!
//! Owns pricing models, option Greeks, SPAN margin calculations, performance analytics,
//! technical indicators, execution fill pairing, screener predicate AST evaluation,
//! and simulation replay clock. Zero UI dependencies.

pub mod analytics;
pub mod indicators;
pub mod margin;
pub mod pairing;
pub mod payoff;
pub mod pricing;
pub mod replay;
pub mod screener;

// Convenient top-level re-exports
pub use analytics::*;
pub use indicators::*;
pub use margin::*;
pub use pairing::*;
pub use payoff::*;
pub use pricing::*;
pub use replay::*;
pub use screener::*;

// Web Worker JS bridge (only compiled with the `js` / wasm-bindgen feature).
#[cfg(feature = "js")]
pub mod bindings;

#[cfg(feature = "js")]
pub use bindings::*;
