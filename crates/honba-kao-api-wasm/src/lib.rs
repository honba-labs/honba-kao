//! `honba-kao-api-wasm` — Pure Math & Compute Engine.
//!
//! Owns pricing models, option Greeks, SPAN margin calculations, performance analytics,
//! technical indicators, execution fill pairing, screener predicate AST evaluation,
//! and simulation replay clock. Zero UI dependencies.

pub mod analytics;
pub mod indicators;
pub mod margin;
pub mod pairing;
pub mod pricing;
pub mod replay;
pub mod screener;

// Convenient top-level re-exports
pub use analytics::*;
pub use indicators::*;
pub use margin::*;
pub use pairing::*;
pub use pricing::*;
pub use replay::*;
pub use screener::*;
