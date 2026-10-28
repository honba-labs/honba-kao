//! Accessible UI hooks for the design system.
//!
//! Ported patterns from the Secured-Finance atomic design system and OpenTerminalUI:
//! click-outside dismissal, Escape handling, responsive media queries, and row
//! virtualization for 1,000+ depth / position / order rows.

pub mod use_click_outside;
pub mod use_escape_key;
pub mod use_media_query;
pub mod use_virtual_rows;

pub use use_click_outside::use_click_outside;
pub use use_escape_key::use_escape_key;
pub use use_media_query::use_media_query;
pub use use_virtual_rows::use_virtual_rows;
