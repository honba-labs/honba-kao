//! Traders configuration store. Mirrors
//! `web/src/stores/tradersConfigStore.ts`.

use leptos::prelude::*;

use crate::types::TraderInfo;

#[derive(Copy, Clone)]
pub struct TradersConfigStore {
    pub traders: RwSignal<Vec<TraderInfo>>,
    pub selected_trader_id: RwSignal<Option<String>>,
    pub show_beginner_onboarding: RwSignal<bool>,
}

impl TradersConfigStore {
    pub fn new() -> Self {
        Self {
            traders: RwSignal::new(crate::data::demo_data::demo_traders()),
            selected_trader_id: RwSignal::new(None),
            show_beginner_onboarding: RwSignal::new(false),
        }
    }

    pub fn select(&self, trader_id: impl Into<String>) {
        self.selected_trader_id.set(Some(trader_id.into()));
    }

    pub fn selected(&self) -> Option<TraderInfo> {
        let id = self.selected_trader_id.get()?;
        self.traders.get().into_iter().find(|t| t.trader_id == id)
    }

    /// Default the selection to the first trader if nothing is picked.
    pub fn ensure_selection(&self) {
        if self.selected_trader_id.get().is_none()
            && let Some(first) = self.traders.get().first()
        {
            self.selected_trader_id.set(Some(first.trader_id.clone()));
        }
    }

    pub fn find_by_slug(&self, slug: &str) -> Option<TraderInfo> {
        self.traders.get().into_iter().find(|t| t.slug() == slug)
    }

    pub fn find_by_id(&self, trader_id: &str) -> Option<TraderInfo> {
        self.traders
            .get()
            .into_iter()
            .find(|t| t.trader_id == trader_id)
    }
}

impl Default for TradersConfigStore {
    fn default() -> Self {
        Self::new()
    }
}

pub fn use_traders_config_store() -> TradersConfigStore {
    expect_context::<TradersConfigStore>()
}
