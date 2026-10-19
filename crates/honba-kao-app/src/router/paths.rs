//! Single source of truth for route paths. Mirrors `web/src/router/paths.ts`.
//!
//! Everything that links to a route should reference these constants rather
//! than hard-coding strings, so a path rename is a one-line change.

/// Route path constants (the `ROUTES` object of the reference app).
pub mod routes {
    pub const HOME: &str = "/";
    pub const LOGIN: &str = "/login";
    pub const REGISTER: &str = "/register";
    pub const RESET_PASSWORD: &str = "/reset-password";
    pub const SETUP: &str = "/setup";
    pub const WELCOME: &str = "/welcome";
    pub const FAQ: &str = "/faq";
    pub const SETTINGS: &str = "/settings";
    pub const DATA: &str = "/data";
    pub const COMPETITION: &str = "/competition";
    pub const TRADERS: &str = "/traders";
    pub const DASHBOARD: &str = "/dashboard";
    pub const PORTFOLIO: &str = "/portfolio";
    pub const STRATEGY: &str = "/strategy";
    pub const STRATEGY_MARKET: &str = "/strategy-market";
    /// Trading planning ("frontier") — the reference app keeps both `/planning`
    /// (legacy) and `/frontier`, redirecting the former to the latter.
    pub const PLANNING: &str = "/frontier";
    pub const LEGACY_PLANNING: &str = "/planning";
}

/// Alias so call sites read `ROUTES::DASHBOARD`.
pub use routes as ROUTES;

/// The set of pages the header can highlight.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Page {
    /// Landing / no specific page (default for the header breadcrumb).
    Home,
    Planning,
    Competition,
    Traders,
    Trader,
    Portfolio,
    Strategy,
    StrategyMarket,
    Data,
    Faq,
    Login,
    Register,
}

impl Page {
    /// Stable id used for `aria-current`-style highlighting and i18n keys.
    pub fn as_id(&self) -> &'static str {
        match self {
            Page::Home => "home",
            Page::Planning => "planning",
            Page::Competition => "competition",
            Page::Traders => "traders",
            Page::Trader => "trader",
            Page::Portfolio => "portfolio",
            Page::Strategy => "strategy",
            Page::StrategyMarket => "strategy-market",
            Page::Data => "data",
            Page::Faq => "faq",
            Page::Login => "login",
            Page::Register => "register",
        }
    }

    /// Human-readable label, used for breadcrumbs.
    pub fn label(&self) -> &'static str {
        match self {
            Page::Home => "Home",
            Page::Planning => "Planning",
            Page::Competition => "Competition",
            Page::Traders => "Traders",
            Page::Trader => "Dashboard",
            Page::Portfolio => "Portfolio",
            Page::Strategy => "Strategy",
            Page::StrategyMarket => "Strategy market",
            Page::Data => "Data",
            Page::Faq => "FAQ",
            Page::Login => "Sign in",
            Page::Register => "Register",
        }
    }
}

/// Map a pathname back to a [`Page`] for header highlighting.
pub fn get_current_page_for_path(pathname: &str) -> Option<Page> {
    match pathname {
        r if r == ROUTES::HOME => Some(Page::Home),
        r if r == ROUTES::PLANNING || r == ROUTES::LEGACY_PLANNING => Some(Page::Planning),
        r if r == ROUTES::WELCOME || r == ROUTES::TRADERS => Some(Page::Traders),
        r if r == ROUTES::DASHBOARD => Some(Page::Trader),
        r if r == ROUTES::PORTFOLIO => Some(Page::Portfolio),
        r if r == ROUTES::STRATEGY => Some(Page::Strategy),
        r if r == ROUTES::STRATEGY_MARKET => Some(Page::StrategyMarket),
        r if r == ROUTES::DATA => Some(Page::Data),
        r if r == ROUTES::FAQ => Some(Page::Faq),
        r if r == ROUTES::LOGIN => Some(Page::Login),
        r if r == ROUTES::REGISTER => Some(Page::Register),
        r if r == ROUTES::COMPETITION => Some(Page::Competition),
        _ => None,
    }
}

/// Build a dashboard URL, optionally deep-linking a trader by slug.
pub fn build_dashboard_path(trader_slug: Option<&str>) -> String {
    match trader_slug {
        None => ROUTES::DASHBOARD.to_string(),
        Some(slug) => format!("{}?trader={}", ROUTES::DASHBOARD, slug),
    }
}

/// Legacy hash routes retained for deep links (`#traders` etc.).
pub fn legacy_hash_route(hash: &str) -> Option<&'static str> {
    match hash {
        "competition" => Some(ROUTES::COMPETITION),
        "traders" => Some(ROUTES::TRADERS),
        "trader" | "details" => Some(ROUTES::DASHBOARD),
        "strategy" => Some(ROUTES::STRATEGY),
        "strategy-market" => Some(ROUTES::STRATEGY_MARKET),
        "data" => Some(ROUTES::DATA),
        _ => None,
    }
}
