//! QuickNavGrid — 4-column keyboard-navigable grid of workspace shortcuts.
//!
//! Models OpenTerminalUI's `QuickNavGrid.tsx`. Implements a roving tabindex grid
//! with Arrow / Home / End / Enter / Space keyboard navigation.

use leptos::prelude::*;
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[derive(Debug, Clone)]
pub struct QuickNavItem {
    pub id: String,
    pub label: String,
    pub shortcut: String,
    pub description: Option<String>,
    pub disabled: bool,
    pub on_select: Option<Callback<()>>,
}

impl PartialEq for QuickNavItem {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.label == other.label
            && self.shortcut == other.shortcut
            && self.description == other.description
            && self.disabled == other.disabled
    }
}

impl QuickNavItem {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        shortcut: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            shortcut: shortcut.into(),
            description: None,
            disabled: false,
            on_select: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_on_select(mut self, cb: Callback<()>) -> Self {
        self.on_select = Some(cb);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuickNavSection {
    pub id: String,
    pub title: String,
    pub items: Vec<QuickNavItem>,
}

impl QuickNavSection {
    pub fn new(id: impl Into<String>, title: impl Into<String>, items: Vec<QuickNavItem>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            items,
        }
    }
}

#[component]
pub fn QuickNavGrid(
    sections: Vec<QuickNavSection>,
    #[prop(optional, into)] selected_item_id: Option<String>,
    #[prop(optional, default = 4)] column_count: usize,
    #[prop(optional, into)] class: String,
    #[prop(optional, into, default = "Quick navigation grid".to_string())] aria_label: String,
    #[prop(optional)] on_select: Option<Callback<QuickNavItem>>,
) -> impl IntoView {
    let safe_columns = column_count.max(1);

    // Compute flattened item list for roving index math
    let flat_items: Vec<QuickNavItem> = sections
        .iter()
        .flat_map(|sec| sec.items.iter().cloned())
        .collect();

    let initial_active = flat_items.iter().position(|it| !it.disabled).unwrap_or(0);

    let (active_index, set_active_index) = signal(initial_active);
    let items_rc = Rc::new(flat_items);

    let grid_class = format!("ot-home-widget-nav {class}").trim().to_string();
    let col_style = format!("grid-template-columns: repeat({safe_columns}, minmax(0, 1fr))");

    let mut running_index = 0usize;

    view! {
        <div class=grid_class aria-label=aria_label>
            {sections.into_iter().map(|section| {
                let section_id = section.id.clone();
                let section_title = section.title.clone();
                let heading_id = format!("quick-nav-{section_id}");
                let start_idx = running_index;
                running_index += section.items.len();

                let total_items = items_rc.len();
                let on_select_cb = on_select;
                let items_for_section = Rc::clone(&items_rc);

                view! {
                    <section class="ot-home-widget-nav-section" aria-labelledby=heading_id.clone()>
                        <header class="ot-home-widget-nav-section-header">
                            <h3 id=heading_id.clone() class="ot-home-widget-nav-section-title">
                                {section_title}
                            </h3>
                        </header>
                        <div class="ot-home-widget-nav-grid" style=col_style.clone()>
                            {section.items.into_iter().enumerate().map(|(item_idx, item)| {
                                let global_idx = start_idx + item_idx;
                                let is_selected = selected_item_id.as_ref() == Some(&item.id);
                                let is_tab_stop = move || active_index.get() == global_idx;

                                let item_aria_label = if let Some(ref desc) = item.description {
                                    format!("{}. {}", item.label, desc)
                                } else {
                                    item.label.clone()
                                };

                                let item_clone = item.clone();
                                let on_click_item = item.clone();
                                let items_for_key = Rc::clone(&items_for_section);

                                let handle_click = move |_| {
                                    if on_click_item.disabled {
                                        return;
                                    }
                                    set_active_index.set(global_idx);
                                    if let Some(ref it_cb) = on_click_item.on_select {
                                        it_cb.run(());
                                    }
                                    if let Some(ref grid_cb) = on_select_cb {
                                        grid_cb.run(on_click_item.clone());
                                    }
                                };

                                let on_focus = move |_| {
                                    set_active_index.set(global_idx);
                                };

                                let on_keydown = move |ev: web_sys::KeyboardEvent| {
                                    if total_items == 0 {
                                        return;
                                    }
                                    let key = ev.key();

                                    let mut target_idx = None;
                                    if key == "ArrowRight" {
                                        target_idx = Some(global_idx + 1);
                                    } else if key == "ArrowLeft" {
                                        if global_idx > 0 {
                                            target_idx = Some(global_idx - 1);
                                        }
                                    } else if key == "ArrowDown" {
                                        target_idx = Some(global_idx + safe_columns);
                                    } else if key == "ArrowUp" {
                                        if global_idx >= safe_columns {
                                            target_idx = Some(global_idx - safe_columns);
                                        }
                                    } else if key == "Home" {
                                        target_idx = Some(0);
                                    } else if key == "End" {
                                        target_idx = Some(total_items.saturating_sub(1));
                                    } else if (key == "Enter" || key == " ") && !item_clone.disabled {
                                        ev.prevent_default();
                                        if let Some(ref it_cb) = item_clone.on_select {
                                            it_cb.run(());
                                        }
                                        if let Some(ref grid_cb) = on_select_cb {
                                            grid_cb.run(item_clone.clone());
                                        }
                                    }

                                    if let Some(idx) = target_idx {
                                        ev.prevent_default();
                                        let clamped = idx.min(total_items.saturating_sub(1));
                                        if let Some(target_item) = items_for_key.get(clamped) {
                                            if !target_item.disabled {
                                                set_active_index.set(clamped);

                                                #[cfg(target_arch = "wasm32")]
                                                {
                                                    let tile_id = format!("ot-quick-nav-tile-{}", target_item.id);
                                                    if let Some(window) = web_sys::window() {
                                                        if let Some(doc) = window.document() {
                                                            if let Some(el) = doc.get_element_by_id(&tile_id) {
                                                                if let Ok(btn) = el.dyn_into::<web_sys::HtmlButtonElement>() {
                                                                    let _ = btn.focus();
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                };

                                let tile_id = format!("ot-quick-nav-tile-{}", item.id);
                                let desc_view = item.description.clone().map(|desc| {
                                    view! { <span class="ot-home-widget-nav-description">{desc}</span> }
                                });

                                view! {
                                    <button
                                        id=tile_id
                                        type="button"
                                        class="ot-home-widget-nav-tile"
                                        data-selected=if is_selected { "true" } else { "false" }
                                        disabled=item.disabled
                                        aria-keyshortcuts=item.shortcut.clone()
                                        aria-label=item_aria_label
                                        tabindex=move || if is_tab_stop() { 0 } else { -1 }
                                        on:click=handle_click
                                        on:focus=on_focus
                                        on:keydown=on_keydown
                                    >
                                        <span class="ot-home-widget-nav-label">{item.label}</span>
                                        <span class="ot-home-widget-shortcut">{item.shortcut.clone()}</span>
                                        {desc_view}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </section>
                }
            }).collect_view()}
        </div>
    }
}
