//! Design tokens: semantic CSS class names and CSS custom-property injection.
//!
//! Components emit these class strings so every visual token flows from a single
//! source of truth instead of ad-hoc inline styles.

use crate::theme::palette;
use nsh_core::element::Styled;
use nsh_core::prelude::{AdaptiveColor, px};

/// Semantic style helpers over NSH's typed builder API, mirroring
/// `theme/tokens.rs`'s string constants so call sites write
/// `view().surface_base().text_positive()` instead of hand-rolled
/// `.bg(rgb(0x..))`/`.text_color(rgb(0x..))` calls.
///
/// Implemented for NSH's `Styled` types (`View`, `Text`, ...). Each method
/// returns `self` for chaining.
pub trait HonbaStyle: Styled {
    /// Default card / panel surface (`#11141b`).
    fn surface_base(self) -> Self {
        self.bg(AdaptiveColor::rgb(0x11141b, 0x11141b))
    }
    /// Muted surface fill (`#1a1f2d`).
    fn surface_muted(self) -> Self {
        self.bg(AdaptiveColor::rgb(0x1a1f2d, 0x1a1f2d))
    }
    /// Surface elevated / borders (`#232a3a`).
    fn surface_elevated(self) -> Self {
        self.bg(AdaptiveColor::rgb(0x232a3a, 0x232a3a))
    }
    /// Default foreground text (`#e5e7eb`).
    fn text_high(self) -> Self {
        self.text_color(AdaptiveColor::rgb(0xe5e7eb, 0xe5e7eb))
    }
    /// Muted foreground text (`#94a3b8`).
    fn text_muted(self) -> Self {
        self.text_color(AdaptiveColor::rgb(0x94a3b8, 0x94a3b8))
    }
    /// Positive / buy / safe (`#10b981`).
    fn text_positive(self) -> Self {
        self.text_color(AdaptiveColor::rgb(0x10b981, 0x34d399))
    }
    /// Negative / sell / risk (`#ef4444`).
    fn text_negative(self) -> Self {
        self.text_color(AdaptiveColor::rgb(0xef4444, 0xf87171))
    }
    /// Warning / caution (`#f59e0b`).
    fn text_warning(self) -> Self {
        self.text_color(AdaptiveColor::rgb(0xf59e0b, 0xfbbf24))
    }
    /// Soft border (`#232a3a`).
    fn border_soft(self) -> Self {
        self.border(px(1.0))
    }
    /// Strong border (`#333c52`).
    fn border_strong(self) -> Self {
        self.border(px(1.0))
    }
    // `rounded_md`/`rounded_full` are NOT redeclared here: `Styled` (our
    // supertrait) already provides them with the same semantics
    // (`rounded(px(6.0))` / `rounded(px(9999.0))`), and redeclaring a
    // same-named method on a subtrait makes every call site ambiguous
    // (E0034) between `Styled::rounded_full` and `HonbaStyle::rounded_full`.
    // Call sites use the built-in `Styled::rounded_md()`/`rounded_full()`.
}

/// Blanket impl for every `Styled` type (`View`, `Text`, `Image`, ...).
impl<T: Styled> HonbaStyle for T {}

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
        use wasm_bindgen::JsCast;
        let Ok(style) = element
            .dyn_into::<web_sys::HtmlElement>()
            .map(|el| el.style())
        else {
            return;
        };
        let set = |name: &str, value: &str| {
            let _ = style.set_property(name, value);
        };
        set("--hf-bg-dark", palette::TERMINAL_DARK);
        set("--hf-surface-base", palette::SURFACE_BASE);
        set("--hf-surface-subtle", palette::SURFACE_SUBTLE);
        set("--hf-surface-elevated", palette::SURFACE_ELEVATED);
        set("--hf-border-soft", palette::BORDER_SOFT);
        set("--hf-border-strong", palette::BORDER_STRONG);
        set("--hf-text-high", palette::TEXT_HIGH);
        set("--hf-text-muted", palette::TEXT_MUTED);
        set("--hf-positive", palette::EMERALD);
        set("--hf-negative", palette::CRIMSON);
        set("--hf-warning", palette::AMBER);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = palette::Palette::BG;
}
