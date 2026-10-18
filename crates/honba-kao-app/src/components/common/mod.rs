//! App chrome shared across screens. Mirrors `web/src/components/common/`.

pub mod brand_logo;
pub mod container;
pub mod header_bar;
pub mod login_required_overlay;
pub mod site_footer;
pub mod status_bar;

pub use brand_logo::BrandLogo;
pub use container::Container;
pub use header_bar::HeaderBar;
pub use login_required_overlay::LoginRequiredOverlay;
pub use site_footer::SiteFooter;
pub use status_bar::StatusBar;
