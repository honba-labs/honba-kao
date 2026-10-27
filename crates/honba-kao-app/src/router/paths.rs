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
    /// Black-Scholes Option Greeks Calculator (Phase 3.4b) — live calculator.
    pub const OPTION_GREEKS: &str = "/equity/option-greeks";
    /// Chart Workstation (Phase 4.1) — multi-pane charts.
    pub const CHART_WORKSTATION: &str = "/equity/chart-workstation";
    /// Multi-Timeframe Analysis (Phase 4.3) — four panes, one symbol.
    pub const MTA: &str = "/equity/mta";
    /// DOM Depth Ladder (Phase 4.4) — mock-data page.
    pub const DOM: &str = "/equity/dom";
    /// Time & Sales print tape (Phase 4.4) — mock-data page.
    pub const TAPE: &str = "/equity/tape";
    /// Launchpad (Phase 4.5) — mosaic workspace, mock-data page.
    pub const LAUNCHPAD: &str = "/equity/launchpad";
    /// Launchpad pop-out window (Phase 4.5) — `?panel=<slug>`, outside the shell.
    pub const LAUNCHPAD_POPOUT: &str = "/equity/launchpad/popout";
    /// Backtesting Deck (Phase 5.1) — strategy templates, runner, and analytics.
    pub const BACKTESTING: &str = "/backtesting";
    /// Model Lab (Phase 5.4) — experiment registry and leaderboard.
    pub const MODEL_LAB: &str = "/backtesting/model-lab";
    /// Portfolio Lab & Optimizer (Phase 5.5) — strategy blending and optimization.
    pub const PORTFOLIO_LAB: &str = "/equity/portfolio/lab";
    pub const WORKBENCH: &str = "/workbench";
    pub const STRATEGY: &str = "/strategy";
    pub const SIMULATOR: &str = "/simulator";
    pub const TRADEBOOK: &str = "/tradebook";
    pub const SETTINGS: &str = "/settings";
    pub const FUNDS: &str = "/funds";
    /// Risk Dashboard (Phase 6.1) — VaR/CVaR, EWMA vol, PCA, stress scenarios.
    pub const RISK: &str = "/equity/risk";
    /// OMS & Pre-Trade Compliance (Phase 6.2) — order screening, throttle rates, circuit filters, restricted symbols, audit log.
    pub const OMS: &str = "/equity/oms";
    /// Operations Console (Phase 6.3) — multi-desk dashboard, kill switches, feed health.
    pub const OPS: &str = "/equity/ops";
    /// Position Sizer (Phase 6.4) — dynamic sizing with Fixed Fractional, Kelly, Volatility Target, Equal Risk Contribution, wired to honba-core::sizing.
    pub const POSITION_SIZER: &str = "/equity/position-sizer";
    /// Shadow Account Reconciler (Phase 6.5) — broker statement vs internal book reconciliation and violation reporting.
    pub const SHADOW_ACCOUNT: &str = "/equity/shadow-account";
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
    OptionGreeksCalc,
    ChartWorkstation,
    MultiTimeframe,
    Dom,
    Tape,
    Launchpad,
    Workbench,
    Strategy,
    Simulator,
    Tradebook,
    Backtesting,
    ModelLab,
    PortfolioLab,
    Settings,
    Funds,
    Risk,                      // Phase 6.1
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
            Page::OptionGreeksCalc => "option_greeks_calc",
            Page::ChartWorkstation => "chart_workstation",
            Page::MultiTimeframe => "multi_timeframe",
            Page::Dom => "dom",
            Page::Tape => "tape",
            Page::Launchpad => "launchpad",
            Page::Workbench => "workbench",
            Page::Strategy => "strategy",
            Page::Simulator => "simulator",
            Page::Backtesting => "backtesting",
            Page::ModelLab => "model_lab",
            Page::PortfolioLab => "portfolio_lab",
            Page::Risk => "risk",
            Page::OMS => "OMS",
            Page::OPS => "ops",
            Page::POSITION_SIZER => "position_sizer",
            Page::SHADOW_ACCOUNT => "shadow_account",
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
            Page::OptionGreeksCalc => "Option Greeks Calculator",
            Page::ChartWorkstation => "Chart Workstation",
            Page::MultiTimeframe => "Multi-Timeframe Analysis",
            Page::Dom => "DOM Depth Ladder",
            Page::Tape => "Time & Sales",
            Page::Launchpad => "Launchpad",
            Page::Workbench => "Workbench",
            Page::Strategy => "Strategy",
            Page::Simulator => "Simulator",
            Page::Backtesting => "Backtesting",
            Page::ModelLab => "Model Lab",
            Page::PortfolioLab => "Portfolio Lab",
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
        r if r == ROUTES::OPTION_GREEKS => Some(Page::OptionGreeksCalc),
        r if r == ROUTES::CHART_WORKSTATION => Some(Page::ChartWorkstation),
        r if r == ROUTES::MTA => Some(Page::MultiTimeframe),
        r if r == ROUTES::DOM => Some(Page::Dom),
        r if r == ROUTES::TAPE => Some(Page::Tape),
        r if r == ROUTES::LAUNCHPAD => Some(Page::Launchpad),
        r if r == ROUTES::STRATEGY => Some(Page::Strategy),
        r if r == ROUTES::SIMULATOR => Some(Page::Simulator),
        r if r == ROUTES::BACKTESTING => Some(Page::Backtesting),
        r if r == ROUTES::MODEL_LAB => Some(Page::ModelLab),
        r if r == ROUTES::PORTFOLIO_LAB || r == ROUTES::PORTFOLIO_OPTIMIZER => {
            Some(Page::PortfolioLab)
        }
        r if r == ROUTES::TRADEBOOK => Some(Page::Tradebook),
        r if r == ROUTES::SETTINGS => Some(Page::Settings),
        r if r == ROUTES::FUNDS => Some(Page::Funds),
        r if r == ROUTES::RISK => Some(Page::Risk),
        r if r == ROUTES::OMS => Some(Page::OMS),
        r if r == ROUTES::OPS => Some(Page::OPS),
        r if r == ROUTES::POSITION_SIZER => Some(Page::POSITION_SIZER),
        r if r == ROUTES::SHADOW_ACCOUNT => Some(Page::SHADOW_ACCOUNT),
        r if r == ROUTES::LOGIN => Some(Page::Login),
        r if r == ROUTES::REGISTER => Some(Page::Register),
        _ => None,
    }
}
