//! Reusable reactive helpers. Mirrors `web/src/hooks/`.
//!
//! Leptos has no "hooks" rule — these are plain functions that create and
//! return reactive state, called during a component body.

pub mod use_interval;
pub mod use_system_config;

pub use use_interval::use_interval;
pub use use_system_config::use_system_config;
