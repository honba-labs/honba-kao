//! Node card (HBD-2026-DES-006 P2) — port of honba-fe algodesigner
//! `nodes/BaseHonbaNode.tsx` + `HonbaNodes.tsx`, as one generic SVG
//! component.
//!
//! A 224px card: category accent along the top border (the
//! `--studio-cat-*` CSS variables via a `studio-cat--*` class — no colour
//! literal in Rust), title + `type · indicator` subtitle, an issue badge
//! counting the validation issues naming the node, per-kind body chips, and
//! named port rows with typed colour dots (inputs on the left edge, outputs
//! on the right). Dragging the card moves it; dragging from an output dot
//! and releasing on an input dot connects them.

use leptos::prelude::*;

use crate::features::strategy_studio::canvas::editor::CanvasCtx;
use crate::features::strategy_studio::canvas::geometry::{CHIPS_HEIGHT, HEADER_HEIGHT};
use crate::features::strategy_studio::canvas::palette::category_slug;
use crate::features::strategy_studio::canvas::scene::RenderNode;

/// Approximate glyph advance for chip sizing (11px UI font).
const CHIP_CHAR_WIDTH: f64 = 6.2;
/// Chip horizontal padding.
const CHIP_PADDING: f64 = 6.0;

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

/// One node card from the scene memo.
#[component]
pub fn NodeCard(
    /// The placed node.
    node: RenderNode,
    /// Canvas context.
    ctx: CanvasCtx,
) -> impl IntoView {
    let width = node.width;
    let class = format!(
        "studio-node studio-cat--{}{}{}",
        category_slug(node.category),
        if node.selected { " studio-node--selected" } else { "" },
        if node.issues.is_empty() { "" } else { " studio-node--invalid" },
    );
    let drag_id = node.id.clone();
    let issue_count = node.issues.len();

    // Chips, laid out left to right and dropped once they'd overflow.
    let mut chip_x = 10.0;
    let chips: Vec<(f64, f64, String)> = node
        .chips
        .iter()
        .filter_map(|chip| {
            let text = truncate(chip, 18);
            let w = text.chars().count() as f64 * CHIP_CHAR_WIDTH + 2.0 * CHIP_PADDING;
            if chip_x + w > width - 10.0 {
                return None;
            }
            let x = chip_x;
            chip_x += w + 4.0;
            Some((x, w, text))
        })
        .collect();

    // Ports carry absolute anchors; the card draws in its own frame.
    let oy = node.y;
    let inputs = node.inputs.clone();
    let outputs = node.outputs.clone();
    let node_id = node.id.clone();

    view! {
        <g
            class=class
            transform=format!("translate({:.1},{:.1})", node.x, node.y)
            on:mousedown=move |ev: web_sys::MouseEvent| {
                if ev.button() != 0 {
                    return;
                }
                ev.stop_propagation();
                ctx.begin_node_drag(&drag_id, &ev);
            }
        >
            <rect class="studio-node__body" width=width height=node.height rx="6"></rect>
            <rect class="studio-node__accent" width=width height="3" rx="1.5"></rect>
            <text class="studio-node__title" x="12" y="20">
                {truncate(&node.title, 24)}
            </text>
            <text class="studio-node__subtitle" x="12" y="34">
                {truncate(&node.subtitle, 30)}
            </text>
            {(issue_count > 0)
                .then(|| {
                    view! {
                        <g
                            class="studio-node__issue"
                            transform=format!("translate({:.1},16)", width - 16.0)
                        >
                            <circle r="8"></circle>
                            <text y="4" text-anchor="middle">
                                {issue_count.to_string()}
                            </text>
                        </g>
                    }
                })}
            <g class="studio-node__chips">
                {chips
                    .into_iter()
                    .map(|(x, w, text)| {
                        view! {
                            <rect
                                class="studio-node__chip"
                                x=x
                                y={HEADER_HEIGHT + 2.0}
                                width=w
                                height={CHIPS_HEIGHT - 6.0}
                                rx="3"
                            ></rect>
                            <text
                                class="studio-node__chip-text"
                                x={x + CHIP_PADDING}
                                y={HEADER_HEIGHT + CHIPS_HEIGHT - 8.0}
                            >
                                {text}
                            </text>
                        }
                    })
                    .collect_view()}
            </g>
            {inputs
                .into_iter()
                .map(|port| {
                    let ly = port.y - oy;
                    let to_id = node_id.clone();
                    let port_id = port.id;
                    let kind = port.kind;
                    let wire = ctx.wire;
                    view! {
                        <g
                            class=move || {
                                let target = wire
                                    .with(|w| w.as_ref().is_some_and(|w| w.kind == kind));
                                format!(
                                    "studio-port studio-port--in studio-kind--{}{}{}",
                                    kind.as_str(),
                                    if port.connected { " studio-port--connected" } else { "" },
                                    if target { " studio-port--target" } else { "" },
                                )
                            }
                            on:mouseup=move |_| ctx.finish_connect(&to_id, port_id)
                        >
                            <circle class="studio-port__hit" cx="0" cy=ly r="10"></circle>
                            <circle class="studio-port__dot" cx="0" cy=ly r="5"></circle>
                            <text class="studio-port__label" x="11" y={ly + 4.0}>
                                {port.id}
                            </text>
                        </g>
                    }
                })
                .collect_view()}
            {outputs
                .into_iter()
                .map(|port| {
                    let ly = port.y - oy;
                    let from_id = node_id.clone();
                    let port_id = port.id;
                    let kind = port.kind;
                    let anchor = (port.x, port.y);
                    view! {
                        <g
                            class=format!(
                                "studio-port studio-port--out studio-kind--{}{}",
                                kind.as_str(),
                                if port.connected { " studio-port--connected" } else { "" },
                            )
                            on:mousedown=move |ev: web_sys::MouseEvent| {
                                if ev.button() != 0 {
                                    return;
                                }
                                ev.stop_propagation();
                                ctx.begin_connect(&from_id, port_id, kind, anchor);
                            }
                        >
                            <circle class="studio-port__hit" cx=width cy=ly r="10"></circle>
                            <circle class="studio-port__dot" cx=width cy=ly r="5"></circle>
                            <text
                                class="studio-port__label"
                                x={width - 11.0}
                                y={ly + 4.0}
                                text-anchor="end"
                            >
                                {port.id}
                            </text>
                        </g>
                    }
                })
                .collect_view()}
        </g>
    }
}
