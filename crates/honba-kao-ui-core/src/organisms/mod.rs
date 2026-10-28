//! Tier 3: ORGANISMS — complex financial assemblies.

pub mod collateral_organism;
pub mod market_tape;
pub mod market_top_bar;
pub mod mutual_fund_dossier;
pub mod option_chain;
pub mod order_book;
pub mod order_ticket;
pub mod strategy_builder;
pub mod terminal_shell;
pub mod trading_dock;

pub use collateral_organism::{CollateralAssetRow, HfCollateralOrganism};
pub use market_tape::{HfMarketTape, TapeTrade};
pub use market_top_bar::HfMarketTopBar;
pub use mutual_fund_dossier::HfMutualFundDossier;
pub use option_chain::{HfOptionChain, OptionStrikePair};
pub use order_book::{BookLevel, HfOrderBook, OrderBookFilter};
pub use order_ticket::{HfOrderTicket, OrderIntent};
pub use strategy_builder::HfStrategyBuilder;
pub use terminal_shell::HfTerminalShell;
pub use trading_dock::HfTradingDock;
