//! Strategy studio stores: one page-scoped [`StudioStore`] (one feature =
//! one store, HBD-2026-DES-006 §6 P3 hint). The page creates it and passes
//! it to the canvas, palette and inspector as a prop.

pub mod studio_store;

pub use studio_store::StudioStore;
