//! Workbench store — the Leptos translation of the labs app's
//! `core/workbench/store.ts` (Zustand), provided on the Workbench page.
//!
//! The state lives in three slices that never touch one another's signals:
//! [`LayoutStore`] (grid, panes, dialogs, drawer), [`OrderDraftStore`] (the
//! order ticket) and [`BookStore`] (the trading book). The few actions that
//! span slices — a pane change re-pointing the ticket, submitting the ticket
//! into the book — live here and pass state between slices explicitly.

use leptos::prelude::*;

use super::book_store::BookStore;
use super::layout_store::{LayoutStore, PaneField};
use super::order_draft_store::OrderDraftStore;

/// The workbench context: one `Copy` handle over the three slices.
#[derive(Clone, Copy)]
pub struct WorkbenchStore {
    pub layout: LayoutStore,
    pub draft: OrderDraftStore,
    pub book: BookStore,
}

impl WorkbenchStore {
    pub fn new() -> Self {
        Self {
            layout: LayoutStore::new(),
            draft: OrderDraftStore::new(),
            book: BookStore::new(),
        }
    }

    /// Provide on the page scope (`provide_context`).
    pub fn provide(self) {
        provide_context(self);
    }

    /// Fetch from context, defined by [`crate::pages::workbench_page`]'s mount.
    pub fn take() -> Self {
        use_context().expect("WorkbenchStore: provide it above the component")
    }

    // -- Panes (layout → ticket) ---------------------------------------------

    pub fn set_active_pane(&self, index: usize) {
        if let Some(pane) = self.layout.panes.get_untracked().get(index) {
            self.layout.active_pane.set(index);
            self.draft.sync_ticket_with_symbol(&pane.symbol);
        }
    }

    pub fn update_pane(&self, index: usize, field: PaneField) {
        // The pane edit and the ticket follow-up both consume `field`, so the
        // changed symbol is captured first.
        let changed_symbol = match &field {
            PaneField::Symbol(symbol) => Some(symbol.to_uppercase()),
            _ => None,
        };

        self.layout.panes.update(|panes| {
            if let Some(pane) = panes.get_mut(index) {
                match field {
                    PaneField::Symbol(symbol) => pane.symbol = symbol.to_uppercase(),
                    PaneField::Timeframe(timeframe) => pane.timeframe = timeframe,
                    PaneField::Mode(mode) => pane.mode = mode,
                }
            }
        });

        if let Some(symbol) = changed_symbol
            && self.layout.active_pane.get_untracked() == index
        {
            self.draft.sync_ticket_with_symbol(&symbol);
        }
    }

    /// Commit a picked symbol to the pane the search modal was opened from.
    pub fn pick_symbol(&self, symbol: &str) {
        let index = self
            .layout
            .search_target
            .get_untracked()
            .unwrap_or_else(|| self.layout.active_pane.get_untracked());
        self.update_pane(index, PaneField::Symbol(symbol.to_string()));
        self.layout.close_symbol_search();
    }

    // -- Order submission (ticket → book) ------------------------------------

    /// Submit the current ticket draft into the trading book.
    pub fn place_order(&self) {
        self.book.place_order(self.draft.ticket.get_untracked());
    }
}

impl Default for WorkbenchStore {
    fn default() -> Self {
        Self::new()
    }
}
