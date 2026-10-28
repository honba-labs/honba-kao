//! Design-system theme: palette tokens and CSS class mappings.
//!
//! Ported from `/home/pmallapp/tmp/secured-finance-app` design tokens (Terminal Dark,
//! Emerald, Crimson, Amber) and the OpenTerminalUI terminal aesthetic.

pub mod palette;
pub mod tokens;

pub use palette::Palette;
pub use tokens::class;
pub use tokens::inject_css_variables;
