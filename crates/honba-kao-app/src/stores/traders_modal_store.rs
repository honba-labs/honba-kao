//! Traders modal store. Mirrors `web/src/stores/tradersModalStore.ts`.

use leptos::prelude::*;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum TraderModalMode {
    #[default]
    Closed,
    Create,
    Edit,
}

#[derive(Copy, Clone)]
pub struct TradersModalStore {
    pub mode: RwSignal<TraderModalMode>,
    pub target_id: RwSignal<Option<String>>,
}

impl TradersModalStore {
    pub fn new() -> Self {
        Self {
            mode: RwSignal::new(TraderModalMode::Closed),
            target_id: RwSignal::new(None),
        }
    }

    pub fn open_create(&self) {
        self.target_id.set(None);
        self.mode.set(TraderModalMode::Create);
    }

    pub fn open_edit(&self, trader_id: impl Into<String>) {
        self.target_id.set(Some(trader_id.into()));
        self.mode.set(TraderModalMode::Edit);
    }

    pub fn close(&self) {
        self.mode.set(TraderModalMode::Closed);
    }

    pub fn is_open(&self) -> bool {
        self.mode.get() != TraderModalMode::Closed
    }
}

impl Default for TradersModalStore {
    fn default() -> Self {
        Self::new()
    }
}

pub fn use_traders_modal_store() -> TradersModalStore {
    expect_context::<TradersModalStore>()
}
