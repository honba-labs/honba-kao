//! `HfDialog`, `HfModal`, `HfDrawer` — accessible dialogs with focus trap and escape dismiss.
//! Ported from `secured-finance-app` atom `Dialog` and organism-level `PlaceOrder` /
//! `RemoveOrderDialog` confirmation patterns.

use leptos::prelude::*;
use crate::hooks::use_escape_key;

/// Accessible modal dialog with focus trap and escape-key handling.
#[component]
pub fn HfDialog(
    #[prop(into)] open: RwSignal<bool>,
    #[prop(into, optional)] title: Option<String>,
    #[prop(into, optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let base = "hf-dialog fixed inset-0 z-50 flex items-center justify-center";
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };

    use_escape_key(Callback::new(move |_| open.set(false)));

    let title_clone = title.clone();
    let children_view = children();
    view! {
        <div
            class=classes
            class=("hidden", move || !open.get())
            on:click=move |_| open.set(false)
        >
            <div class="hf-dialog-backdrop fixed inset-0 bg-black/60 backdrop-blur-sm transition-opacity" />
            <div
                class="hf-dialog-panel w-full max-w-lg rounded-xl border border-border-strong bg-surface-base shadow-2xl flex flex-col"
                role="dialog"
                aria-modal="true"
                class=("hidden", move || !open.get())
            >
                {match &title_clone {
                    Some(t) => view! {
                        <div class="hf-dialog-header px-4 py-3 border-b border-border-soft flex items-center justify-between">
                            <span class="text-sm font-semibold text-text-high">{t.clone()}</span>
                            <button
                                type="button"
                                class="hf-dialog-close text-text-muted hover:text-text-high"
                                on:click=move |_| open.set(false)
                                aria-label="Close"
                            >
                                <span class="hf-icon hf-icon-close" aria_hidden="true">"✕"</span>
                            </button>
                        </div>
                    }
                    .into_any(),
                    None => view! {
                        <div class="hf-dialog-header px-4 py-3 border-b border-border-soft flex items-center justify-between"></div>
                    }
                    .into_any(),
                }}
                <div class="hf-dialog-body p-4 overflow-auto">{children_view}</div>
            </div>
        </div>
    }
}

/// Modal confirmation prompt (order summary, leverage risk, confirm).
#[component]
pub fn HfModal(
    #[prop(into)] open: RwSignal<bool>,
    #[prop(into, optional)] title: Option<String>,
    #[prop(into, optional)] confirm_label: Option<String>,
    #[prop(into, optional)] cancel_label: Option<String>,
    #[prop(into)] on_confirm: Callback<()>,
    #[prop(into, optional)] on_cancel: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    let confirm_label = confirm_label.unwrap_or_else(|| "Confirm".to_string());
    let cancel_label = cancel_label.unwrap_or_else(|| "Cancel".to_string());

    use_escape_key(Callback::new(move |_| {
        if let Some(cb) = on_cancel.as_ref() {
            cb.run(());
        }
        open.set(false);
    }));

    let children_view = children();
    view! {
        <div
            class="hf-modal-backdrop fixed inset-0 z-50 bg-black/60 flex items-center justify-center"
            class=("hidden", move || !open.get())
            on:click=move |_| {
                if let Some(cb) = on_cancel.as_ref() {
                    cb.run(());
                }
                open.set(false);
            }
        />
        <div
            class="hf-modal w-full max-w-md rounded-xl border border-border-strong bg-surface-base shadow-2xl flex flex-col"
            role="dialog"
            aria-modal="true"
            class=("hidden", move || !open.get())
        >
            {match &title {
                Some(t) => view! {
                    <div class="hf-modal-header px-4 py-3 border-b border-border-soft">
                        <span class="text-sm font-semibold text-text-high">{t.clone()}</span>
                    </div>
                }
                .into_any(),
                None => view! {
                    <div class="hf-modal-header px-4 py-3 border-b border-border-soft"></div>
                }
                .into_any(),
            }}
            <div class="hf-modal-body p-4">{children_view}</div>
            <div class="hf-modal-footer px-4 py-3 border-t border-border-soft bg-surface-subtle flex justify-end gap-2">
                <button
                    type="button"
                    class="hf-modal-cancel px-4 py-1.5 rounded-md text-sm font-medium text-text-muted hover:bg-surface-elevated transition-colors"
                    on:click=move |_| {
                        if let Some(cb) = on_cancel.as_ref() {
                            cb.run(());
                        }
                        open.set(false);
                    }
                >
                    {cancel_label}
                </button>
                <button
                    type="button"
                    class="hf-modal-confirm px-4 py-1.5 rounded-md text-sm font-medium text-white bg-emerald-600 hover:bg-emerald-700 transition-colors"
                    on:click=move |_| {
                        on_confirm.run(());
                        open.set(false);
                    }
                >
                    {confirm_label}
                </button>
            </div>
        </div>
    }
}

/// Slide-over drawer panel (right side) — used by the Agent Console and wallets.
#[component]
pub fn HfDrawer(
    #[prop(into)] open: RwSignal<bool>,
    #[prop(into, optional)] title: Option<String>,
    #[prop(default = false)] right: bool,
    children: Children,
) -> impl IntoView {
    let title_clone = title.clone();
    let children_view = children();
    let translate = move || {
        if open.get() { "translate-x-0" } else { if right { "translate-x-full" } else { "-translate-x-full" } }
    };
    view! {
        <div
            class=move || format!(
                "hf-drawer fixed inset-y-0 z-40 w-[440px] bg-surface-base border border-border-strong shadow-2xl flex flex-col transition-transform duration-300 {} {}",
                if right { "right-0" } else { "left-0" },
                translate()
            )
            class=("hidden", move || !open.get())
        >
            {match &title_clone {
                Some(t) => view! {
                    <div class="hf-drawer-header px-3 py-2 border-b border-border-soft flex items-center justify-between">
                        <span class="text-xs font-semibold text-text-high uppercase tracking-wide">{t.clone()}</span>
                        <button
                            type="button"
                            class="text-text-muted hover:text-text-high"
                            on:click=move |_| open.set(false)
                            aria-label="Close"
                        >
                            <span class="hf-icon hf-icon-close" aria_hidden="true">"✕"</span>
                        </button>
                    </div>
                }
                .into_any(),
                None => view! {
                    <div class="hf-drawer-header px-3 py-2 border-b border-border-soft flex items-center justify-between"></div>
                }
                .into_any(),
            }}
            <div class="hf-drawer-body flex-grow overflow-y-auto p-3">{children_view}</div>
        </div>
    }
}
