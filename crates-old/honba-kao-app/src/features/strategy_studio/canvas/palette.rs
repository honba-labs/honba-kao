//! Node palette (HBD-2026-DES-006 P2) — port of honba-fe algodesigner
//! `components/NodePalette.tsx`.
//!
//! Items drag onto the canvas with HTML5 drag-and-drop under fe's MIME
//! type [`HONBA_NODE_MIME`]; the payload is a tiny JSON document parsed back
//! by [`parse_palette_payload`], which only accepts known node types and
//! catalogue indicators (a foreign drop can never inject an arbitrary node).
//! Clicking an item adds it into the visible area — the keyboard/no-DnD path
//! fe lacked.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::features::simulator::kernel::selection::StrategyGraphNode;
use crate::features::strategy_studio::canvas::dom;
use crate::features::strategy_studio::kernel::graph::{
    CATALOGUE, NodeCategory, STRUCTURAL_NODE_TYPES, default_node, is_known_indicator,
};
use crate::features::strategy_studio::stores::StudioStore;

/// fe's drag MIME type.
pub const HONBA_NODE_MIME: &str = "application/honba-node";

/// One palette entry.
#[derive(Clone, Debug, PartialEq)]
pub struct PaletteItem {
    /// Graph node type.
    pub node_type: &'static str,
    /// Indicator kind, for indicator entries.
    pub indicator: Option<&'static str>,
    /// Display label.
    pub label: String,
    /// One-line hint.
    pub hint: &'static str,
    /// Palette group.
    pub category: NodeCategory,
}

/// The drag payload (`application/honba-node`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PalettePayload {
    /// Graph node type.
    pub node_type: String,
    /// Indicator kind, for indicator nodes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indicator: Option<String>,
}

fn structural_hint(node_type: &str) -> &'static str {
    match node_type {
        "universe" => "the instrument's bar series",
        "condition" => "compare two series (or one against a threshold)",
        "logic" => "combine two signals (and / or / not)",
        "sizing" => "turn a signal into a position size",
        "risk" => "gate a signal behind a drawdown guard",
        "order" => "emit a buy or sell order",
        "schedule" => "a session-time signal",
        "benchmark" => "a comparison series",
        _ => "",
    }
}

fn title_case(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Every palette entry: the structural node types plus the nine catalogue
/// indicators, each tagged with its category.
pub fn palette_items() -> Vec<PaletteItem> {
    let mut items: Vec<PaletteItem> = STRUCTURAL_NODE_TYPES
        .iter()
        .map(|&node_type| PaletteItem {
            node_type,
            indicator: None,
            label: title_case(node_type),
            hint: structural_hint(node_type),
            category: NodeCategory::of(node_type),
        })
        .collect();
    items.extend(CATALOGUE.iter().map(|entry| PaletteItem {
        node_type: entry.node_type,
        indicator: Some(entry.kind),
        label: entry.kind.to_uppercase(),
        hint: entry.summary,
        category: NodeCategory::Indicator,
    }));
    items
}

/// The palette's group order.
pub const PALETTE_GROUPS: [NodeCategory; 5] = [
    NodeCategory::Data,
    NodeCategory::Indicator,
    NodeCategory::Logic,
    NodeCategory::Risk,
    NodeCategory::Execution,
];

/// Serializes an item to the drag payload.
pub fn palette_payload(item: &PaletteItem) -> String {
    serde_json::to_string(&PalettePayload {
        node_type: item.node_type.to_string(),
        indicator: item.indicator.map(str::to_string),
    })
    .unwrap_or_default()
}

/// Parses a drop payload. `None` for anything that is not a palette item:
/// unknown fields, unknown node types, an indicator node without a
/// catalogue kind, or a kind on a non-indicator node.
pub fn parse_palette_payload(data: &str) -> Option<PalettePayload> {
    let payload: PalettePayload = serde_json::from_str(data.trim()).ok()?;
    let structural = STRUCTURAL_NODE_TYPES.contains(&payload.node_type.as_str());
    match (payload.node_type.as_str(), payload.indicator.as_deref()) {
        ("indicator", Some(kind)) if is_known_indicator(kind) => Some(payload),
        (_, None) if structural => Some(payload),
        _ => None,
    }
}

/// A fresh node for a payload, with the catalogue / fe defaults filled in
/// and an empty id (the store assigns a unique one).
pub fn palette_node(payload: &PalettePayload) -> StrategyGraphNode {
    let mut node = default_node("", &payload.node_type, payload.indicator.as_deref());
    match payload.node_type.as_str() {
        // fe's splice-menu defaults for the two nodes `default_node` leaves bare.
        "sizing" => {
            node.params
                .insert("mode".to_string(), "fixed_quantity".into());
            node.params.insert("value".to_string(), 1.into());
        }
        "risk" => {
            node.params.insert("max_drawdown".to_string(), 0.05.into());
        }
        _ => {}
    }
    if node.label.is_none() {
        node.label = Some(title_case(&payload.node_type));
    }
    node
}

/// The floating node palette.
#[component]
pub fn Palette(
    /// The studio store.
    store: StudioStore,
) -> impl IntoView {
    let items = palette_items();
    view! {
        <aside class="studio-palette" aria-label="Node palette">
            <div class="studio-palette__head">
                <span class="studio-palette__title">"Nodes"</span>
                <span class="studio-palette__note">"drag onto the canvas or click to add"</span>
            </div>
            {PALETTE_GROUPS
                .iter()
                .map(|group| {
                    let group = *group;
                    let entries: Vec<PaletteItem> = items
                        .iter()
                        .filter(|item| item.category == group)
                        .cloned()
                        .collect();
                    view! {
                        <div class=format!(
                            "studio-palette__group studio-cat--{}",
                            category_slug(group),
                        )>
                            <div class="studio-palette__group-label">{group.label()}</div>
                            {entries
                                .into_iter()
                                .map(|item| {
                                    let payload = palette_payload(&item);
                                    let click_payload = payload.clone();
                                    view! {
                                        <button
                                            type="button"
                                            class="studio-palette__item"
                                            draggable="true"
                                            title=item.hint
                                            on:dragstart=move |ev: web_sys::DragEvent| {
                                                dom::drag_set_data(&ev, HONBA_NODE_MIME, &payload);
                                            }
                                            on:click=move |_| {
                                                if let Some(parsed) = parse_palette_payload(
                                                    &click_payload,
                                                ) {
                                                    store.add_node_visible(palette_node(&parsed));
                                                }
                                            }
                                        >
                                            <span class="studio-palette__dot"></span>
                                            <span class="studio-palette__label">{item.label}</span>
                                        </button>
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                })
                .collect_view()}
        </aside>
    }
}

/// The CSS modifier slug of a category (`studio-cat--indicator`, …).
pub const fn category_slug(category: NodeCategory) -> &'static str {
    match category {
        NodeCategory::Data => "data",
        NodeCategory::Indicator => "indicator",
        NodeCategory::Logic => "logic",
        NodeCategory::Risk => "risk",
        NodeCategory::Execution => "execution",
    }
}
