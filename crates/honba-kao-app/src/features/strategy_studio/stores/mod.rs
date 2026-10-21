#! StudioStore — persistent position & graph state for the Strategy Studio canvas.
//! Wire this via `provide_context` so canvas components can read/write
//! position snapshots across reloads.

use leptos::prelude::*;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Persisted state for the entire canvas.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StudioPersistedState {
    /// Per-node grid-snapped positions.
    pub nodes: HashMap<String, String>,
    /// Per-edge routing data.
    pub edges: HashMap<String, String>,
}

/// The top-level store for the Strategy Studio canvas.
#[derive(Clone, Debug)]
pub struct StudioStore {
    /// Persisted canvas state.
    state: leptos::RwSignal<StudioPersistedState>,
}

impl StudioStore {
    /// Create a new StudioStore.
    pub fn new() -> Self {
        let initial = StudioPersistedState {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        };
        let (state, _) = leptos::create_signal(initial);
        Self { state }
    }

    /// Get the persisted state signal.
    pub fn state_signal(&self) -> leptos::RwSignal<StudioPersistedState> {
        self.state.clone()
    }
}
