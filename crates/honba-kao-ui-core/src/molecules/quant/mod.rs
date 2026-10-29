//! Tier 2: MOLECULES — Quantitative & Terminal specific.
//!
//! NSH port (Phase 2, see `nsh-port.md` §4): still Leptos, not yet ported.
//! Uncomment each line as its file is converted.

pub mod agent_console;
pub mod artifact_card;
pub mod backtest_card;
pub mod cointegration_band;
pub mod filings_card;
pub mod filter_pill;
pub mod go_bar;
pub mod market_regime_badge;
pub mod underwater_gauge;

pub use agent_console::{AgentThoughtStep, HfAgentConsole};
pub use artifact_card::HfAiArtifactCard;
pub use backtest_card::HfBacktestMetricCard;
pub use cointegration_band::HfCointegrationBand;
pub use filings_card::HfFilingsCard;
pub use filter_pill::HfFilterPill;
pub use go_bar::HfGoBar;
pub use market_regime_badge::{HfMarketRegimeBadge, Regime};
pub use underwater_gauge::HfUnderwaterGauge;
