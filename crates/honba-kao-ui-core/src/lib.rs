// NSH port (see nsh-port.md): `hooks` and `icons` are still Leptos and
// gated out until their own phase lands. `primitive`, `theme`, `molecules`
// are NSH-ported (Phase 1-2); `organisms`/`templates`/`app` land here
// (Phase 3-4, ported from the parallel `nsh-port-m3-m4` effort per the
// user's reconciliation decision — see HANDOFF.md).
pub mod app;
pub mod molecules;
pub mod organisms;
pub mod primitive;
pub mod templates;
pub mod theme;

// Top-level re-exports for convenience
pub use app::*;
pub use organisms::*;
pub use primitive::*;
pub use templates::*;
pub use theme::*;

// Canonical type for all financial values (prices, sizes, margin, collateral,
// haircuts, NAV). `honba-kao` enforces a Zero Floats Policy: money is
// represented with `rust_decimal::Decimal`, never `f64`/`f32`.
pub use rust_decimal::Decimal;
