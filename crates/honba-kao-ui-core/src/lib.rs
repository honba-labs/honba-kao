//! honba-kao-ui-core — Design System Primitives
//!
//! Mirrors `bfx-hf-ui-core` with `Hf` prefix. All components are accessible,
//! unstyled (token-driven), and compose via Leptos signals.
//!
//! **Icon Rule**: Outline SVG only, generated on-the-fly via `icons::outline`.
//! No static .svg files. Use `HfIcon` or `outline::icon_name()`.

pub mod composed;
pub mod home;
pub mod hooks;
pub mod icons;
pub mod primitive;
pub mod theme;
pub mod trading;

// Re-export home primitives at top level
pub use home::*;
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
pub use primitive::accordion::*;
pub use primitive::alert::*;
pub use primitive::aspect_ratio::*;
pub use primitive::avatar::*;
pub use primitive::badge::*;
pub use primitive::breadcrumb::*;
pub use primitive::button::*;
pub use primitive::card::*;
pub use primitive::checkbox::*;
pub use primitive::collapsible::*;
pub use primitive::command::*;
pub use primitive::context_menu::*;
pub use primitive::data_table::*;
pub use primitive::dialog::*;
pub use primitive::drawer::*;
pub use primitive::form::*;
pub use primitive::hover_card::*;
pub use primitive::input::*;
pub use primitive::kpi::*;
pub use primitive::label::*;
pub use primitive::menu::*;
pub use primitive::navigation_menu::*;
pub use primitive::page_header::*;
pub use primitive::pagination::*;
pub use primitive::panel::*;
pub use primitive::placeholder::*;
pub use primitive::popover::*;
pub use primitive::progress::*;
pub use primitive::radio::*;
pub use primitive::resizable::*;
pub use primitive::scroll_area::*;
pub use primitive::scrollbar::*;
pub use primitive::segmented::*;
pub use primitive::select::*;
pub use primitive::separator::*;
pub use primitive::sheet::*;
pub use primitive::skeleton::*;
pub use primitive::slider::*;
pub use primitive::switch::*;
pub use primitive::table::*;
pub use primitive::tabs::*;
pub use primitive::toast::*;
pub use primitive::toggle::*;
pub use primitive::toolbar::*;
pub use primitive::tooltip::*;
