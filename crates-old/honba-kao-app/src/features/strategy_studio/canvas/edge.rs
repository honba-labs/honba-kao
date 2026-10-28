//! Insertable edge (HBD-2026-DES-006 P2) — port of honba-fe algodesigner
//! `edges/HonbaInsertableEdge.tsx`.
//!
//! A smoothstep path (routed in [`super::geometry::smoothstep_path`]) with a
//! wide invisible hit path for selection and a midpoint "+" control. The
//! "+" opens a splice menu filtered by port kind; choosing an entry replaces
//! `a → b` with `a → new → b` (store `splice`).
//!
//! Deviation from fe: fe filtered only by the *source* kind, so "Buy Order"
//! was offered on every bool edge and splicing it orphaned the downstream
//! input (an `order` output cannot feed a `bool` port). Here an option is
//! offered only when its ports type-check on **both** sides.

use leptos::prelude::*;

use crate::features::simulator::kernel::selection::{StrategyGraphNode, StrategyGraphShape};
use crate::features::strategy_studio::canvas::editor::CanvasCtx;
use crate::features::strategy_studio::canvas::palette::{PalettePayload, palette_node};
use crate::features::strategy_studio::canvas::scene::{RenderEdge, Selection};
use crate::features::strategy_studio::kernel::edit::{input_kind, output_kind, splice_fits};

/// One splice-menu entry (fe `SplicableOption`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpliceOption {
    /// Menu label.
    pub label: &'static str,
    /// Node type to insert.
    pub node_type: &'static str,
    /// Indicator kind, for indicator entries.
    pub indicator: Option<&'static str>,
    /// `op` param override (gates / transforms).
    pub op: Option<&'static str>,
    /// `side` param override (orders).
    pub side: Option<&'static str>,
    /// The inserted node's input port the edge's source feeds.
    pub in_port: &'static str,
    /// The inserted node's output port that feeds the edge's target.
    pub out_port: &'static str,
}

const fn option(
    label: &'static str,
    node_type: &'static str,
    indicator: Option<&'static str>,
    op: Option<&'static str>,
    side: Option<&'static str>,
    in_port: &'static str,
    out_port: &'static str,
) -> SpliceOption {
    SpliceOption {
        label,
        node_type,
        indicator,
        op,
        side,
        in_port,
        out_port,
    }
}

/// fe's `SERIES_SPLICE_OPTIONS` + `BOOL_SPLICE_OPTIONS`, in fe's order.
pub const SPLICE_OPTIONS: &[SpliceOption] = &[
    option(
        "SMA Indicator",
        "indicator",
        Some("sma"),
        None,
        None,
        "input",
        "value",
    ),
    option(
        "EMA Indicator",
        "indicator",
        Some("ema"),
        None,
        None,
        "input",
        "value",
    ),
    option(
        "RSI Indicator",
        "indicator",
        Some("rsi"),
        None,
        None,
        "input",
        "value",
    ),
    option(
        "Pct Change",
        "transform",
        None,
        Some("pct_change"),
        None,
        "input",
        "value",
    ),
    option("AND Gate", "logic", None, Some("and"), None, "a", "signal"),
    option("OR Gate", "logic", None, Some("or"), None, "a", "signal"),
    option("Risk Gate", "risk", None, None, None, "signal", "signal"),
    option(
        "Position Sizing",
        "sizing",
        None,
        None,
        None,
        "signal",
        "size",
    ),
    option(
        "Buy Order",
        "order",
        None,
        None,
        Some("buy"),
        "signal",
        "orders",
    ),
    option(
        "Sell Order",
        "order",
        None,
        None,
        Some("sell"),
        "signal",
        "orders",
    ),
];

/// The node a splice option inserts, with catalogue / fe defaults.
pub fn splice_node(option: &SpliceOption) -> StrategyGraphNode {
    let mut node = palette_node(&PalettePayload {
        node_type: option.node_type.to_string(),
        indicator: option.indicator.map(str::to_string),
    });
    if let Some(op) = option.op {
        node.params.insert("op".to_string(), op.into());
    }
    if let Some(side) = option.side {
        node.params.insert("side".to_string(), side.into());
    }
    node.label = Some(option.label.trim_end_matches(" Indicator").to_string());
    node
}

/// The options that type-check on both sides of `edge_id`; empty when the
/// edge is unknown or its own ports are broken.
pub fn splice_options_for(graph: &StrategyGraphShape, edge_id: &str) -> Vec<SpliceOption> {
    let Some(edge) = graph.edges.iter().find(|e| e.id == edge_id) else {
        return Vec::new();
    };
    let (Some(edge_kind), Some(target_kind)) = (
        output_kind(graph, &edge.from_node, &edge.from_port),
        input_kind(graph, &edge.to_node, &edge.to_port),
    ) else {
        return Vec::new();
    };
    SPLICE_OPTIONS
        .iter()
        .filter(|option| {
            splice_fits(
                &splice_node(option),
                option.in_port,
                option.out_port,
                edge_kind,
                target_kind,
            )
        })
        .copied()
        .collect()
}

/// One routed edge with its hit path and "+" splice control.
#[component]
pub fn EdgeView(
    /// The routed edge from the scene memo.
    edge: RenderEdge,
    /// Canvas context.
    ctx: CanvasCtx,
) -> impl IntoView {
    let store = ctx.store;
    let kind = edge.kind.map(|k| k.as_str()).unwrap_or("unknown");
    let class = format!(
        "studio-edge studio-kind--{kind}{}",
        if edge.selected {
            " studio-edge--selected"
        } else {
            ""
        }
    );
    let select_id = edge.id.clone();
    let menu_id = edge.id.clone();
    let open_id = edge.id.clone();
    view! {
        <g class=class>
            <path
                class="studio-edge__hit"
                d=edge.d.clone()
                on:mousedown=move |ev: web_sys::MouseEvent| {
                    if ev.button() != 0 {
                        return;
                    }
                    ev.stop_propagation();
                    store.select(Selection::Edge(select_id.clone()));
                }
            ></path>
            <path class="studio-edge__line" d=edge.d></path>
            <g
                class=move || {
                    if store.splice_menu.with(|m| m.as_deref() == Some(open_id.as_str())) {
                        "studio-edge__plus studio-edge__plus--open"
                    } else {
                        "studio-edge__plus"
                    }
                }
                transform=format!("translate({:.1},{:.1})", edge.mid_x, edge.mid_y)
                on:mousedown=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                on:click=move |ev: web_sys::MouseEvent| {
                    ev.stop_propagation();
                    let open = store
                        .splice_menu
                        .with_untracked(|m| m.as_deref() == Some(menu_id.as_str()));
                    store.splice_menu.set(if open { None } else { Some(menu_id.clone()) });
                }
            >
                <circle r="9"></circle>
                <text class="studio-edge__plus-glyph" y="4" text-anchor="middle">
                    "+"
                </text>
            </g>
        </g>
    }
}

/// The splice menu for the open edge, as an HTML overlay positioned at the
/// edge's "+" in screen space.
#[component]
pub fn SpliceMenu(
    /// Canvas context.
    ctx: CanvasCtx,
    /// The edge being split.
    edge_id: String,
    /// Screen-space anchor (canvas-local px).
    left: f64,
    /// Screen-space anchor (canvas-local px).
    top: f64,
) -> impl IntoView {
    let store = ctx.store;
    let options = store
        .graph
        .with_untracked(|g| splice_options_for(g, &edge_id));
    let empty = options.is_empty();
    view! {
        <div
            class="studio-splice"
            style=format!("left:{left:.0}px;top:{top:.0}px")
            on:mousedown=move |ev: web_sys::MouseEvent| ev.stop_propagation()
        >
            <div class="studio-splice__title">"Insert between nodes"</div>
            {options
                .into_iter()
                .map(|option| {
                    let edge_id = edge_id.clone();
                    view! {
                        <button
                            type="button"
                            class="studio-splice__item"
                            on:click=move |_| {
                                store
                                    .splice(
                                        &edge_id,
                                        splice_node(&option),
                                        option.in_port,
                                        option.out_port,
                                    );
                            }
                        >
                            {option.label}
                        </button>
                    }
                })
                .collect_view()}
            <Show when=move || empty>
                <div class="studio-splice__empty">"Nothing type-checks on this connection."</div>
            </Show>
        </div>
    }
}
