// NSH port (see nsh-port.md): `hooks`, `icons`, `organisms` and `templates`
// are still Leptos and gated out until their own phase lands. `primitive`
// and `theme` are NSH-ported; `molecules` is landing piecemeal (Phase 2) —
// each domain's `mod.rs` declares only the siblings already ported, so
// `cargo check` actually exercises what's been converted so far.
pub mod molecules;
pub mod primitive;
pub mod theme;

// Top-level re-exports for convenience
pub use primitive::*;
pub use theme::*;

// Canonical type for all financial values (prices, sizes, margin, collateral,
// haircuts, NAV). `honba-kao` enforces a Zero Floats Policy: money is
// represented with `rust_decimal::Decimal`, never `f64`/`f32`.
pub use rust_decimal::Decimal;
