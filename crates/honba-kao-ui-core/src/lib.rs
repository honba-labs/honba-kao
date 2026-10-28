pub mod hooks;
pub mod icons;
pub mod molecules;
pub mod organisms;
pub mod primitive;
pub mod templates;
pub mod theme;

// Top-level re-exports for convenience
pub use hooks::*;
pub use icons::*;
pub use molecules::*;
pub use organisms::*;
pub use primitive::*;
pub use templates::*;
pub use theme::*;

// Canonical type for all financial values (prices, sizes, margin, collateral,
// haircuts, NAV). `honba-kao` enforces a Zero Floats Policy: money is
// represented with `rust_decimal::Decimal`, never `f64`/`f32`.
pub use rust_decimal::Decimal;
