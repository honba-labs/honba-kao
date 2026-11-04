//! `ScreenCatalogModal` (doc §7 `.`): built-in preset shelf + saved custom
//! screens + a save sheet that snapshots the page's live state via Shift+N
//! or the save button. Applying a screen delegates the full criteria/tab/
//! sort/columns state transition to the page callback — the modal stays dumb.

use honba_kao_ui_core::primitive::{ButtonSize, ButtonVariant, HfButton};
use honba_kao_ui_core::{HfInput, HfModal};
use leptos::prelude::*;

use crate::features::screener::screen_catalog::{CustomScreen, builtin_groups};

#[component]
pub fn ScreenCatalogModal(
    /// Open flag; `.` (and Shift+N feedback) flips it from the page.
    open: RwSignal<bool>,
    /// User-saved screens (page owns loading/persisting through localStorage).
    custom_screens: RwSignal<Vec<CustomScreen>>,
    /// Toggles the save-name sheet inside the catalog (Shift+N opens it).
    save_sheet_open: RwSignal<bool>,
    /// Applies the chosen screen (tab, sort, criteria, visible columns).
    on_apply: Callback<CustomScreen>,
    /// Saves the page's current state under the provided name.
    on_save: Callback<String>,
    /// Deletes a user (non-builtin) screen by id when the page wires it up.
    #[prop(optional)]
    on_delete: Option<Callback<String>>,
) -> impl IntoView {
    let save_name = RwSignal::new(String::new());
    view! {
        <HfModal
            open=open
            title="Screen Catalog"
            on_confirm=Callback::new(move |_| open.set(false))
        >
            <div class="flex flex-col gap-3">
                {builtin_groups()
                    .iter()
                    .map(|(group, group_screens)| {
                        let label = group.to_string();
                        view! {
                            <div class="flex flex-col gap-1">
                                <span class="text-[10px] uppercase tracking-wide text-text-muted">
                                    {label}
                                </span>
                                <div class="grid grid-cols-2 gap-1">
                                    {group_screens
                                        .iter()
                                        .map(|screen| {
                                            let chosen = screen.clone();
                                            let name = screen.name.clone();
                                            let description = screen.description.clone();
                                            view! {
                                                <button
                                                    type="button"
                                                    class="hf-screen-row rounded border border-border-soft bg-surface-base px-2 py-1 text-left transition-colors hover:border-emerald-500/60"
                                                    on:click=move |_| {
                                                        on_apply.run(chosen.clone());
                                                        open.set(false);
                                                    }
                                                >
                                                    <span class="block truncate text-[11px] font-semibold text-text-high">
                                                        {name}
                                                    </span>
                                                    <span class="block truncate text-[10px] text-text-muted">
                                                        {description}
                                                    </span>
                                                </button>
                                            }
                                            .into_any()
                                        })
                                        .collect::<Vec<AnyView>>()}
                                </div>
                            </div>
                        }
                        .into_any()
                    })
                    .collect::<Vec<AnyView>>()}
                <div class="flex flex-col gap-1 border-t border-border-soft pt-2">
                    <span class="text-[10px] uppercase tracking-wide text-text-muted">
                        "Saved Screens"
                    </span>
                    {move || {
                        if custom_screens.get().is_empty() {
                            vec![view! {
                                <span class="text-[10px] text-text-muted">
                                    "No custom screens yet — set filters, then press Shift+N or use the save sheet."
                                </span>
                            }
                            .into_any()]
                        } else {
                            custom_screens
                                .get()
                                .iter()
                                .filter(|screen| !screen.id.starts_with("builtin."))
                                .map(|screen| {
                                    let apply = screen.clone();
                                    let apply_name = screen.name.clone();
                                    let description = screen.description.clone();
                                    let remove = screen.id.clone();
                                    let remove_name = screen.name.clone();
                                    view! {
                                        <div class="flex items-center gap-2 rounded border border-border-soft bg-surface-base px-2 py-1">
                                            <button
                                                type="button"
                                                class="min-w-0 flex-1 text-left"
                                                on:click=move |_| {
                                                    on_apply.run(apply.clone());
                                                    open.set(false);
                                                }
                                            >
                                                <span class="block truncate text-[11px] font-semibold text-text-high">
                                                    {apply_name}
                                                </span>
                                                <span class="block truncate text-[10px] text-text-muted">
                                                    {description}
                                                </span>
                                            </button>
                                            {on_delete.map(|del| view! {
                                                <HfButton
                                                    variant=ButtonVariant::Ghost
                                                    size=ButtonSize::Xs
                                                    aria_label=remove_name.clone()
                                                    on_click=Callback::new(move |_| {
                                                        del.run(remove.clone())
                                                    })
                                                >
                                                    "✕"
                                                </HfButton>
                                            }
                                            .into_any())}
                                        </div>
                                    }
                                    .into_any()
                                })
                                .collect::<Vec<AnyView>>()
                        }
                    }}
                </div>
                {move || {
                    if save_sheet_open.get() {
                        view! {
                            <div class="flex items-center gap-2 border-t border-border-soft pt-2">
                                <div class="flex-1">
                                    <HfInput
                                        value=save_name
                                        placeholder="New screen name…"
                                        on_input=Callback::new(move |text| save_name.set(text))
                                    />
                                </div>
                                <HfButton
                                    variant=ButtonVariant::Primary
                                    size=ButtonSize::Xs
                                    on_click=Callback::new(move |_| {
                                        let name = save_name.get_untracked().trim().to_string();
                                        if !name.is_empty() {
                                            on_save.run(name);
                                            save_name.set(String::new());
                                            save_sheet_open.set(false);
                                        }
                                    })
                                >
                                    "Save Screen"
                                </HfButton>
                                <HfButton
                                    variant=ButtonVariant::Ghost
                                    size=ButtonSize::Xs
                                    on_click=Callback::new(move |_| save_sheet_open.set(false))
                                >
                                    "Cancel"
                                </HfButton>
                            </div>
                        }
                        .into_any()
                    } else {
                        view! {
                            <div class="flex items-center gap-2 border-t border-border-soft pt-2">
                                <HfButton
                                    variant=ButtonVariant::Ghost
                                    size=ButtonSize::Xs
                                    on_click=Callback::new(move |_| save_sheet_open.set(true))
                                >
                                    "Save current view…"
                                </HfButton>
                            </div>
                        }
                        .into_any()
                    }
                }}
            </div>
        </HfModal>
    }
}
