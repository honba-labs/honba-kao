//! Saved screener views — persists a snapshot of the Screen Setup Deck /
//! Results panel state (asset class, setup tab, result view, selected
//! preset/universe, query text) to localStorage, mirroring the existing
//! `custom_screens` persistence pattern in [`super::screener_store`] exactly.

use serde::{Deserialize, Serialize};

use super::columns::ScreenerType;
use super::screener_store::{ScreenView, SetupTab};

const SAVED_VIEWS_KEY: &str = "honba_screener_saved_views_v1";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SavedView {
    pub id: String,
    pub name: String,
    pub screener_type: ScreenerType,
    pub setup_tab: SetupTab,
    pub screen_view: ScreenView,
    pub selected_preset_id: Option<String>,
    pub selected_universe_id: String,
    pub query: String,
    pub created_at_ms: u128,
}

impl Default for SavedView {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            screener_type: ScreenerType::Stocks,
            setup_tab: SetupTab::default(),
            screen_view: ScreenView::default(),
            selected_preset_id: None,
            selected_universe_id: String::new(),
            query: String::new(),
            created_at_ms: 0,
        }
    }
}

fn storage() -> Option<web_sys::Storage> {
    web_sys::window().and_then(|window| window.local_storage().ok().flatten())
}

pub fn load_saved_views() -> Vec<SavedView> {
    storage()
        .and_then(|storage| storage.get_item(SAVED_VIEWS_KEY).ok().flatten())
        .and_then(|raw| serde_json::from_str::<Vec<SavedView>>(&raw).ok())
        .unwrap_or_default()
}

pub fn save_saved_views(views: &[SavedView]) {
    if let Some(storage) = storage() {
        if let Ok(json) = serde_json::to_string(views) {
            let _ = storage.set_item(SAVED_VIEWS_KEY, &json);
        }
    }
}
