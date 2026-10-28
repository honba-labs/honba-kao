//! Terminal-dark design token palette ported from
//! `/home/pmallapp/tmp/secured-finance-app` (Secured-Finance design system).
//!
//! All values are the raw CSS hex tokens consumed by `honba-kao-app/assets/style/palette_256.css`.
//! This crate re-exports them so Rust-side components and tests share a single source of truth.

/// Terminal Dark background (canvas, panels, surfaces).
pub const TERMINAL_DARK: &str = "#0b0e14";

/// Terminal Darker (elevated surfaces, hover states).
pub const TERMINAL_DARKER: &str = "#090c10";

/// Surface base used for cards, order ticket, dock panels.
pub const SURFACE_BASE: &str = "#11141b";

/// Surface subtle (muted fills, progress tracks, inactive chips).
pub const SURFACE_SUBTLE: &str = "#1a1f2d";

/// Surface elevated (borders, dividers, input outlines).
pub const SURFACE_ELEVATED: &str = "#232a3a";

/// Surface strong (active borders, focus rings).
pub const SURFACE_STRONG: &str = "#333c52";

/// Brand accent — Emerald (`#10b981`), used for buys / bullish / safe states.
pub const EMERALD: &str = "#10b981";
pub const EMERALD_DARK: &str = "#047857";
pub const EMERALD_SOFT: &str = "#064e3b";

/// Risk / danger — Crimson (`#ef4444`), used for sells / bearish / liquidation states.
pub const CRIMSON: &str = "#ef4444";
pub const CRIMSON_DARK: &str = "#b91c1c";
pub const CRIMSON_SOFT: &str = "#450a0a";

/// Caution — Amber (`#f59e0b`), used for warnings / margin calls.
pub const AMBER: &str = "#f59e0b";
pub const AMBER_DARK: &str = "#b45309";
pub const AMBER_SOFT: &str = "#451a03";

/// High-contrast primary text.
pub const TEXT_HIGH: &str = "#e5e7eb";

/// Muted secondary text.
pub const TEXT_MUTED: &str = "#94a3b8";

/// Inverted text (light text on accent backgrounds).
pub const TEXT_INVERTED: &str = "#ffffff";

/// Subtle borders.
pub const BORDER_SOFT: &str = "#232a3a";
pub const BORDER_STRONG: &str = "#333c52";

/// Semantic token groupings used by components.
#[derive(Debug, Clone, Copy)]
pub struct Palette;

impl Palette {
    /// Background of the application canvas.
    pub const BG: &str = TERMINAL_DARK;
    /// Default card / panel surface.
    pub const SURFACE: &str = SURFACE_BASE;
    /// Muted surface fill.
    pub const SURFACE_MUTED: &str = SURFACE_SUBTLE;
    /// Default border color.
    pub const BORDER: &str = BORDER_SOFT;
    /// Default foreground text.
    pub const TEXT: &str = TEXT_HIGH;
    /// Muted foreground text.
    pub const TEXT_MUTED: &str = TEXT_MUTED;
    /// Positive / buy / safe color.
    pub const POSITIVE: &str = EMERALD;
    /// Negative / sell / risk color.
    pub const NEGATIVE: &str = CRIMSON;
    /// Warning / caution color.
    pub const WARNING: &str = AMBER;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_hex_values_are_valid() {
        for hex in [
            TERMINAL_DARK, SURFACE_BASE, SURFACE_SUBTLE, EMERALD, CRIMSON, AMBER,
            TEXT_HIGH, TEXT_MUTED, BORDER_SOFT,
        ] {
            assert!(hex.starts_with('#'));
            assert_eq!(hex.len(), 7);
            u32::from_str_radix(&hex[1..], 16).expect("hex must parse");
        }
    }

    #[test]
    fn semantic_tokens_reference_palette() {
        assert_eq!(Palette::POSITIVE, EMERALD);
        assert_eq!(Palette::NEGATIVE, CRIMSON);
        assert_eq!(Palette::WARNING, AMBER);
        assert_eq!(Palette::BG, TERMINAL_DARK);
    }
}
