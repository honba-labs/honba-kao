//! Theme management and dynamic class synchronization.

pub mod palette;

use leptos::prelude::*;

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
}

#[derive(Copy, Clone)]
pub struct ThemeContext {
    pub theme: RwSignal<Theme>,
}

impl ThemeContext {
    pub fn new() -> Self {
        let theme = RwSignal::new(Theme::Dark);

        // Sync to document.body class whenever the theme changes
        Effect::new(move |_| {
            let current = theme.get();
            if let Some(window) = web_sys::window() {
                if let Some(doc) = window.document() {
                    if let Some(body) = doc.body() {
                        body.set_class_name(current.as_class());
                    }
                }
            }
        });

        Self { theme }
    }

    pub fn toggle(&self) {
        self.theme.update(|t| *t = t.toggle());
    }
}
