//! Top-level screens. Mirrors `web/src/pages/`.
//!
//! Pages compose domain components from [`crate::components`]; anything that is
//! reused across pages lives under `components/<domain>/`.

pub mod beginner_onboarding_page;
pub mod data_page;
pub mod faq_page;
pub mod landing_page;
pub mod page_not_found;
pub mod portfolio_page;
pub mod settings_page;
pub mod strategy_studio_page;
pub mod trader_dashboard_page;
pub mod trading_planning_page;

pub use beginner_onboarding_page::BeginnerOnboardingPage;
pub use data_page::DataPage;
pub use faq_page::FaqPage;
pub use landing_page::LandingPage;
pub use page_not_found::PageNotFound;
pub use portfolio_page::PortfolioPage;
pub use settings_page::SettingsPage;
pub use strategy_studio_page::StrategyStudioPage;
pub use trader_dashboard_page::TraderDashboardPage;
pub use trading_planning_page::TradingPlanningPage;
