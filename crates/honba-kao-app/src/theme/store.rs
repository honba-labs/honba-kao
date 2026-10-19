//! Theme persistence and the system colour-scheme probe.

use web_sys::window;

use super::{DEFAULT_THEME, FALLBACK_DARK, FALLBACK_LIGHT, Preference, find_theme};

const STORAGE_KEY: &str = "honba-theme";

pub fn load_preference() -> Preference {
    let Some(storage) = window().and_then(|window| window.local_storage().ok().flatten()) else {
        return Preference::Theme(DEFAULT_THEME);
    };
    match storage.get_item(STORAGE_KEY) {
        Ok(Some(raw)) => parse(&raw),
        _ => Preference::Theme(DEFAULT_THEME),
    }
}

pub fn save_preference(preference: Preference) {
    let Some(storage) = window().and_then(|window| window.local_storage().ok().flatten()) else {
        return;
    };

    match preference {
        Preference::System => {
            // Written explicitly: an absent key means "first visit", which
            // resolves to the default noFX skin rather than the OS scheme.
            let _ = storage.set_item(STORAGE_KEY, "system");
        }
        Preference::Theme(id) => {
            let _ = storage.set_item(STORAGE_KEY, id);
        }
    }
}

pub fn prefers_dark() -> Option<bool> {
    let window = window()?;
    let query = window.match_media("(prefers-color-scheme: dark)").ok()??;
    Some(query.matches())
}

fn parse(raw: &str) -> Preference {
    match raw.trim() {
        "" => Preference::Theme(DEFAULT_THEME),
        "system" => Preference::System,
        // Values written by the earlier two-theme implementation.
        "Dark" => Preference::Theme(FALLBACK_DARK),
        "Light" => Preference::Theme(FALLBACK_LIGHT),
        id => match find_theme(id) {
            Some(theme) => Preference::Theme(theme.id),
            None => Preference::System,
        },
    }
}
