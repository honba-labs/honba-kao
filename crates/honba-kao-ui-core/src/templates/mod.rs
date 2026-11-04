//! Tier 4: TEMPLATES — structural layout grids.

pub mod mosaic_workspace;
pub mod page;
pub mod three_columns;
pub mod two_columns;

pub use mosaic_workspace::HfMosaicWorkspace;
pub use page::{AlertSeverity, HfAlertBannerStack, HfPage};
pub use three_columns::HfThreeColumnTradingLayout;
pub use two_columns::HfTwoColumnSplitLayout;
