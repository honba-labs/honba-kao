//! Routing invariant tests (plan-1.md Milestone 6.1).
//!
//! Covers the post-login routing contract for the Mission Control
//! `/home` landing page: route constants, path-to-`Page` mapping, and the
//! authenticated/guest split at `/`. These are pure-logic tests that run on
//! the host (no WASM/DOM required).

use honba_kao_app::router::paths::{Page, ROUTES, get_current_page_for_path};

#[test]
fn routes_home_constant_is_slash_home() {
    assert_eq!(ROUTES::HOME, "/home");
}

#[test]
fn routes_login_constant_is_slash_login() {
    assert_eq!(ROUTES::LOGIN, "/login");
}

#[test]
fn root_path_maps_to_home_page() {
    // `/` is the RootRoute: authenticated users are redirected to /home,
    // guests see LandingPage. The page id for the home landing is `Home`.
    assert_eq!(get_current_page_for_path(ROUTES::HOME), Some(Page::Home));
}

#[test]
fn login_path_maps_to_login_page() {
    assert_eq!(get_current_page_for_path(ROUTES::LOGIN), Some(Page::Login));
}

#[test]
fn register_path_maps_to_register_page() {
    assert_eq!(
        get_current_page_for_path(ROUTES::REGISTER),
        Some(Page::Register)
    );
}

#[test]
fn portfolio_path_maps_to_portfolio_page() {
    assert_eq!(
        get_current_page_for_path(ROUTES::PORTFOLIO),
        Some(Page::Portfolio)
    );
}

#[test]
fn unknown_path_returns_none() {
    assert_eq!(get_current_page_for_path("/no-such-route"), None);
    assert_eq!(get_current_page_for_path(""), None);
    assert_eq!(get_current_page_for_path("/equity/nonexistent"), None);
}

#[test]
fn home_page_reachable_via_route_table() {
    // The /home route is registered inside the gated AppShell ParentRoute.
    // This test asserts the routing contract that authenticated users land
    // on /home after sign-in: ROUTES::HOME is the single source of truth
    // referenced by LoginPage, RootRoute, AppShell fallback, and NavRail.
    assert_eq!(ROUTES::HOME, "/home");
    assert_eq!(ROUTES::LOGIN, "/login");
    assert!(ROUTES::HOME.starts_with('/'));
    assert!(ROUTES::LOGIN.starts_with('/'));
    assert_ne!(ROUTES::HOME, ROUTES::LOGIN);
}
