//! Language context. Mirrors `web/src/contexts/LanguageContext.tsx`.

use leptos::prelude::*;

use crate::i18n::{Language, t};

#[derive(Copy, Clone)]
pub struct LanguageContext {
    pub language: RwSignal<Language>,
}

impl LanguageContext {
    pub fn new() -> Self {
        Self {
            language: RwSignal::new(Language::En),
        }
    }

    pub fn set(&self, language: Language) {
        self.language.set(language);
    }

    pub fn toggle(&self) {
        self.language.update(|l| {
            *l = match l {
                Language::En => Language::Ja,
                Language::Ja => Language::En,
            }
        });
    }

    /// Translate `key` in the active language.
    pub fn t(&self, key: &str) -> &'static str {
        t(key, self.language.get())
    }
}

impl Default for LanguageContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn use_language() -> LanguageContext {
    expect_context::<LanguageContext>()
}
