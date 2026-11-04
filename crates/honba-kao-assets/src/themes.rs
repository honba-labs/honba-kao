//! Canonical theme palettes in the shadcn `cssVars` shape that
//! `nsh_base::Theme::from_json` parses: `{"name", "cssVars": {"theme",
//! "light", "dark"}}`. The three shipped files share one key set; see
//! the spec §5 design-token mapping table for how the honba tokens map
//! onto the cssVars keys.

use nsh_base::Theme;

/// The canonical raw JSON string for the default Honba theme.
pub const DEFAULT_THEME_JSON: &str = include_str!("../assets/themes/theme.json");

/// The High-Contrast Monokai Pro Trader theme.
pub const MONOKAI_THEME_JSON: &str = include_str!("../assets/themes/theme-monokai.json");

/// The Classic Bloomberg Terminal theme.
pub const BLOOMBERG_THEME_JSON: &str = include_str!("../assets/themes/theme-bloomberg.json");

/// Parses the default theme into an NSH `Theme`.
pub fn default_theme() -> Theme {
    Theme::from_json(DEFAULT_THEME_JSON).expect("default theme.json is valid")
}

/// Retrieves a theme by key identifier.
pub fn theme_by_name(name: &str) -> Option<Theme> {
    let json = match name {
        "default" | "honba" => DEFAULT_THEME_JSON,
        "monokai" => MONOKAI_THEME_JSON,
        "bloomberg" => BLOOMBERG_THEME_JSON,
        _ => return None,
    };
    Theme::from_json(json).ok()
}
