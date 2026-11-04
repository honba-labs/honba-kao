//! Canonical static assets for `honba-kao`.
//!
//! The single source of truth for every static file the Honba Kao UI
//! ships: typography ([`fonts`]), branding & exchange/broker emblems
//! ([`branding`]), institutional OMS audio cues ([`audio`]), theme
//! palettes ([`themes`]), and offline market-simulation fixtures
//! ([`fixtures`]). Every byte is compiled into the binary (`asset!` /
//! `include_bytes!` / `include_str!`), so consumers need no runtime
//! asset resolution.
//!
//! [`registry`] hands the whole `assets/` tree to the NSH engine so
//! `nsh run` and the native desktop runners serve the same files the
//! typed constants describe.

use nsh_core::element::{Registry, assets};

pub mod audio;
pub mod branding;
pub mod fixtures;
pub mod fonts;
pub mod themes;

/// Registers all assets with an NSH `Registry`.
///
/// In native desktop runners (`main.rs`), this allows `asset_directory()` to
/// copy the exact file assets needed by the platform runtime:
///
/// ```ignore
/// platform::set_ui_sets(vec![
///     nsh_base::registry(),
///     honba_kao_assets::registry(),
///     honba_kao_ui_core::registry(),
/// ]);
/// ```
pub fn registry() -> Registry {
    let mut reg = Registry::default();
    reg.assets(assets!());
    reg
}
