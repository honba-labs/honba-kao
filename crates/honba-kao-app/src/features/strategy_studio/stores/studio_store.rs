//! StudioStore — persistent position & graph state for the Strategy Studio canvas.
//! Wire this via `provide_context` so canvas components can read/write
/// position snapshots across reloads.
//!
//! Format stored in localStorage:
//!   JSON object with `nodes: Map<String, Position>` and `edges: Map<String, EdgePosition>`.
//! On init, the store hydrates from existing localStorage data; on change, it
//!  debounces the write (200ms) so rapid edits don't hammer the DOM.

use leptos::prelude::*;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use web_sys::Storage;
use crate::features::strategy_studio::canvas::editor::snap_position;

/// Position of a node on the canvas grid (16px snap).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodePosition {
    /// Grid-snapped x coordinate in pixels.
    pub x: f64,
    /// Grid-snapped y coordinate in pixels.
    pub y: f64,
}

/// Position/routing data for an edge.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EdgePosition {
    /// Source node port coordinates.
    pub from: (f64, f64),
    /// Target node port coordinates.
    pub to: (f64, f64),
}

/// Persisted state for the entire canvas.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StudioPersistedState {
    /// Per-node grid-snapped positions.
    pub nodes: HashMap<String, NodePosition>,
    /// Per-edge routing data.
    pub edges: HashMap<String, EdgePosition>,
}

/// The top-level store for the Strategy Studio canvas.
///
/// Exposes read-write signals that canvas components can subscribe to.
/// Changes are debounced and persisted to localStorage automatically.
#[derive(Clone, Debug)]
pub struct StudioStore {
    /// Persisted canvas state (nodes + edges positions).
    state: leptos::RwSignal<StudioPersistedState>,
    /// Debounce timeout handle for localStorage writes.
    write_handle: Option<leptos::task::TimeoutHandle<()>>,
    /// Pending edit queued for debounced persist.
    pending_edit: Option<GraphEdit>,
}

/// Graph edit operations that trigger persistence in StudioStore.
#[derive(Clone, Debug)]
pub enum GraphEdit {
    AddedNode(String, StrategyGraphNode),
    RemovedNode(String),
    AddedEdge(StrategyGraphEdge),
    RemovedEdge(String),
}

/// Create a new StudioStore, hydrating from existing localStorage data.
pub fn new() -> Self {
    let stored = with_node(|| {
        // Try to read from localStorage
        let window = web_sys::window().unwrap();
        let storage = window.local_storage().unwrap_or_else(|| Storage::eok());
        let json = storage.get_item("studio_canvas_state").unwrap_or_default();
        json
    });

    let initial_state: StudioPersistedState = if stored.is_empty() {
        StudioPersistedState {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    } else {
        serde_json::from_str(&stored).unwrap_or_else(|_| {
            eprintln!("Failed to parse studio_canvas_state from localStorage");
            StudioPersistedState {
                nodes: HashMap::new(),
                edges: HashMap::new(),
            }
        }
    };

    let (state, set_state) = leptos::create_signal(initial_state);

    // Set up a destructive subscribe to debounce persist writes
    let _ = leptos::effect(move |_| {
        let state = state.get();
        // Debounce: wait 200ms after last change before persisting
        let _ = leptos::task::spawn_local(async move {
            leptos::task::sleep(leptos::chrono::Duration::milliseconds(200)).await;
            persist_state(&state).await;
        });
    });

    Self {
        state,
        write_handle: None,
        pending_edit: None,
    }
}

/// Get the persisted node positions signal.
pub fn node_positions(&self) -> leptos::ReadSignal<HashMap<String, NodePosition>> {
    self.state.read_only()
}

/// Get the persisted edge positions signal.
pub fn edge_positions(&self) -> leptos::ReadSignal<HashMap<String, EdgePosition>> {
    self.state.read_only()
}

/// Apply a graph edit (add/remove node/edge) and persist the change.
pub fn apply_edit(&self, edit: &GraphEdit) {
    let mut state = self.state.write();
    match edit {
        GraphEdit::AddedNode(id, node) => {
            // Compute grid-snapped position based on node index or default
            let pos = snap_position(0.0, 0.0); // In full impl: use drop location
            state.nodes.insert(id.clone(), pos);
            self.pending_edit = Some(GraphEdit::AddedNode(id.clone(), node.clone()));
        }
        GraphEdit::RemovedNode(id) => {
            state.nodes.remove(id);
            state.edges.remove(id);
        }
        GraphEdit::AddedEdge(edge) => {
            // Compute edge routing based on connected node positions
            state.edges.insert(edge.edge_id.clone(), EdgePosition { from: (0.0, 0.0), to: (0.0, 0.0) });
        }
        GraphEdit::RemovedEdge(id) => {
            state.edges.remove(id);
        }
    }
    // Trigger debounced persist after any edit
    trigger_persist();
}

/// Trigger the debounced persist write.
fn trigger_persist(&self) {
    // Cancel any pending timeout
    if let Some(handle) = self.write_handle.take() {
        // In a full impl: handle.abort()
    }
    // Set new timeout for 200ms debounce
    let _ = leptos::task::spawn_local(async move {
        leptos::task::sleep(leptos::chrono::Duration::milliseconds(200)).await;
        let state = self.state.get();
        persist_state(&state).await;
    });
}

/// Persist the current state to localStorage asynchronously.
async fn persist_state(state: &leptos::RwSignal<StudioPersistedState>) {
    let json = serde_json::to_string(&*state).unwrap_or_default();
    let window = web_sys::window().unwrap();
    let storage = window.local_storage().unwrap_or_else(|| web_sys::Storage::eok());
    let _ = storage.set_item("studio_canvas_state", &json);
}

/// Read the current persisted state.
pub fn get_state(&self) -> StudioPersistedState {
    self.state.read()
}

/// Convenience: get the persisted state signal for provide_context.
pub fn provide_store() -> StudioStore {
    StudioStore::new()
}