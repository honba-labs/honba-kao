//! Presentational components used by more than one feature or by the app
//! shell: brand mark, layout container, the sign-in-required overlay, the
//! palette studio dialog and the [`charts`] family.

pub mod brand_logo;
pub mod charts;
pub mod command_bar;
pub mod command_palette;
pub mod container;
pub mod feature_placeholder;
pub mod login_required_overlay;
pub mod palette_studio;

pub use brand_logo::BrandLogo;
pub use command_bar::CommandBar;
pub use command_palette::CommandPalette;
pub use container::Container;
pub use feature_placeholder::FeaturePlaceholderPage;
pub use login_required_overlay::LoginRequiredOverlay;
pub use palette_studio::PaletteStudio;
