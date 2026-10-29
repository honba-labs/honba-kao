//! Tier 4: TEMPLATES — structural layout grids (NSH port).

pub mod mosaic_workspace;
pub mod page;
pub mod three_columns;
pub mod two_columns;

pub use mosaic_workspace::{HfMosaicWorkspace, HfMosaicWorkspaceProps};
pub use page::{HfPage, HfPageAlertSlot, HfPageAlertSlotProps, HfPageProps};
pub use three_columns::{HfThreeColumnTradingLayout, HfThreeColumnTradingLayoutProps};
pub use two_columns::{HfTwoColumnSplitLayout, HfTwoColumnSplitLayoutProps};
