//! Composite intelligence panels for the Mission Control dashboard.
//!
//! Each panel models its OpenTerminalUI counterpart (plan-1.md §3 Milestone 2):
//! - [`AiInsightCard`] — on-demand AI market outlook (regime + flows briefing)
//! - [`IdeasRadar`] — algorithmic signal cards with trigger badges
//! - [`PortfolioMiniChart`] — equity curve vs benchmark mini chart
//! - [`IntelWire`] — breaking headlines with sentiment chips
//! - [`ResultsSummaryCards`] / [`IntelligenceTimeline`] — earnings scorecards
//!   and the corporate events / filings feed

pub mod ai_insight_card;
pub mod ideas_radar;
pub mod intel_wire;
pub mod portfolio_mini_chart;
pub mod results_summary;

pub use ai_insight_card::*;
pub use ideas_radar::*;
pub use intel_wire::*;
pub use portfolio_mini_chart::*;
pub use results_summary::*;
