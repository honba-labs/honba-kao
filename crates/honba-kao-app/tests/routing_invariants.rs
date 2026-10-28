//! Routing invariant tests.
//!
//! Verifies the `ROUTE_*` path constants declared in
//! `crates/honba-kao-app/src/router/paths.rs` resolve to the expected URLs
//! and that every route constant is a well-formed absolute path. These are
//! pure-logic tests that run on the host (no WASM/DOM required).

use honba_kao_app::router::paths::*;

#[test]
fn route_constants_are_absolute_paths() {
    for route in [
        ROUTE_HOME,
        ROUTE_LANDING,
        ROUTE_AUTH,
        ROUTE_ACCOUNT,
        ROUTE_TRADING_COCKPIT,
        ROUTE_TRADING_MULTI_CHART,
        ROUTE_TRADING_DOM,
        ROUTE_TRADING_TAPE,
        ROUTE_TRADING_MTA,
        ROUTE_TRADING_COMPARE,
        ROUTE_TRADING_LAUNCHPAD,
        ROUTE_DERIVATIVES_OPTION_CHAIN,
        ROUTE_DERIVATIVES_STRATEGY,
        ROUTE_DERIVATIVES_GREEKS,
        ROUTE_DERIVATIVES_OI,
        ROUTE_PORTFOLIO,
        ROUTE_PORTFOLIO_TRADEBOOK,
        ROUTE_PORTFOLIO_SHADOW,
        ROUTE_PORTFOLIO_OPTIMIZER,
        ROUTE_WEALTH_MUTUAL_FUNDS,
        ROUTE_WEALTH_SIP,
        ROUTE_WEALTH_ALLOCATION,
        ROUTE_WEALTH_BONDS,
        ROUTE_QUANT_SIMULATOR,
        ROUTE_QUANT_BACKTESTING,
        ROUTE_QUANT_MODEL_LAB,
        ROUTE_QUANT_STAT_LAB,
        ROUTE_QUANT_PAIR_TRADING,
        ROUTE_QUANT_STRATEGY_STUDIO,
        ROUTE_SCREENER,
        ROUTE_RESEARCH_SECURITY,
        ROUTE_RESEARCH_DOSSIERS,
        ROUTE_RESEARCH_TIMELINE,
        ROUTE_RESEARCH_MARKETS,
        ROUTE_RISK,
        ROUTE_OMS,
        ROUTE_OPS,
        ROUTE_POSITION_SIZER,
    ] {
        assert!(
            route.starts_with('/'),
            "route constant must be an absolute path: {route:?}"
        );
        assert!(!route.is_empty(), "route constant must not be empty");
    }
}

#[test]
fn route_home_is_root() {
    assert_eq!(ROUTE_HOME, "/");
}

#[test]
fn route_trading_cockpit() {
    assert_eq!(ROUTE_TRADING_COCKPIT, "/trading");
}

#[test]
fn route_derivatives_option_chain() {
    assert_eq!(ROUTE_DERIVATIVES_OPTION_CHAIN, "/derivatives/option-chain");
}

#[test]
fn route_portfolio() {
    assert_eq!(ROUTE_PORTFOLIO, "/portfolio");
}

#[test]
fn route_wealth_sip() {
    assert_eq!(ROUTE_WEALTH_SIP, "/wealth/sip");
}

#[test]
fn route_quant_simulator() {
    assert_eq!(ROUTE_QUANT_SIMULATOR, "/quant/simulator");
}

#[test]
fn route_research_security() {
    assert_eq!(ROUTE_RESEARCH_SECURITY, "/research/security");
}

#[test]
fn route_risk_ops() {
    assert_eq!(ROUTE_OPS, "/risk/ops");
}

#[test]
fn route_position_sizer() {
    assert_eq!(ROUTE_POSITION_SIZER, "/risk/sizer");
}
