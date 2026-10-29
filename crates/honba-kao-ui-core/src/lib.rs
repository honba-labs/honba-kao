// NSH port (see nsh-port.md): `hooks`, `icons`, `molecules`, `organisms` and
// `templates` are still Leptos and gated out until their own phase lands.
// `primitive` and `theme` are the only NSH-ported modules so far.
pub mod primitive;
pub mod theme;

// Top-level re-exports for convenience
pub use primitive::*;
pub use theme::*;

// Canonical type for all financial values (prices, sizes, margin, collateral,
// haircuts, NAV). `honba-kao` enforces a Zero Floats Policy: money is
// represented with `rust_decimal::Decimal`, never `f64`/`f32`.
pub use rust_decimal::Decimal;
