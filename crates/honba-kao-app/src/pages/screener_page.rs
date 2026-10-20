//! Screener / market view — the Leptos composition of the labs app's
//! `ScreenerPage.tsx` + `DockableWorkspace`: the filter bar, the
//! table/heatmap view switch inside the split workspace, the symbol
//! detail drawer docked on the right and the three configuration dialogs
//! (screen catalog, filters, columns), all over the page-scoped
//! `ScreenerStore`.
//!
//! The labs hotkeys are mirrored here: `.` opens the screen catalog and
//! `Shift+N` saves the current view as a new custom screen (ignored while
//! typing in inputs).

use leptos::prelude::*;

use crate::components::screener::column_modal::ColumnModal;
use crate::components::screener::drawers::SymbolDetailDrawer;
use crate::components::screener::filter_bar::FilterBar;
use crate::components::screener::filters_modal::FiltersModal;
use crate::components::screener::heatmap::ScreenerHeatmap;
use crate::components::screener::open_screen_modal::OpenScreenModal;
use crate::components::screener::screener_table::ScreenerTable;
use crate::stores::screener_store::{ScreenerStore, ViewMode};
use wasm_bindgen::prelude::*;

#[component]
pub fn ScreenerPage() -> impl IntoView {
    // The screener is page-scoped: every child `take()`s this instance.
    let store = ScreenerStore::new();
    store.provide();

    // -- Hotkeys --------------------------------------------------------------
    // `.` opens the screen catalog, `Shift+N` prompts a new screen name.
    // Mirrors the labs `keydown` listener while typing is ignored.
    Effect::new(move |_| {
        let handle = window_event_listener_untyped("keydown", move |event: web_sys::Event| {
            let key_event = event.unchecked_ref::<web_sys::KeyboardEvent>();
            let element = event
                .target()
                .and_then(|target| target.clone().dyn_into::<web_sys::HtmlElement>().ok());
            let typing = element
                .map(|element| {
                    matches!(element.tag_name().as_str(), "INPUT" | "TEXTAREA" | "SELECT")
                        || element.is_content_editable()
                })
                .unwrap_or(false);
            if typing || key_event.ctrl_key() || key_event.meta_key() || key_event.alt_key() {
                return;
            }
            if key_event.key() == "." && !key_event.shift_key() {
                event.prevent_default();
                store.toggle_open_screen_modal(true);
            } else if key_event.key() == "N" && key_event.shift_key() {
                event.prevent_default();
                if let Some(window) = web_sys::window() {
                    let name = window
                        .prompt_with_message("Enter a name for the new screen:")
                        .ok()
                        .flatten();
                    if let Some(name) = name {
                        let trimmed = name.trim().to_string();
                        if !trimmed.is_empty() {
                            store.save_current_as_new_screen(&trimmed);
                        }
                    }
                }
            } else if key_event.key() == "Escape" {
                store.detail_drawer_open.set(false);
            }
        });
        on_cleanup(move || handle.remove());
    });

    // -- Split workspace ------------------------------------------------------
    // The symbol detail drawer docked to the right of table / heatmap, with
    // the labs splitter handle (drag to resize, double-click to reset). The
    // drag runs event-listener style: mousedown arms the flags on the
    // splitter, the global mousemove/mouseup listeners do the rest.
    let drawer_width = RwSignal::new(360.0_f64);
    let dragging = RwSignal::new(false);
    let drag_start_x = RwSignal::new(0.0_f64);
    let drag_start_width = RwSignal::new(360.0_f64);

    let begin_drag = move |event: web_sys::MouseEvent| {
        event.prevent_default();
        drag_start_x.set(event.client_x() as f64);
        drag_start_width.set(drawer_width.get_untracked());
        dragging.set(true);
    };

    // Global dragging pass (kept cheap by the `dragging` short-circuit).
    Effect::new(move |_| {
        let handle = window_event_listener_untyped("mousemove", move |event: web_sys::Event| {
            if !dragging.get_untracked() {
                return;
            }
            if let Ok(mouse) = event.clone().dyn_into::<web_sys::MouseEvent>() {
                let delta = drag_start_x.get_untracked() - mouse.client_x() as f64;
                drawer_width.set((drag_start_width.get_untracked() + delta).clamp(280.0, 620.0));
            }
        });
        on_cleanup(move || handle.remove());
    });
    Effect::new(move |_| {
        let handle = window_event_listener_untyped("mouseup", move |_event: web_sys::Event| {
            if dragging.get_untracked() {
                dragging.set(false);
            }
        });
        on_cleanup(move || handle.remove());
    });
    Effect::new(move |_| {
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            let _ = document.body().map(|body| {
                if dragging.get() {
                    body.style().set_property("cursor", "col-resize").ok();
                    body.style().set_property("user-select", "none").ok();
                } else {
                    let _ = body.style().remove_property("cursor");
                    let _ = body.style().remove_property("user-select");
                }
            });
        }
    });

    view! {
        <div class="sc-view">
            <FilterBar />

            <div class="sc-view__body">
                <div class="sc-workspace__primary">
                    {move || {
                        if store.view_mode.get() == ViewMode::Heatmap {
                            view! { <ScreenerHeatmap /> }.into_any()
                        } else {
                            view! { <ScreenerTable /> }.into_any()
                        }
                    }}
                </div>

                <Show when=move || store.detail_drawer_open.get()>
                    <div
                        class="sc-workspace__splitter"
                        title="Drag to resize (double-click to reset)"
                        on:mousedown=begin_drag
                        on:dblclick=move |_| drawer_width.set(360.0)
                    ></div>
                    <div
                        class="sc-workspace__secondary"
                        style=move || format!("width: {:.0}px", drawer_width.get())
                    >
                        <SymbolDetailDrawer />
                    </div>
                </Show>
            </div>

            <OpenScreenModal />
            <FiltersModal />
            <ColumnModal />
        </div>
    }
}
