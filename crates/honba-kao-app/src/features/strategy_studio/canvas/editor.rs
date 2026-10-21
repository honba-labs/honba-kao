//! Canvas editor (HBD-2026-DES-006 P2) — port of honba-fe algodesigner
//! `components/DesignerCanvas.tsx`, as a custom Leptos SVG viewport (no
//! xyflow).
//!
//! Gestures, each landing on a [`StudioStore`] action:
//! - **pan**: drag the background; **zoom**: mouse wheel (anchored under
//!   the cursor), plus the `+ / − / Fit` buttons;
//! - **select**: press a card or an edge; press the background to clear;
//! - **move**: drag a card — positions snap to the 16px grid;
//! - **connect**: drag from an output dot, release on an input dot of the
//!   same kind (kind, self-loop and cycle refusals show as a notice);
//! - **splice**: the "+" on any edge (see [`super::edge`]);
//! - **drop**: palette items under the `application/honba-node` MIME type;
//! - **keys**: `Escape` clears the selection first, then closes the splice
//!   menu / notice; `Delete`/`Backspace` removes the selected card or edge.
//!   Neither ever fires while focus is in an INPUT/TEXTAREA/SELECT or a
//!   contenteditable (fe's guard).
//!
//! Rendering follows the design doc's rule: one `Memo<Scene>` built by
//! [`build_scene`] per change, no per-node or per-port effects. Pan/zoom is
//! a plain SVG `translate(..) scale(..)` on the world group in user units —
//! no device-pixel-ratio handling. Multi-select (rubber band) is out of
//! scope, as in fe.

use leptos::prelude::*;
use leptos::svg::Svg;

use crate::features::strategy_studio::canvas::dom;
use crate::features::strategy_studio::canvas::edge::{EdgeView, SpliceMenu};
use crate::features::strategy_studio::canvas::geometry::{
    smoothstep_path, snap_position, Position, Viewport, GRID, NODE_WIDTH,
};
use crate::features::strategy_studio::canvas::node_card::NodeCard;
use crate::features::strategy_studio::canvas::palette::{
    palette_node, parse_palette_payload, HONBA_NODE_MIME,
};
use crate::features::strategy_studio::canvas::scene::{build_scene, Scene, Selection};
use crate::features::strategy_studio::kernel::graph::PortKind;
use crate::features::strategy_studio::stores::StudioStore;

/// The in-flight pointer gesture (non-reactive; lives in a `StoredValue`).
#[derive(Clone, Debug, PartialEq)]
pub enum DragState {
    /// Panning: the client point the drag started at and the viewport then.
    Pan {
        /// Client-space start.
        start: (f64, f64),
        /// Viewport at the start.
        origin: Viewport,
    },
    /// Moving one card.
    Node {
        /// The card's node id.
        id: String,
        /// Client-space start.
        start: (f64, f64),
        /// The card's position at the start.
        origin: Position,
    },
    /// Dragging a wire out of an output port.
    Connect {
        /// Source node id.
        node: String,
        /// Source port id.
        port: &'static str,
    },
}

/// The provisional wire drawn while connecting, in world units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wire {
    /// Output anchor.
    pub from: (f64, f64),
    /// Cursor.
    pub to: (f64, f64),
    /// The source port's kind (compatible inputs highlight).
    pub kind: PortKind,
}

/// What the canvas children (cards, edges, menus) share. `Copy`.
#[derive(Clone, Copy)]
pub struct CanvasCtx {
    /// The studio store.
    pub store: StudioStore,
    /// The in-flight gesture.
    pub drag: StoredValue<Option<DragState>>,
    /// The provisional wire, while connecting.
    pub wire: RwSignal<Option<Wire>>,
    /// The `<svg>` element (for client → canvas-local coordinates).
    pub svg: NodeRef<Svg>,
}

impl CanvasCtx {
    /// The svg's client rect `(left, top, width, height)`.
    fn rect(&self) -> Option<(f64, f64, f64, f64)> {
        let svg = self.svg.get_untracked()?;
        dom::client_rect(&svg)
    }

    /// Client point → canvas-local px.
    fn local(&self, ev: &web_sys::MouseEvent) -> Option<(f64, f64)> {
        let (left, top, _, _) = self.rect()?;
        Some((f64::from(ev.client_x()) - left, f64::from(ev.client_y()) - top))
    }

    /// Client point → world units.
    fn world(&self, ev: &web_sys::MouseEvent) -> Option<(f64, f64)> {
        let (lx, ly) = self.local(ev)?;
        Some(self.store.viewport.get_untracked().to_world(lx, ly))
    }

    /// A card was pressed: select it and start a move.
    pub fn begin_node_drag(&self, id: &str, ev: &web_sys::MouseEvent) {
        self.store.select(Selection::Node(id.to_string()));
        let origin = self
            .store
            .positions
            .with_untracked(|p| p.get(id).copied())
            .unwrap_or(Position::new(0.0, 0.0));
        self.drag.set_value(Some(DragState::Node {
            id: id.to_string(),
            start: (f64::from(ev.client_x()), f64::from(ev.client_y())),
            origin,
        }));
    }

    /// An output dot was pressed: start a wire.
    pub fn begin_connect(&self, node: &str, port: &'static str, kind: PortKind, anchor: (f64, f64)) {
        self.store.select(Selection::None);
        self.drag
            .set_value(Some(DragState::Connect { node: node.to_string(), port }));
        self.wire.set(Some(Wire { from: anchor, to: anchor, kind }));
    }

    /// The pointer was released on an input dot: connect when a wire is
    /// in flight (the store refuses mismatches with a notice).
    pub fn finish_connect(&self, to_node: &str, to_port: &'static str) {
        if let Some(DragState::Connect { node, port }) = self.drag.get_value() {
            self.store.connect(&node, port, to_node, to_port);
        }
    }

    /// Ends any gesture.
    fn end_drag(&self) {
        self.drag.set_value(None);
        if self.wire.get_untracked().is_some() {
            self.wire.set(None);
        }
    }

    /// Escape while wiring cancels the wire. Returns whether it did.
    fn cancel_connect(&self) -> bool {
        if self.wire.get_untracked().is_some() {
            self.end_drag();
            true
        } else {
            false
        }
    }

    /// Zooms about the canvas centre.
    fn zoom_centre(&self, factor: f64) {
        let (w, h) = self.rect().map(|(_, _, w, h)| (w, h)).unwrap_or((800.0, 500.0));
        self.store
            .viewport
            .update(|vp| *vp = vp.zoom_at(w / 2.0, h / 2.0, factor));
    }

    /// Fits every card into view.
    fn fit(&self, scene: &Scene) {
        if let Some((_, _, w, h)) = self.rect() {
            self.store
                .viewport
                .set(Viewport::fit(&scene.positions(), &scene.heights(), w, h));
        }
    }
}

/// The node-graph canvas.
#[component]
pub fn CanvasEditor(
    /// The studio store.
    store: StudioStore,
) -> impl IntoView {
    let svg = NodeRef::<Svg>::new();
    let ctx = CanvasCtx {
        store,
        drag: StoredValue::new(None),
        wire: RwSignal::new(None),
        svg,
    };

    // The single render pass.
    let scene = Memo::new(move |_| {
        let selection = store.selection.get();
        store.issues.with(|issues| {
            store.graph.with(|graph| {
                store
                    .positions
                    .with(|positions| build_scene(graph, positions, &selection, issues))
            })
        })
    });

    // Fit once the svg is mounted (each open remounts the editor).
    Effect::new(move |_| {
        if svg.get().is_some() {
            request_animation_frame(move || scene.with_untracked(|s| ctx.fit(s)));
        }
    });

    // Keyboard: Escape / Delete / Backspace, guarded against editable
    // controls so typing in the inspector never deletes a node.
    let keys = window_event_listener(leptos::ev::keydown, move |ev: web_sys::KeyboardEvent| {
        if !store.open.get_untracked() || dom::is_editable_target(&ev) {
            return;
        }
        match ev.key().as_str() {
            "Escape" => {
                if ctx.cancel_connect() || store.escape() {
                    ev.prevent_default();
                }
            }
            "Delete" | "Backspace" if store.delete_selection() => ev.prevent_default(),
            _ => {}
        }
    });
    on_cleanup(move || keys.remove());

    // -- Pointer ------------------------------------------------------------
    let on_mousedown = move |ev: web_sys::MouseEvent| {
        if ev.button() != 0 {
            return;
        }
        // Cards, ports, edges and the "+" stop propagation: reaching the
        // svg means the background was pressed.
        store.select(Selection::None);
        ctx.drag.set_value(Some(DragState::Pan {
            start: (f64::from(ev.client_x()), f64::from(ev.client_y())),
            origin: store.viewport.get_untracked(),
        }));
    };
    let on_mousemove = move |ev: web_sys::MouseEvent| {
        let Some(drag) = ctx.drag.get_value() else {
            return;
        };
        let (cx, cy) = (f64::from(ev.client_x()), f64::from(ev.client_y()));
        match drag {
            DragState::Pan { start, origin } => {
                store.viewport.set(Viewport {
                    x: origin.x + (cx - start.0),
                    y: origin.y + (cy - start.1),
                    zoom: origin.zoom,
                });
            }
            DragState::Node { id, start, origin } => {
                let zoom = store.viewport.get_untracked().zoom;
                let next = snap_position(Position::new(
                    origin.x + (cx - start.0) / zoom,
                    origin.y + (cy - start.1) / zoom,
                ));
                store.move_node(&id, next);
            }
            DragState::Connect { .. } => {
                if let Some(to) = ctx.world(&ev) {
                    ctx.wire.update(|wire| {
                        if let Some(wire) = wire {
                            wire.to = to;
                        }
                    });
                }
            }
        }
    };
    let on_mouseup = move |_: web_sys::MouseEvent| ctx.end_drag();
    let on_mouseleave = move |_: web_sys::MouseEvent| ctx.end_drag();
    let on_wheel = move |ev: web_sys::WheelEvent| {
        ev.prevent_default();
        let Some((lx, ly)) = ctx.local(&ev) else {
            return;
        };
        let factor = (-ev.delta_y() * 0.0015).exp();
        store.viewport.update(|vp| *vp = vp.zoom_at(lx, ly, factor));
    };

    // -- Palette drop ------------------------------------------------------
    let on_dragover = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        dom::drag_set_drop_effect(&ev);
    };
    let on_drop = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        let Some(payload) = dom::drag_get_data(&ev, HONBA_NODE_MIME)
            .and_then(|data| parse_palette_payload(&data))
        else {
            store
                .notice
                .set(Some("That drop is not a Honba palette node.".to_string()));
            return;
        };
        let (wx, wy) = ctx.world(&ev).unwrap_or((GRID * 4.0, GRID * 4.0));
        store.add_node_at(
            palette_node(&payload),
            Position::new(wx - NODE_WIDTH / 2.0, wy - GRID),
        );
    };

    // -- Background grid follows pan/zoom (CSS dots, no per-dot SVG) -------
    let grid_style = move || {
        let vp = store.viewport.get();
        let gap = GRID * vp.zoom;
        format!(
            "background-size:{gap:.2}px {gap:.2}px;background-position:{:.2}px {:.2}px",
            vp.x, vp.y
        )
    };

    let empty = Memo::new(move |_| store.graph.with(|g| g.nodes.is_empty()));

    view! {
        <div class="studio-canvas" style=grid_style>
            <svg
                node_ref=svg
                class="studio-canvas__svg"
                on:mousedown=on_mousedown
                on:mousemove=on_mousemove
                on:mouseup=on_mouseup
                on:mouseleave=on_mouseleave
                on:wheel=on_wheel
                on:dragover=on_dragover
                on:drop=on_drop
            >
                <g transform=move || store.viewport.get().transform()>
                    <g class="studio-edges">
                        {move || {
                            scene
                                .with(|s| {
                                    s.edges
                                        .iter()
                                        .map(|edge| view! { <EdgeView edge=edge.clone() ctx=ctx /> })
                                        .collect_view()
                                })
                        }}
                    </g>
                    {move || {
                        ctx.wire
                            .get()
                            .map(|wire| {
                                let (d, _, _) = smoothstep_path(wire.from, wire.to);
                                view! {
                                    <path
                                        class=format!("studio-wire studio-kind--{}", wire.kind.as_str())
                                        d=d
                                    ></path>
                                }
                            })
                    }}
                    <g class="studio-nodes">
                        {move || {
                            scene
                                .with(|s| {
                                    s.nodes
                                        .iter()
                                        .map(|node| view! { <NodeCard node=node.clone() ctx=ctx /> })
                                        .collect_view()
                                })
                        }}
                    </g>
                </g>
            </svg>

            // Splice menu, placed over the open edge's "+" in screen space.
            {move || {
                let edge_id = store.splice_menu.get()?;
                let vp = store.viewport.get();
                let (mx, my) = scene
                    .with(|s| s.edges.iter().find(|e| e.id == edge_id).map(|e| (e.mid_x, e.mid_y)))?;
                let (left, top) = vp.to_screen(mx, my);
                Some(view! { <SpliceMenu ctx=ctx edge_id=edge_id left=left top={top + 14.0} /> })
            }}

            <div class="studio-canvas__controls">
                <button type="button" title="Zoom in" on:click=move |_| ctx.zoom_centre(1.2)>
                    "+"
                </button>
                <button type="button" title="Zoom out" on:click=move |_| ctx.zoom_centre(1.0 / 1.2)>
                    "−"
                </button>
                <button type="button" title="Fit to view" on:click=move |_| scene.with_untracked(|s| ctx.fit(s))>
                    "Fit"
                </button>
                <button type="button" title="Re-run the auto-layout" on:click=move |_| store.auto_layout()>
                    "Layout"
                </button>
                <span class="studio-canvas__zoom tnum">
                    {move || format!("{:.0}%", store.viewport.get().zoom * 100.0)}
                </span>
            </div>

            <Show when=move || empty.get()>
                <div class="studio-canvas__empty">
                    "Drag nodes from the palette, or click one to add it."
                </div>
            </Show>

            {move || {
                store
                    .notice
                    .get()
                    .map(|message| {
                        view! {
                            <div class="studio-canvas__notice" role="status">
                                <span>{message}</span>
                                <button
                                    type="button"
                                    title="Dismiss"
                                    on:click=move |_| store.notice.set(None)
                                >
                                    "×"
                                </button>
                            </div>
                        }
                    })
            }}

            <div class="studio-canvas__hint">
                "Drag background to pan · wheel to zoom · drag output → input to connect · Del removes · Esc clears (single selection only)"
            </div>
        </div>
    }
}
