//! Workbench — the multi-pane charting and order-entry cockpit.
//!
//! * [`components`] — toolbar, chart grid and panes, drawing rail, order
//!   ticket/modal, trading drawer, market depth, symbol search.
//! * [`stores`] — `layout_store`, `order_draft_store` and `book_store`,
//!   composed by `workbench_store`.
//! * [`data`] — the universal symbol-search pool and the NSE bhavcopy rows
//!   backing it.

pub mod components;
pub mod data;
pub mod stores;
pub mod workbench_page;

pub use workbench_page::WorkbenchPage;
