//! Command grammar, history and the symbol universe behind the CommandBar
//! (`Ctrl+G`) and CommandPalette (`Ctrl+K`).
//!
//! * [`commanding`] — pure parse / fuzzy-score / route logic (OpenTerminalUI `commanding.ts`).
//! * [`palette`] — CommandPalette row model (NAV / ASSET / FUNC / CMD).
//! * [`symbols`] — MOCK symbol universe for suggestions and preview cards until a
//!   market-data search endpoint is wired.

pub mod commanding;
pub mod palette;
pub mod symbols;
