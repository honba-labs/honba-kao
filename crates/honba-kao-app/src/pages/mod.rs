//! Top-level screens.
//!
//! The navigation-to-pagename mapping is:
//!
//! ```text
//!   nav rail            page module
//!   Portfolio           portfolio_page
//!   Screener            screener_page
//!   Workbench           workbench_page
//!   Strategy            strategy_studio_page
//!   Simulator           simulator_page
//!   Tradebook           tradebook_page
//!   User → Settings     settings_page
//!   User → Funds        funds_page
//! ```
//!
//! Pages compose domain components from [`crate::components`]; anything that
//! is reused across pages lives under `components/<domain>/`.

pub mod funds_page;
pub mod landing_page;
pub mod portfolio_page;
pub mod screener_page;
pub mod settings_page;
pub mod simulator_page;
pub mod strategy_studio_page;
pub mod tradebook_page;
pub mod workbench_page;

pub use funds_page::FundsPage;
pub use landing_page::LandingPage;
pub use portfolio_page::PortfolioPage;
pub use screener_page::ScreenerPage;
pub use settings_page::SettingsPage;
pub use simulator_page::SimulatorPage;
pub use strategy_studio_page::StrategyStudioPage;
pub use tradebook_page::TradebookPage;
pub use workbench_page::WorkbenchPage;
