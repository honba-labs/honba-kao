//! App-wide reactive stores. Mirrors the reference app's `web/src/stores/`:
//! Zustand stores become plain structs of `RwSignal`s provided via
//! `provide_context`. Feature-owned stores live in each
//! `features/<name>/stores/`.

pub mod ops_audit;
pub mod trading_mode;

pub use ops_audit::{AuditEntry, AuditSeverity, OpsAuditStore, use_ops_audit};
pub use trading_mode::{TradingMode, TradingModeStore};
