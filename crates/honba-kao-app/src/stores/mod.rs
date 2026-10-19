//! Reactive stores. Mirrors the reference app's `web/src/stores/`: Zustand
//! stores become plain structs of `RwSignal`s provided via `provide_context`.

pub mod screener;
pub mod workbench;
