//! honba-kao-ui-core — Design System Primitives
//!
//! Mirrors `bfx-hf-ui-core` with `Hf` prefix. All components are accessible,
//! unstyled (token-driven), and compose via Leptos signals.
//!
//! **Icon Rule**: Outline SVG only, generated on-the-fly via `icons::outline`.
//! No static .svg files. Use `HfIcon` or `outline::icon_name()`.

pub mod primitive;
pub mod trading;
pub mod composed;
pub mod hooks;
pub mod theme;
pub mod icons;

// Re-export icons at top level for convenience
pub use icons::*;
// Re-export trading primitives at top level
pub use trading::*;
// Re-export composed components at top level
pub use composed::*;
// Re-export hooks at top level
pub use hooks::*;
// Re-export theme at top level
pub use theme::*;
// Re-export primitive components
pub use primitive::button::*;
pub use primitive::input::*;
pub use primitive::select::*;
pub use primitive::checkbox::*;
pub use primitive::radio::*;
pub use primitive::switch::*;
pub use primitive::label::*;
pub use primitive::separator::*;
pub use primitive::tabs::*;
pub use primitive::panel::*;
pub use primitive::page_header::*;
pub use primitive::placeholder::*;
pub use primitive::card::*;
pub use primitive::kpi::*;
pub use primitive::badge::*;
