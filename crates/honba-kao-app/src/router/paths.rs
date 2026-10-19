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
