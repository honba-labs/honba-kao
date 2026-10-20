//! The pane drawing rail — the vertical TradingView-style toolbar floating
//! over a workbench chart (cursor, trend line, rays, shapes, measure, undo /
//! redo / trash).
//!
//! The tools live in the vendored engine (`DrawingController`, armed via the
//! bridge's `setDrawingTool`), and placements persist per symbol in
//! localStorage through the engine's `InstrumentDrawings` store — the rail is
//! only state-keeping: which tool is pressed, and undo/redo/clear plumbing.
//!
//! The pressed state stays honest: the engine broadcasts `honba-drawing-tool`
//! on every tool change (including the placement reset and Esc), and the rail
//! listens so its highlight follows the real active tool.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use crate::shared::components::charts::openalgo_chart as bridge;

/// One rail entry — `id` either the engine tool id (`""` = cursor) or the
/// action name (`undo` / `redo` / `trash`).
struct RailEntry {
    id: &'static str,
    title: &'static str,
    /// SVG path data in a 16×16 box (stroke style, see the CSS).
    path: &'static str,
}

/// Tool buttons, rendered top-down.
const RAIL_TOOLS: [RailEntry; 10] = [
    RailEntry {
        id: "",
        title: "Cursor",
        path: "M5 2l7 6.2-3.7.4 2 3.9-1.8.9-2-3.9L5 11.7z",
    },
    RailEntry {
        id: "trend-line",
        title: "Trend line",
        path: "M4.5 11.5L11.5 4.5M4.5 10.6a1.1 1.1 0 1 0 0 2.2 1.1 1.1 0 0 0 0-2.2M11.5 2.6a1.1 1.1 0 1 0 0 2.2 1.1 1.1 0 0 0 0-2.2",
    },
    RailEntry {
        id: "horizontal-line",
        title: "Horizontal line",
        path: "M2 8h12M7 8a1 1 0 1 0 2 0a1 1 0 1 0-2 0",
    },
    RailEntry {
        id: "vertical-line",
        title: "Vertical line",
        path: "M8 2v12M8 7a1 1 0 1 0 0 2a1 1 0 1 0 0-2",
    },
    RailEntry {
        id: "ray",
        title: "Ray",
        path: "M3 12L13 4M3 12a1.1 1.1 0 1 0 0-2.2a1.1 1.1 0 0 0 0 2.2",
    },
    RailEntry {
        id: "parallel-channel",
        title: "Parallel channel",
        path: "M3 4.5l10 6M3 8.5l10 6",
    },
    RailEntry {
        id: "rectangle",
        title: "Rectangle",
        path: "M3 4h10v8H3z",
    },
    RailEntry {
        id: "ellipse",
        title: "Ellipse",
        path: "M3 8a5 4 0 1 0 10 0a5 4 0 1 0-10 0",
    },
    RailEntry {
        id: "measure",
        title: "Measure",
        path: "M2 10.5L10.5 2l3.5 3.5L5.5 14zM6 6.5l2 2M8.5 4l2 2M3.5 9l2 2",
    },
    RailEntry {
        id: "text",
        title: "Text",
        path: "M4 4h8M8 4v9",
    },
];

/// Action buttons under the separator.
const RAIL_ACTIONS: [RailEntry; 3] = [
    RailEntry {
        id: "undo",
        title: "Undo",
        path: "M6 4L3 7l3 3M3 7h6a4 4 0 0 1 0 7h-1",
    },
    RailEntry {
        id: "redo",
        title: "Redo",
        path: "M10 4l3 3-3 3M13 7H7a4 4 0 0 0-4 4a4 4 0 0 0 4 4h1",
    },
    RailEntry {
        id: "trash",
        title: "Remove all drawings",
        path: "M3 4.5h10M6 4.5V3h4v1.5M4.5 4.5l.8 8.5h5.4l.8-8.5M6.5 6.5v5M9.5 6.5v5",
    },
];

/// The engine tool id for a broadcast (`None` = cursor / unknown), used to
/// keep the rail's pressed highlight in step with the engine.
fn rail_tool_of(engine_tool: &str) -> Option<&'static str> {
    RAIL_TOOLS
        .iter()
        .find(|entry| entry.id == engine_tool)
        .map(|entry| entry.id)
}

/// The drawing rail for one pane's chart.
#[component]
pub(crate) fn DrawingRail(chart_handle: RwSignal<Option<f64>>) -> impl IntoView {
    // The armed tool id (`None` = cursor). The engine resets it after every
    // placement — the `honba-drawing-tool` streamer below keeps this in step.
    let pressed = RwSignal::new(None::<&'static str>);

    // Engine → rail sync: after a placement or Esc the controller clears its
    // tool and broadcasts; the rail drops its highlight to match. (The event
    // carries the bridge chart id — stale panes are ignored.)
    let streamer = Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
        let Some(id) = chart_handle.get_untracked() else {
            return;
        };
        let Ok(event) = event.dyn_into::<web_sys::CustomEvent>() else {
            return;
        };
        let detail = event.detail();
        let field = |name: &str| js_sys::Reflect::get(&detail, &JsValue::from(name)).ok();
        let chart_id = field("chartId").and_then(|value| value.as_f64());
        if chart_id != Some(id) {
            return;
        }
        let tool = field("tool")
            .and_then(|value| value.as_string())
            .unwrap_or_default();
        pressed.set(rail_tool_of(&tool));
    });

    if let Some(window) = web_sys::window() {
        let _ = window.add_event_listener_with_callback(
            "honba-drawing-tool",
            streamer.as_ref().unchecked_ref::<js_sys::Function>(),
        );
    }
    let streamer = StoredValue::new_local(streamer);
    on_cleanup(move || {
        if let Some(window) = web_sys::window() {
            streamer.with_value(|closure| {
                let _ = window.remove_event_listener_with_callback(
                    "honba-drawing-tool",
                    closure.as_ref().unchecked_ref::<js_sys::Function>(),
                );
            });
        }
        streamer.dispose();
    });

    let pick = move |engine_id: &'static str| {
        if let Some(id) = chart_handle.get_untracked() {
            if bridge::set_drawing_tool(id, engine_id) {
                pressed.set(if engine_id.is_empty() {
                    None
                } else {
                    Some(engine_id)
                });
            }
        }
    };

    let run = move |action: &'static str| {
        if let Some(id) = chart_handle.get_untracked() {
            match action {
                "undo" => bridge::undo_drawing(id),
                "redo" => bridge::redo_drawing(id),
                _ => bridge::clear_drawings(id),
            }
        }
    };

    view! {
        <div class="wb-draw-rail" role="toolbar" aria-label="Drawing tools">
            {RAIL_TOOLS
                .iter()
                .map(|entry| {
                    let id = entry.id;
                    let title = entry.title;
                    let path = entry.path;
                    view! {
                        <button
                            class=move || {
                                if pressed.get() == Some(id) {
                                    "wb-draw-rail__btn wb-draw-rail__btn--on"
                                } else {
                                    "wb-draw-rail__btn"
                                }
                            }
                            title=title
                            aria-label=title
                            on:click=move |_| pick(id)
                        >
                            <svg viewBox="0 0 16 16"><path d=path /></svg>
                        </button>
                    }
                })
                .collect_view()}
            <span class="wb-draw-rail__sep" />
            {RAIL_ACTIONS
                .iter()
                .map(|entry| {
                    let id = entry.id;
                    let title = entry.title;
                    let path = entry.path;
                    view! {
                        <button
                            class="wb-draw-rail__btn"
                            title=title
                            aria-label=title
                            on:click=move |_| run(id)
                        >
                            <svg viewBox="0 0 16 16"><path d=path /></svg>
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}
