//! Theme registry: named skins, a system preference, and persistence.
//!
//! Every skin is a `:root[data-theme="<id>"]` block in `style/themes.css`
//! defining one token contract, so switching a skin is a single dataset write
//! on `<html>`. `data-mode` is derived from the skin and drives `color-scheme`
//! plus the OpenAlgo chart palette, which reads the resolved tokens on demand.

pub mod palette;
mod store;

use leptos::prelude::*;
use wasm_bindgen::prelude::*;

pub use palette::{ColorToken, get_palette_256};

/// Which side of the palette a skin belongs to.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Dark,
    Light,
}

impl Mode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Mode::Dark => "dark",
            Mode::Light => "light",
        }
    }
}

/// One shippable skin. `id` is the `data-theme` value in `style/themes.css`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ThemeSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub mode: Mode,
}

/// The skins, dark first. Order is the picker order.
pub const THEMES: &[ThemeSpec] = &[
    ThemeSpec {
        id: "dark",
        label: "Honba Dark",
        mode: Mode::Dark,
    },
    ThemeSpec {
        id: "midnight",
        label: "Midnight",
        mode: Mode::Dark,
    },
    ThemeSpec {
        id: "nord",
        label: "Nord",
        mode: Mode::Dark,
    },
    ThemeSpec {
        id: "gruvbox-dark",
        label: "Gruvbox Dark",
        mode: Mode::Dark,
    },
    ThemeSpec {
        id: "light",
        label: "NOFX Light",
        mode: Mode::Light,
    },
    ThemeSpec {
        id: "github-light",
        label: "GitHub Light",
        mode: Mode::Light,
    },
    ThemeSpec {
        id: "solarized-light",
        label: "Solarized Light",
        mode: Mode::Light,
    },
    ThemeSpec {
        id: "catppuccin-latte",
        label: "Catppuccin Latte",
        mode: Mode::Light,
    },
];

/// Skin shown on a first visit. noFX's palette is the house look.
pub const DEFAULT_THEME: &str = "light";

/// Defaults used when the preference is `System` or a stored id is unknown.
pub const FALLBACK_DARK: &str = "dark";
pub const FALLBACK_LIGHT: &str = "light";

pub fn find_theme(id: &str) -> Option<&'static ThemeSpec> {
    THEMES.iter().find(|theme| theme.id == id)
}

/// What the user picked in the theme menu.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Preference {
    /// Follow `prefers-color-scheme`.
    System,
    /// A specific skin.
    Theme(&'static str),
}

#[derive(Copy, Clone)]
pub struct ThemeContext {
    pub preference: RwSignal<Preference>,
    /// The skin that is actually applied, after resolving `System`.
    pub resolved: Memo<&'static ThemeSpec>,
}

impl ThemeContext {
    pub fn new() -> Self {
        let preference = RwSignal::new(store::load_preference());
        let resolved = Memo::new(move |_| resolve(preference.get()));

        Effect::new(move |_| {
            let theme = resolved.get();
            apply(theme);
            store::save_preference(preference.get_untracked());
        });

        Self {
            preference,
            resolved,
        }
    }

    pub fn set(&self, preference: Preference) {
        self.preference.set(preference);
    }

    pub fn current(&self) -> &'static ThemeSpec {
        self.resolved.get()
    }

    pub fn mode(&self) -> Mode {
        self.resolved.get().mode
    }

    pub fn is_dark(&self) -> bool {
        self.mode() == Mode::Dark
    }

    /// Quick toggle for the header button: flip to the default skin of the
    /// other mode. Named skins stay reachable from the picker.
    pub fn toggle(&self) {
        let next = if self.is_dark() {
            FALLBACK_LIGHT
        } else {
            FALLBACK_DARK
        };
        self.set(Preference::Theme(next));
    }
}

impl Default for ThemeContext {
    fn default() -> Self {
        Self::new()
    }
}

fn resolve(preference: Preference) -> &'static ThemeSpec {
    match preference {
        Preference::Theme(id) => find_theme(id)
            .or_else(|| find_theme(FALLBACK_DARK))
            .expect("fallback theme must exist"),
        Preference::System => {
            let dark = store::prefers_dark().unwrap_or(true);
            find_theme(if dark { FALLBACK_DARK } else { FALLBACK_LIGHT })
                .expect("fallback theme must exist")
        }
    }
}

/// Write the skin to `<html>` and restyle live charts.
fn apply(theme: &'static ThemeSpec) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(root) = document.document_element() else {
        return;
    };

    let _ = root.set_attribute("data-theme", theme.id);
    let _ = root.set_attribute("data-mode", theme.mode.as_str());

    // Kept for the handful of rules that still branch on the mode class.
    if let Some(body) = document.body() {
        body.set_class_name(match theme.mode {
            Mode::Dark => "Dark",
            Mode::Light => "Light",
        });
    }

    refresh_charts();
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(
        js_namespace = ["window", "honbaCharts"],
        js_name = refreshThemeAll,
        catch
    )]
    fn js_refresh_theme_all() -> Result<JsValue, JsValue>;
}

/// Ask the chart bridge to re-read the skin. A no-op when no chart is mounted
/// or when the engine has not loaded yet.
fn refresh_charts() {
    let _ = js_refresh_theme_all();
}
