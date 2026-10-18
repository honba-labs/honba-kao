//! Theme management: dark/light mode, synced to `<body>` and persisted.

pub mod palette;

use leptos::prelude::*;

const STORAGE_KEY: &str = "honba-theme";

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    pub fn as_class(&self) -> &'static str {
        match self {
            Theme::Dark => "Dark",
            Theme::Light => "Light",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }

    fn from_stored(value: &str) -> Option<Self> {
        match value {
            "Dark" => Some(Theme::Dark),
            "Light" => Some(Theme::Light),
            _ => None,
        }
    }
}

#[derive(Copy, Clone)]
pub struct ThemeContext {
    pub theme: RwSignal<Theme>,
}

impl ThemeContext {
    pub fn new() -> Self {
        let theme = RwSignal::new(read_stored_theme().unwrap_or(Theme::Dark));

        // Sync the body class and persist the choice whenever the theme changes.
        Effect::new(move |_| {
            let current = theme.get();
            if let Some(window) = web_sys::window() {
                if let Some(body) = window.document().and_then(|doc| doc.body()) {
                    body.set_class_name(current.as_class());
                }
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item(STORAGE_KEY, current.as_class());
                }
            }
        });

        Self { theme }
    }

    pub fn toggle(&self) {
        self.theme.update(|t| *t = t.toggle());
    }

    pub fn is_dark(&self) -> bool {
        self.theme.get() == Theme::Dark
    }
}

impl Default for ThemeContext {
    fn default() -> Self {
        Self::new()
    }
}

fn read_stored_theme() -> Option<Theme> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    let stored = storage.get_item(STORAGE_KEY).ok()??;
    Theme::from_stored(&stored)
}
