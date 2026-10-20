//! Theme helpers — semantic token access and CSS variable helpers.
//! Mirrors bfx-hf-ui-core styling language mapped to honba-kao's 256-color palette.

// Re-exports are handled in lib.rs

/// Semantic color token access.
///
/// Provides convenient access to CSS custom properties defined in `style/themes.css`.
/// All values resolve to the active theme's 256-color palette tokens.
pub struct Tokens;

impl Tokens {
    /// `--background-color` — main surface background
    pub const BACKGROUND: &'static str = "var(--background-color)";
    /// `--darker-background-color` — card/panel surface
    pub const DARKER_BACKGROUND: &'static str = "var(--darker-background-color)";
    /// `--lighter-background-color` — elevated hover containers
    pub const LIGHTER_BACKGROUND: &'static str = "var(--lighter-background-color)";
    /// `--border-color` — structural borders
    pub const BORDER: &'static str = "var(--border-color)";
    /// `--text-color` — body copy
    pub const TEXT: &'static str = "var(--text-color)";
    /// `--on-accent` — text on accent backgrounds
    pub const ON_ACCENT: &'static str = "var(--on-accent)";
    /// `--pnl-gain` — positive profit/gain color
    pub const PNL_GAIN: &'static str = "var(--pnl-gain)";
    /// `--pnl-loss` — negative loss color
    pub const PNL_LOSS: &'static str = "var(--pnl-loss)";
    /// `--primary-0` — accent tint
    pub const PRIMARY_0: &'static str = "var(--primary-0)";
    /// `--primary-8` — accent text
    pub const PRIMARY_8: &'static str = "var(--primary-8)";
    /// `--primary-9` — pure accent
    pub const PRIMARY_9: &'static str = "var(--primary-9)";
    /// `--shade-0` through `--shade-9` — depth elevation & text scale
    pub const SHADE_0: &'static str = "var(--shade-0)";
    pub const SHADE_1: &'static str = "var(--shade-1)";
    pub const SHADE_2: &'static str = "var(--shade-2)";
    pub const SHADE_3: &'static str = "var(--shade-3)";
    pub const SHADE_4: &'static str = "var(--shade-4)";
    pub const SHADE_5: &'static str = "var(--shade-5)";
    pub const SHADE_6: &'static str = "var(--shade-6)";
    pub const SHADE_7: &'static str = "var(--shade-7)";
    pub const SHADE_8: &'static str = "var(--shade-8)";
    pub const SHADE_9: &'static str = "var(--shade-9)";
    /// `--button-green` — buy/success button
    pub const BUTTON_GREEN: &'static str = "var(--button-green)";
    /// `--button-red` — sell/loss button
    pub const BUTTON_RED: &'static str = "var(--button-red)";
    /// `--button-blue` — info button
    pub const BUTTON_BLUE: &'static str = "var(--button-blue)";
}

/// CSS variable helpers for inline styles.
pub mod css_vars {
    use super::*;

    /// Create a background-color style string.
    pub fn bg(var: &'static str) -> String {
        format!("background-color: {var};")
    }

    /// Create a color style string.
    pub fn color(var: &'static str) -> String {
        format!("color: {var};")
    }

    /// Create a border-color style string.
    pub fn border(var: &'static str) -> String {
        format!("border-color: {var};")
    }

    /// Create a full button style from variant tokens.
    pub fn button_style(variant: &str) -> String {
        match variant {
            "default" => format!("background: {}; color: {}", Tokens::LIGHTER_BACKGROUND, Tokens::TEXT),
            "green" => format!("background: {}; color: {}", Tokens::BUTTON_GREEN, Tokens::ON_ACCENT),
            "red" => format!("background: {}; color: {}", Tokens::BUTTON_RED, Tokens::ON_ACCENT),
            "blue" => format!("background: {}; color: {}", Tokens::BUTTON_BLUE, Tokens::ON_ACCENT),
            "ghost" => format!("background: transparent; color: {}", Tokens::SHADE_6),
            _ => String::new(),
        }
    }
}

/// Palette256 struct for runtime theme management.
///
/// Mirrors the 256-color canonical palette architecture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palette256 {
    pub colors: [String; 256],
}

impl Palette256 {
    /// Get a color by index (0-255).
    pub fn get(&self, index: usize) -> Option<&str> {
        self.colors.get(index).map(|s| s.as_str())
    }

    /// Get a color by semantic name (e.g., "--c-marine-02").
    pub fn get_by_name(&self, _name: &str) -> Option<&str> {
        // Implemented via CSS custom properties at runtime
        None
    }
}

impl Default for Palette256 {
    fn default() -> Self {
        // Default dark theme palette (BFX Honey) — first 16 canonical entries,
        // remaining 240 generated as interpolations until palette_256.css is ported.
        const BASE: [&str; 16] = [
            "#0a1017", "#d9383e", "#00b88d", "#d9b300", "#0073d9", "#b340b3", "#00a3a3",
            "#d0d7de", "#4f5a62", "#f05359", "#00e0ad", "#fcd206", "#0089ff", "#e05ce0",
            "#00d9d9", "#ffffff",
        ];
        let colors: [String; 256] = std::array::from_fn(|i| {
            if i < 16 {
                BASE[i].to_string()
            } else {
                // Cycle through base hues with varying lightness until full port.
                let base = BASE[i % 16];
                format!("{base}{:02x}", 128 + (i as u32 % 128))
            }
        });
        Self { colors }
    }
}

/// ThemeContext reactive state — managed by honba-kao-app::theme::ThemeContext.
/// Re-exported here for ui-core components that need theme awareness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

impl ThemeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            ThemeMode::Dark => "dark",
            ThemeMode::Light => "light",
        }
    }
}