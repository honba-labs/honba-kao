//! The studio store (HBD-2026-DES-006 P2 "store wiring").
//!
//! The Leptos port of the canvas half of honba-fe algodesigner `store.ts`:
//! one page-scoped struct of granular signals (the simulator store's idiom),
//! with every editor gesture landing on a store action that delegates the
//! graph change to the pure [`kernel::edit`](crate::features::strategy_studio::kernel::edit)
//! functions. Derived reads — validation issues, the graph hash and the
//! generated Python — are `Memo`s, so every graph mutation re-derives the
//! source atomically (fe's invariant).
//!
//! View state is kept apart from graph state: `positions`, `viewport`,
//! `selection` and the splice menu never enter the graph, so nothing
//! canvas-only can leak into `to_manifest`.
//!
//! Not yet here (later phases of the design doc): layout/library/version
//! persistence (P3), the code pane round-trip and verify (P4).

use std::collections::BTreeMap;

use leptos::prelude::*;

use crate::features::simulator::kernel::selection::{StrategyGraphNode, StrategyGraphShape};
use crate::features::strategy_studio::canvas::geometry::{
    auto_layout, snap_position, Position, Viewport, GRID,
};
use crate::features::strategy_studio::canvas::scene::Selection;
use crate::features::strategy_studio::kernel::codegen::generate_code;
use crate::features::strategy_studio::kernel::edit;
use crate::features::strategy_studio::kernel::graph::{graph_hash, graph_issues, ValidationIssue};
use crate::features::strategy_studio::kernel::templates::{blank_graph, template_by_name};

/// The page-scoped studio store. `Copy`: every field is an arena handle.
#[derive(Clone, Copy, Debug)]
pub struct StudioStore {
    /// Whether a strategy is open in the editor (else the library shows).
    pub open: RwSignal<bool>,
    /// Where the open graph came from (`template · sma_cross`, `new`, …).
    pub origin: RwSignal<String>,
    /// The strategy graph being edited.
    pub graph: RwSignal<StrategyGraphShape>,
    /// Canvas-only node positions (view state, never in the manifest).
    pub positions: RwSignal<BTreeMap<String, Position>>,
    /// The editor selection.
    pub selection: RwSignal<Selection>,
    /// Pan/zoom.
    pub viewport: RwSignal<Viewport>,
    /// The edge whose "+" splice menu is open.
    pub splice_menu: RwSignal<Option<String>>,
    /// The last refused gesture, rendered verbatim (cleared by the next
    /// successful edit).
    pub notice: RwSignal<Option<String>>,
    /// Whether the graph changed since it was opened.
    pub modified: RwSignal<bool>,
    /// `graph_issues` over the current graph, in fe's code order.
    pub issues: Memo<Vec<ValidationIssue>>,
    /// fe-compatible graph hash.
    pub hash: Memo<String>,
    /// The deterministic Python rendering of the graph.
    pub source: Memo<String>,
}

impl Default for StudioStore {
    fn default() -> Self {
        Self::new()
    }
}

impl StudioStore {
    /// A fresh store with nothing open. Call inside the page component so
    /// the memos root in the page's reactive owner.
    pub fn new() -> Self {
        let graph = RwSignal::new(blank_graph());
        let issues = Memo::new(move |_| graph.with(graph_issues));
        let hash = Memo::new(move |_| graph.with(graph_hash));
        let source = Memo::new(move |_| {
            let hash = hash.get();
            graph.with(|g| generate_code(g, &hash))
        });
        Self {
            open: RwSignal::new(false),
            origin: RwSignal::new(String::new()),
            graph,
            positions: RwSignal::new(BTreeMap::new()),
            selection: RwSignal::new(Selection::None),
            viewport: RwSignal::new(Viewport::default()),
            splice_menu: RwSignal::new(None),
            notice: RwSignal::new(None),
            modified: RwSignal::new(false),
            issues,
            hash,
            source,
        }
    }

    // -- Lifecycle ----------------------------------------------------------

    /// Opens `graph` in the editor (fe `setGraph`): auto-laid-out, nothing
    /// selected, viewport reset, unmodified.
    pub fn open_graph(&self, graph: StrategyGraphShape, origin: impl Into<String>) {
        self.positions.set(auto_layout(&graph));
        self.graph.set(graph);
        self.origin.set(origin.into());
        self.selection.set(Selection::None);
        self.viewport.set(Viewport::default());
        self.splice_menu.set(None);
        self.notice.set(None);
        self.modified.set(false);
        self.open.set(true);
    }

    /// Opens one of fe's starting templates by name. `false` when unknown.
    pub fn open_template(&self, name: &str) -> bool {
        match template_by_name(name) {
            Some(template) => {
                self.open_graph(template.graph, format!("template · {}", template.name));
                true
            }
            None => false,
        }
    }

    /// "+ New strategy": the blank graph.
    pub fn new_strategy(&self) {
        self.open_graph(blank_graph(), "new");
    }

    /// Back to the library. The graph stays in memory until the next open.
    pub fn close(&self) {
        self.splice_menu.set(None);
        self.selection.set(Selection::None);
        self.open.set(false);
    }

    // -- Selection ----------------------------------------------------------

    /// Selects a node or edge (or clears with [`Selection::None`]); any open
    /// splice menu closes.
    pub fn select(&self, selection: Selection) {
        if self.selection.get_untracked() != selection {
            self.selection.set(selection);
        }
        if self.splice_menu.get_untracked().is_some() {
            self.splice_menu.set(None);
        }
    }

    /// fe's Escape rule: clear the selection first, then close whatever
    /// else is open. Returns whether anything changed.
    pub fn escape(&self) -> bool {
        if self.selection.get_untracked() != Selection::None {
            self.selection.set(Selection::None);
            return true;
        }
        if self.splice_menu.get_untracked().is_some() {
            self.splice_menu.set(None);
            return true;
        }
        if self.notice.get_untracked().is_some() {
            self.notice.set(None);
            return true;
        }
        false
    }

    // -- Graph edits --------------------------------------------------------

    fn edit<R>(&self, f: impl FnOnce(&mut StrategyGraphShape) -> R) -> R {
        let mut out = None;
        self.graph.update(|graph| out = Some(f(graph)));
        self.modified.set(true);
        out.expect("graph update ran")
    }

    fn refuse(&self, error: edit::EditError) {
        self.notice.set(Some(error.message()));
    }

    fn accept(&self) {
        if self.notice.get_untracked().is_some() {
            self.notice.set(None);
        }
    }

    /// Drops positions of nodes the graph no longer has.
    fn prune_positions(&self) {
        let ids: Vec<String> = self
            .graph
            .with_untracked(|g| g.nodes.iter().map(|n| n.id.clone()).collect());
        self.positions.update(|positions| positions.retain(|id, _| ids.contains(id)));
    }

    /// fe `addNode`: appends at `position` (snapped), selects it, returns
    /// the id it landed under.
    pub fn add_node_at(&self, node: StrategyGraphNode, position: Position) -> String {
        let id = self.edit(|graph| edit::add_node(graph, node));
        self.positions
            .update(|positions| {
                positions.insert(id.clone(), snap_position(position));
            });
        self.select(Selection::Node(id.clone()));
        self.accept();
        id
    }

    /// Adds a node into the visible area (the palette's click path, so the
    /// palette works without drag-and-drop), cascading so repeated adds
    /// don't stack exactly.
    pub fn add_node_visible(&self, node: StrategyGraphNode) -> String {
        let count = self.graph.with_untracked(|g| g.nodes.len()) % 8;
        let step = GRID * 2.0 * count as f64;
        let (wx, wy) = self.viewport.get_untracked().to_world(64.0 + step, 64.0 + step);
        self.add_node_at(node, Position::new(wx, wy))
    }

    /// Moves a node (drag); the caller passes an already-snapped position.
    pub fn move_node(&self, node_id: &str, position: Position) {
        let current = self.positions.with_untracked(|p| p.get(node_id).copied());
        if current == Some(position) {
            return;
        }
        self.positions.update(|positions| {
            positions.insert(node_id.to_string(), position);
        });
        self.modified.set(true);
    }

    /// fe `deleteNode`.
    pub fn delete_node(&self, node_id: &str) {
        if self.edit(|graph| edit::remove_node(graph, node_id)) {
            self.prune_positions();
            if self.selection.get_untracked() == Selection::Node(node_id.to_string()) {
                self.selection.set(Selection::None);
            }
        }
    }

    /// fe `deleteEdge`.
    pub fn delete_edge(&self, edge_id: &str) {
        if self.edit(|graph| edit::remove_edge(graph, edge_id))
            && self.selection.get_untracked() == Selection::Edge(edge_id.to_string())
        {
            self.selection.set(Selection::None);
        }
        if self.splice_menu.get_untracked().as_deref() == Some(edge_id) {
            self.splice_menu.set(None);
        }
    }

    /// fe `deleteSelected` (the Delete/Backspace key).
    pub fn delete_selection(&self) -> bool {
        match self.selection.get_untracked() {
            Selection::Node(id) => {
                self.delete_node(&id);
                true
            }
            Selection::Edge(id) => {
                self.delete_edge(&id);
                true
            }
            Selection::None => false,
        }
    }

    /// fe `onConnect`, with the kernel's type/cycle checks; a refusal is
    /// surfaced in `notice` and leaves the graph untouched.
    pub fn connect(&self, from: &str, from_port: &str, to: &str, to_port: &str) {
        let mut candidate = self.graph.get_untracked();
        match edit::connect(&mut candidate, from, from_port, to, to_port) {
            Ok(edge_id) => {
                self.graph.set(candidate);
                self.modified.set(true);
                self.select(Selection::Edge(edge_id));
                self.accept();
            }
            Err(error) => self.refuse(error),
        }
    }

    /// fe `spliceEdge`: `a → b` becomes `a → node → b`, the new card placed
    /// midway between the two endpoints and selected.
    pub fn splice(&self, edge_id: &str, node: StrategyGraphNode, in_port: &str, out_port: &str) {
        let mut candidate = self.graph.get_untracked();
        let endpoints = candidate
            .edges
            .iter()
            .find(|e| e.id == edge_id)
            .map(|e| (e.from_node.clone(), e.to_node.clone()));
        match edit::splice_edge(&mut candidate, edge_id, node, in_port, out_port) {
            Ok(new_id) => {
                let position = self.positions.with_untracked(|positions| {
                    let (from, to) = endpoints.unwrap_or_default();
                    let a = positions.get(&from).copied().unwrap_or(Position::new(96.0, 96.0));
                    let b = positions.get(&to).copied().unwrap_or(Position::new(400.0, 96.0));
                    Position::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0 + GRID * 6.0)
                });
                self.graph.set(candidate);
                self.positions.update(|positions| {
                    positions.insert(new_id.clone(), snap_position(position));
                });
                self.modified.set(true);
                self.splice_menu.set(None);
                self.select(Selection::Node(new_id));
                self.accept();
            }
            Err(error) => {
                self.splice_menu.set(None);
                self.refuse(error);
            }
        }
    }

    /// Writes one node parameter.
    pub fn set_param(&self, node_id: &str, name: &str, value: serde_json::Value) {
        self.edit(|graph| edit::set_param(graph, node_id, name, value));
    }

    /// Removes one node parameter.
    pub fn remove_param(&self, node_id: &str, name: &str) {
        self.edit(|graph| edit::remove_param(graph, node_id, name));
    }

    /// Relabels a node (empty clears).
    pub fn set_label(&self, node_id: &str, label: &str) {
        self.edit(|graph| edit::set_label(graph, node_id, label));
    }

    /// Switches an indicator node's kind (resets params to defaults).
    pub fn set_indicator(&self, node_id: &str, kind: &str) {
        self.edit(|graph| edit::set_indicator(graph, node_id, kind));
    }

    // -- Strategy-level settings ------------------------------------------

    /// Renames the strategy.
    pub fn set_name(&self, name: &str) {
        let name = name.trim().to_string();
        self.edit(|graph| graph.name = name);
    }

    /// Sets the single explicit instrument (the NSE single-symbol rule).
    pub fn set_instrument(&self, symbol: &str, exchange: &str) {
        let symbol = symbol.trim().to_uppercase();
        let exchange = exchange.trim().to_uppercase();
        self.edit(|graph| {
            graph.universe_named.clear();
            graph.universe_explicit = if symbol.is_empty() {
                Vec::new()
            } else {
                vec![(symbol, if exchange.is_empty() { "NSE".to_string() } else { exchange })]
            };
        });
    }

    /// Sets the driving timeframe.
    pub fn set_timeframe(&self, interval: u32, aggregation: &str) {
        let aggregation = aggregation.to_string();
        self.edit(|graph| {
            graph.driving_interval = interval;
            graph.driving_aggregation = aggregation;
        });
    }

    /// Sets the warm-up bar count.
    pub fn set_warmup(&self, bars: u32) {
        self.edit(|graph| graph.warmup_bars = bars);
    }

    // -- View -------------------------------------------------------------

    /// Re-runs the layered auto-layout over the current graph.
    pub fn auto_layout(&self) {
        let positions = self.graph.with_untracked(auto_layout);
        self.positions.set(positions);
    }
}
