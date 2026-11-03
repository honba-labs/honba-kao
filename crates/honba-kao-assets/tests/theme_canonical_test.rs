//! Spec (`honba-kao-assets-spec.md`) §6.3 consolidation: the canonical Honba
//! palette lives in this crate (`assets/themes/theme.json`); the desktop
//! runners load it via `themes::default_theme()`, while the web flow
//! (`nsh run --target web`) keeps reading each crate's local `theme.json`
//! through `nsh.json`. Those local copies must therefore stay byte-equal to
//! the canonical file — this test fails the moment one of them drifts.

const CANONICAL: &str = include_str!("../assets/themes/theme.json");
const APP_COPY: &str = include_str!("../../honba-kao-app/theme.json");

#[test]
fn the_app_theme_json_matches_the_canonical_theme() {
    assert_eq!(
        APP_COPY, CANONICAL,
        "honba-kao-app/theme.json drifted from the canonical honba theme"
    );
}
