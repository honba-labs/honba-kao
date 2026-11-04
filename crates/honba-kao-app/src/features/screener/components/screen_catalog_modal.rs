//! §7 screen catalog modal: builtin groups + custom screens with an inline
//! save sheet (Shift + N) driven by the terminal's one-window shortcuts.
use leptos::prelude::*;

use crate::features::screener::screen_catalog::{CustomScreen, builtin_groups, delete_screen};
use honba_kao_ui_core::primitive::{ButtonSize, ButtonVariant, HfButton};
use honba_kao_ui_core::{HfInput, HfModal};

#[component]
pub fn ScreenCatalogModal(
    open: RwSignal<bool>,
    custom_screens: RwSignal<Vec<CustomScreen>>,
    save_sheet_open: RwSignal<bool>,
    on_apply: Callback<CustomScreen>,
    on_save: Callback<String>,
) -> impl IntoView {
    let save_name = RwSignal::new(String::from("My Screen"));

    // Built-ins render once at mount; only the custom list re-reads its
    // signal so deletions re-render their section.
    let groups: Vec<AnyView> = builtin_groups()
        .into_iter()
        .map(|(label, screens)| {
            let rows: Vec<AnyView> = screens
                .into_iter()
                .map(|screen| {
                    let name = screen.name.clone();
                    let description = screen.description.clone();
                    view! {
                        <div
                            role="button"
                            class="rounded border border-border-soft hover:bg-surface-subtle px-2 py-1 text-left cursor-pointer"
                            on:click=move |_| {
                                on_apply.run(screen.clone());
                                open.set(false);
                            }
                        >
                            <div class="font-mono text-xs text-text-high">{name}</div>
                            <div class="text-[10px] text-text-muted truncate">{description}</div>
                        </div>
                    }
                    .into_any()
                })
                .collect();
            view! {
                <span class="text-[10px] uppercase tracking-wide text-text-muted">{label}</span>
                {rows}
            }
            .into_any()
        })
        .collect();

    view! {
        <HfModal
            open=open
            title="Screen Catalog"
            on_confirm=Callback::new(move |_| open.set(false))
            on_cancel=Callback::new(move |_| open.set(false))
        >
            <div class="flex flex-col gap-2">
                {move || {
                    save_sheet_open
                        .get()
                        .then(|| {
                            view! {
                                <div class="flex items-center gap-1">
                                    <HfInput
                                        value=save_name
                                        placeholder="Screen name"
                                        on_input=Callback::new(move |v| save_name.set(v))
                                    />
                                    <HfButton
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Xs
                                        on_click=Callback::new(move |_| {
                                            let name = save_name.get_untracked().trim().to_string();
                                            if !name.is_empty() {
                                                on_save.run(name.clone());
                                                save_name.set(String::new());
                                                save_sheet_open.set(false);
                                                open.set(false);
                                            }
                                        })
                                    >
                                        "Save"
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
                        })
                }}
                {groups}
                <span class="text-[10px] uppercase tracking-wide text-text-muted">"Custom"</span>
                {move || {
                    let screens = custom_screens.get();
                    if screens.is_empty() {
                        view! {
                            <div class="text-[10px] text-text-muted">
                                "No custom screens yet. Press Shift + N to save the current view."
                            </div>
                        }
                        .into_any()
                    } else {
                        let rows: Vec<AnyView> = screens
                            .into_iter()
                            .map(|screen| {
                                let id = screen.id.clone();
                                let name = screen.name.clone();
                                let description = screen.description.clone();
                                view! {
                                    <div class="flex items-center gap-2 rounded border border-border-soft hover:bg-surface-subtle px-2 py-1">
                                        <div
                                            role="button"
                                            class="flex-1 min-w-0 text-left cursor-pointer"
                                            on:click=move |_| {
                                                on_apply.run(screen.clone());
                                                open.set(false);
                                            }
                                        >
                                            <div class="font-mono text-xs text-text-high">{name}</div>
                                            <div class="text-[10px] text-text-muted truncate">
                                                {description}
                                            </div>
                                        </div>
                                        <HfButton
                                            variant=ButtonVariant::Ghost
                                            size=ButtonSize::Xs
                                            on_click=Callback::new(move |_| {
                                                custom_screens.update(|v| delete_screen(v, &id));
                                            })
                                        >
                                            "✕"
                                        </HfButton>
                                    </div>
                                }
                                .into_any()
                            })
                            .collect();
                        rows.into_any()
                    }
                }}
            </div>
        </HfModal>
    }
}
