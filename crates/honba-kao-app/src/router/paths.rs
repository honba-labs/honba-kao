//! Single source of truth for route paths. Previously mirrored
//! `web/src/router/paths.ts`; the console now owns its own IA:
//!
//! ```text
//!   /portfolio   <- default landing after sign-in
//!   /screener    <- market view / screener
//!   /workbench   <- charting, buy/sell, analysis
//!   /strategy    <- AI-native strategy builder
//!   /simulator   <- strategy runner / backtester
//!   /tradebook   <- past analysis, session/month/year-wise
//!   /settings, /funds  <- user cluster
//! ```
//!
//! Everything that links to a route should reference these constants rather
//! than hard-coding strings, so a path rename is a one-line change.

/// Route path constants (the `ROUTES` object).
pub mod routes {
    pub const HOME: &str = "/";
    pub const LOGIN: &str = "/login";
    pub const REGISTER: &str = "/register";
    pub const RESET_PASSWORD: &str = "/reset-password";
    pub const SETUP: &str = "/setup";
    pub const PORTFOLIO: &str = "/portfolio";
    pub const SCREENER: &str = "/screener";
    /// Security Hub base; the page itself is `{SECURITY_HUB}/:ticker?tab=...`.
    pub const SECURITY_HUB: &str = "/equity/security";
    /// Research Dossiers (Phase 2.5) — mock-data page.
    pub const RESEARCH_DOSSIERS: &str = "/equity/research-dossiers";
    /// Derivatives & F&O (Phase 3) — Option Chain, mock-data page.
    pub const FNO: &str = "/fno";
    /// Multi-Leg Strategy Builder (Phase 3.3) — mock-data page.
    pub const FNO_STRATEGY: &str = "/fno/strategy";
    /// Derivatives analytics (Phase 3.4a) — mock-data pages.
    pub const FNO_OI: &str = "/fno/oi";
    pub const FNO_GREEKS: &str = "/fno/greeks";
    pub const FNO_FUTURES: &str = "/fno/futures";
    pub const FNO_FLOW: &str = "/fno/flow";
    pub const WORKBENCH: &str = "/workbench";
    pub const STRATEGY: &str = "/strategy";
    pub const SIMULATOR: &str = "/simulator";
    pub const TRADEBOOK: &str = "/tradebook";
    pub const SETTINGS: &str = "/settings";
    pub const FUNDS: &str = "/funds";
}

/// Alias so call sites read `ROUTES::PORTFOLIO`.
pub use routes as ROUTES;

/// The set of pages the header can highlight.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Page {
    /// Landing / no specific page (default for the header breadcrumb).
    Home,
    Portfolio,
    Screener,
    SecurityHub,
    Dossiers,
    OptionChain,
    StrategyBuilder,
    OiAnalysis,
    GreeksHeatmap,
    Futures,
    Flow,
    Workbench,
    Strategy,
    Simulator,
    Tradebook,
    Settings,
    Funds,
    Login,
    Register,
}

impl Page {
    /// Stable id used for `aria-current`-style highlighting and i18n keys.
    pub fn as_id(&self) -> &'static str {
        match self {
            Page::Home => "home",
            Page::Portfolio => "portfolio",
            Page::Screener => "screener",
            Page::SecurityHub => "security_hub",
            Page::Dossiers => "dossiers",
            Page::OptionChain => "option_chain",
            Page::StrategyBuilder => "strategy_builder",
            Page::OiAnalysis => "oi_analysis",
            Page::GreeksHeatmap => "greeks_heatmap",
            Page::Futures => "futures",
            Page::Flow => "flow",
            Page::Workbench => "workbench",
            Page::Strategy => "strategy",
            Page::Simulator => "simulator",
            Page::Tradebook => "tradebook",
            Page::Settings => "settings",
            Page::Funds => "funds",
            Page::Login => "login",
            Page::Register => "register",
        }
    }

    /// Human-readable label, used for breadcrumbs.
    pub fn label(&self) -> &'static str {
        match self {
            Page::Home => "Home",
            Page::Portfolio => "Portfolio",
            Page::Screener => "Screener",
            Page::SecurityHub => "Security Hub",
            Page::Dossiers => "Research Dossiers",
            Page::OptionChain => "Option Chain",
            Page::StrategyBuilder => "Strategy Builder",
            Page::OiAnalysis => "Open Interest",
            Page::GreeksHeatmap => "Greeks Heatmap",
            Page::Futures => "Futures",
            Page::Flow => "Options Flow",
            Page::Workbench => "Workbench",
            Page::Strategy => "Strategy",
            Page::Simulator => "Simulator",
            Page::Tradebook => "Tradebook",
            Page::Settings => "Settings",
            Page::Funds => "Funds",
            Page::Login => "Sign in",
            Page::Register => "Register",
        }
    }
}

/// Map a pathname back to a [`Page`] for header highlighting.
pub fn get_current_page_for_path(pathname: &str) -> Option<Page> {
    match pathname {
        r if r == ROUTES::HOME => Some(Page::Home),
        r if r == ROUTES::PORTFOLIO => Some(Page::Portfolio),
        r if r == ROUTES::SCREENER => Some(Page::Screener),
        r if r == ROUTES::WORKBENCH => Some(Page::Workbench),
        r if r == ROUTES::SECURITY_HUB || r.starts_with("/equity/security/") => {
            Some(Page::SecurityHub)
        }
        r if r == ROUTES::RESEARCH_DOSSIERS => Some(Page::Dossiers),
        r if r == ROUTES::FNO => Some(Page::OptionChain),
        r if r == ROUTES::FNO_STRATEGY => Some(Page::StrategyBuilder),
        r if r == ROUTES::FNO_OI => Some(Page::OiAnalysis),
        r if r == ROUTES::FNO_GREEKS => Some(Page::GreeksHeatmap),
        r if r == ROUTES::FNO_FUTURES => Some(Page::Futures),
        r if r == ROUTES::FNO_FLOW => Some(Page::Flow),
        r if r == ROUTES::STRATEGY => Some(Page::Strategy),
        r if r == ROUTES::SIMULATOR => Some(Page::Simulator),
        r if r == ROUTES::TRADEBOOK => Some(Page::Tradebook),
        r if r == ROUTES::SETTINGS => Some(Page::Settings),
        r if r == ROUTES::FUNDS => Some(Page::Funds),
        r if r == ROUTES::LOGIN => Some(Page::Login),
        r if r == ROUTES::REGISTER => Some(Page::Register),
        _ => None,
    }
}
