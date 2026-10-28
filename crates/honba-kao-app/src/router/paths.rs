//! Route paths and Page enumeration.

pub const ROUTE_HOME: &str = "/";
pub const ROUTE_LANDING: &str = "/landing";
pub const ROUTE_AUTH: &str = "/auth";
pub const ROUTE_ACCOUNT: &str = "/account";

// Trading Cockpit
pub const ROUTE_TRADING_COCKPIT: &str = "/trading";
pub const ROUTE_TRADING_MULTI_CHART: &str = "/trading/multi-chart";
pub const ROUTE_TRADING_DOM: &str = "/trading/dom";
pub const ROUTE_TRADING_TAPE: &str = "/trading/tape";
pub const ROUTE_TRADING_MTA: &str = "/trading/mta";
pub const ROUTE_TRADING_COMPARE: &str = "/trading/compare";
pub const ROUTE_TRADING_LAUNCHPAD: &str = "/trading/launchpad";

// Derivatives
pub const ROUTE_DERIVATIVES_OPTION_CHAIN: &str = "/derivatives/option-chain";
pub const ROUTE_DERIVATIVES_STRATEGY: &str = "/derivatives/strategy";
pub const ROUTE_DERIVATIVES_GREEKS: &str = "/derivatives/greeks";
pub const ROUTE_DERIVATIVES_OI: &str = "/derivatives/oi";

// Portfolio
pub const ROUTE_PORTFOLIO: &str = "/portfolio";
pub const ROUTE_PORTFOLIO_TRADEBOOK: &str = "/portfolio/tradebook";
pub const ROUTE_PORTFOLIO_SHADOW: &str = "/portfolio/shadow";
pub const ROUTE_PORTFOLIO_OPTIMIZER: &str = "/portfolio/optimizer";

// Wealth
pub const ROUTE_WEALTH_MUTUAL_FUNDS: &str = "/wealth/mutual-funds";
pub const ROUTE_WEALTH_SIP: &str = "/wealth/sip";
pub const ROUTE_WEALTH_ALLOCATION: &str = "/wealth/allocation";
pub const ROUTE_WEALTH_BONDS: &str = "/wealth/bonds";

// Quant
pub const ROUTE_QUANT_SIMULATOR: &str = "/quant/simulator";
pub const ROUTE_QUANT_BACKTESTING: &str = "/quant/backtesting";
pub const ROUTE_QUANT_MODEL_LAB: &str = "/quant/model-lab";
pub const ROUTE_QUANT_STAT_LAB: &str = "/quant/stat-lab";
pub const ROUTE_QUANT_PAIR_TRADING: &str = "/quant/pair-trading";
pub const ROUTE_QUANT_STRATEGY_STUDIO: &str = "/quant/strategy-studio";

// Screener
pub const ROUTE_SCREENER: &str = "/screener";

// Research
pub const ROUTE_RESEARCH_SECURITY: &str = "/research/security";
pub const ROUTE_RESEARCH_DOSSIERS: &str = "/research/dossiers";
pub const ROUTE_RESEARCH_TIMELINE: &str = "/research/timeline";
pub const ROUTE_RESEARCH_MARKETS: &str = "/research/markets";

// Risk & Ops
pub const ROUTE_RISK: &str = "/risk";
pub const ROUTE_OMS: &str = "/risk/oms";
pub const ROUTE_OPS: &str = "/risk/ops";
pub const ROUTE_POSITION_SIZER: &str = "/risk/sizer";
