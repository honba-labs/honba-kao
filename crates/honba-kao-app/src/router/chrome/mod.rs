//! The persistent app chrome composed by [`crate::router::app_chrome`]: top
//! header bar, iconized side rail, footer and status bar. Mirrors
//! `web/src/components/common/`'s chrome pieces.

pub mod header_bar;
pub mod nav_rail;
pub mod site_footer;
pub mod status_bar;

pub use header_bar::HeaderBar;
pub use nav_rail::NavRail;
pub use site_footer::SiteFooter;
pub use status_bar::StatusBar;
