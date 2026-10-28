//! Design tokens: semantic CSS class names and CSS custom-property injection.
//!
//! Components emit these class strings so every visual token flows from a single
//! source of truth instead of ad-hoc inline styles.

use crate::theme::palette::Palette;

/// Tailwind-ish semantic class names mapped to the token palette.
pub mod class {
    /// Backgrounds
    pub const BG_CANVAS: &str = "bg-terminal-dark";
    pub const BG_SURFACE: &str = "bg-surface-base";
    pub const BG_SURFACE_MUTED: &str = "bg-surface-subtle";
    pub const BG_SURFACE_ELEVATED: &str = "bg-surface-elevated";

    /// Text
    pub const TEXT_HIGH: &str = "text-text-high";
    pub const TEXT_MUTED: &str = "text-text-muted";
    pub const TEXT_INVERTED: &str = "text-text-inverted";
    pub const TEXT_POSITIVE: &str = "text-emerald-400";
    pub const TEXT_NEGATIVE: &str = "text-rose-400";
    pub const TEXT_WARNING: &str = "text-amber-400";

    /// Borders
    pub const BORDER_SOFT: &str = "border-border-soft";
    pub const BORDER_STRONG: &str = "border-border-strong";

    /// Radius
    pub const ROUNDED: &str = "rounded";
    pub const ROUNDED_SM: &str = "rounded-sm";
    pub const ROUNDED_MD: &str = "rounded-md";
    pub const ROUNDED_LG: &str = "rounded-lg";
    pub const ROUNDED_FULL: &str = "rounded-full";
}

/// Injects the token palette as CSS custom properties on the document element so
/// both the static Tailwind classes and dynamic inline styles share one source.
///
/// No-op on non-Wasm targets; intended to be called once at app bootstrap.
pub fn inject_css_variables() {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window();
        let Some(document) = window.and_then(|w| w.document()) else {
            return;
        };
        let Some(element) = document.document_element() else {
            return;
        };
        let style = element.style();
        let set = |name: &str, value: &str| {
            let _ = style.set_property(name, value);
        };
        set("--hf-bg-dark", Palette::TERMINAL_DARK);
        set("--hf-surface-base", Palette::SURFACE_BASE);
        set("--hf-surface-subtle", Palette::SURFACE_SUBTLE);
        set("--hf-surface-elevated", Palette::SURFACE_ELEVATED);
        set("--hf-border-soft", Palette::BORDER_SOFT);
        set("--hf-border-strong", Palette::BORDER_STRONG);
        set("--hf-text-high", Palette::TEXT_HIGH);
        set("--hf-text-muted", Palette::TEXT_MUTED);
        set("--hf-positive", Palette::EMERALD);
        set("--hf-negative", Palette::CRIMSON);
        set("--hf-warning", Palette::AMBER);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = Palette::BG;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_constants_are_non_empty() {
        for class in [
            class::BG_CANVAS, class::BG_SURFACE, class::TEXT_HIGH, class::TEXT_MUTED,
            class::BORDER_SOFT, class::ROUNDED, class::TEXT_POSITIVE, class::TEXT_NEGATIVE,
        ] {
            assert!(!class.is_empty());
        }
    }
}
