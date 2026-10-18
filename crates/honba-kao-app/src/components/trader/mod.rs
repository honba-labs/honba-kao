//! Trader-domain components. Mirrors `web/src/components/trader/`.

pub mod ai_traders_page;
pub mod competition_page;
pub mod decision_log;
pub mod position_history;
pub mod trader_launch_guest_page;
pub mod traders_list;

pub use ai_traders_page::AITradersPage;
pub use competition_page::CompetitionPage;
pub use decision_log::DecisionLog;
pub use position_history::PositionHistory;
pub use trader_launch_guest_page::TraderLaunchGuestPage;
pub use traders_list::TradersList;
