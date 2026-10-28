//! Operations audit-trail store — records every emergency kill-switch
//! execution so the Ops Console's trail is a real, app-wide, timestamped
//! log rather than static placeholder copy. Provided once at the app root
//! (mirrors `TradingModeStore`) so other surfaces (e.g. the OMS compliance
//! page) can read the same trail later.

use chrono::Local;
use leptos::prelude::*;

/// Severity of a logged operations event.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AuditSeverity {
    /// Trading-halting or capital-at-risk action (panic sell, halt engine).
    Critical,
    /// Risk-reducing but non-halting action (cancel pending, safe mode).
    Warning,
}

impl AuditSeverity {
    pub const fn label(self) -> &'static str {
        match self {
            AuditSeverity::Critical => "CRITICAL",
            AuditSeverity::Warning => "WARNING",
        }
    }
}

/// One immutable audit-trail entry.
#[derive(Clone, Debug, PartialEq)]
pub struct AuditEntry {
    pub timestamp: String,
    pub action: String,
    pub detail: String,
    pub severity: AuditSeverity,
}

/// The shared, app-wide operations audit trail.
#[derive(Copy, Clone)]
pub struct OpsAuditStore {
    entries: RwSignal<Vec<AuditEntry>>,
}

impl OpsAuditStore {
    pub fn new() -> Self {
        Self {
            entries: RwSignal::new(Vec::new()),
        }
    }

    /// Newest-first view of the trail.
    pub fn entries(&self) -> Signal<Vec<AuditEntry>> {
        let entries = self.entries;
        Signal::derive(move || {
            let mut list = entries.get();
            list.reverse();
            list
        })
    }

    /// Appends a new entry, stamped with the current local time.
    pub fn log(
        &self,
        action: impl Into<String>,
        detail: impl Into<String>,
        severity: AuditSeverity,
    ) {
        let timestamp = Local::now().format("%H:%M:%S").to_string();
        self.entries.update(|list| {
            list.push(AuditEntry {
                timestamp,
                action: action.into(),
                detail: detail.into(),
                severity,
            });
        });
    }
}

impl Default for OpsAuditStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Access the app-provided operations audit trail.
pub fn use_ops_audit() -> OpsAuditStore {
    expect_context::<OpsAuditStore>()
}
