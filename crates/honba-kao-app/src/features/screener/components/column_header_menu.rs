//! `ColumnHeaderMenu` — per-header dropdown (doc §8 Phase 5): Sort Asc/Desc,
//! Move Left/Right within the master column order, Hide via the page's
//! `column_flags`, and criteria-bound numeric min/max Apply bound to the
//! SAME signals the Filters modal and flyouts drive (single source).
//!
//! Pure contracts (`swap_in_order`, `filter_slots_for`) are pinned by
//! `tests/screener_column_menu.rs`.

use honba_kao_ui_core::HfNumberInput;
use leptos::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Move target direction: `Left` = toward the leading (first) column,
/// `Right` = toward the trailing (last) column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection {
    Left,
    Right,
}

/// Criteria-bound filter slot resolved from a column key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterSlot {
    PriceMin,
    PriceMax,
    PeMin,
    PeMax,
    RvolMin,
    RsiMin,
    RsiMax,
}

/// Swaps `key` with its adjacent neighbor in `order`. Returns `None` when
/// the key is unknown or already at the boundary (no-op; the caller keeps
/// the current order).
pub fn swap_in_order(
    order: &[&'static str],
    key: &'static str,
    direction: MoveDirection,
) -> Option<Vec<&'static str>> {
    let position = order.iter().position(|entry| *entry == key)?;
    let target = match direction {
        MoveDirection::Left => position.checked_sub(1)?,
        MoveDirection::Right => {
            if position + 1 < order.len() {
                position + 1
            } else {
                return None;
            }
        }
    };
    let mut next = order.to_vec();
    next.swap(position, target);
    Some(next)
}

/// Criteria binding of one column key as `(min_slot, max_slot)`; `None`
/// where `FilterCriteria` carries no matching bound (rvol is min-only).
/// Keys outside the binding map yield `None` — no filter rows in the menu.
pub fn filter_slots_for(key: &str) -> Option<(Option<FilterSlot>, Option<FilterSlot>)> {
    match key {
        "price" => Some((Some(FilterSlot::PriceMin), Some(FilterSlot::PriceMax))),
        "pe_ratio" | "forward_pe" => Some((Some(FilterSlot::PeMin), Some(FilterSlot::PeMax))),
        "rvol" => Some((Some(FilterSlot::RvolMin), None)),
        "rsi_14" => Some((Some(FilterSlot::RsiMin), Some(FilterSlot::RsiMax))),
        _ => None,
    }
}

/// Resolves a filter slot onto the shared criteria signal (the table passes
/// all seven; only the bound ones surface in the menu).
#[allow(clippy::too_many_arguments)]
fn slot_signal(
    slot: Option<FilterSlot>,
    min_price: RwSignal<Decimal>,
    max_price: RwSignal<Decimal>,
    min_pe: RwSignal<Decimal>,
    max_pe: RwSignal<Decimal>,
    min_rvol: RwSignal<Decimal>,
    min_rsi: RwSignal<Decimal>,
    max_rsi: RwSignal<Decimal>,
) -> Option<RwSignal<Decimal>> {
    match slot {
        Some(FilterSlot::PriceMin) => Some(min_price),
        Some(FilterSlot::PriceMax) => Some(max_price),
        Some(FilterSlot::PeMin) => Some(min_pe),
        Some(FilterSlot::PeMax) => Some(max_pe),
        Some(FilterSlot::RvolMin) => Some(min_rvol),
        Some(FilterSlot::RsiMin) => Some(min_rsi),
        Some(FilterSlot::RsiMax) => Some(max_rsi),
        None => None,
    }
}

#[component]
pub fn ColumnHeaderMenu(
    column_key: &'static str,
    open: RwSignal<Option<&'static str>>,
    hide_flag: Option<RwSignal<bool>>,
    on_sort: Option<Callback<bool>>,
    on_move: Callback<MoveDirection>,
    min_price: RwSignal<Decimal>,
    max_price: RwSignal<Decimal>,
    min_pe: RwSignal<Decimal>,
    max_pe: RwSignal<Decimal>,
    min_rvol: RwSignal<Decimal>,
    min_rsi: RwSignal<Decimal>,
    max_rsi: RwSignal<Decimal>,
) -> impl IntoView {
    view! {
        <span class="relative inline-flex">
            <button
                type="button"
                title="Column menu — sort, move, hide, filter"
                class="px-0.5 text-[9px] leading-none text-text-muted hover:text-text-high"
                on:click=move |_| {
                    let next = if open.get_untracked() == Some(column_key) {
                        None
                    } else {
                        Some(column_key)
                    };
                    open.set(next);
                }
            >
                "▾"
            </button>
            {move || {
                (open.get() == Some(column_key)).then(|| {
                    // One open: seed the drafts via get_untracked so the panel
                    // survives criteria churn and only remounts per toggle.
                    let (min_slot, max_slot) =
                        filter_slots_for(column_key).unwrap_or((None, None));
                    let min_signal = slot_signal(
                        min_slot,
                        min_price,
                        max_price,
                        min_pe,
                        max_pe,
                        min_rvol,
                        min_rsi,
                        max_rsi,
                    );
                    let max_signal = slot_signal(
                        max_slot,
                        min_price,
                        max_price,
                        min_pe,
                        max_pe,
                        min_rvol,
                        min_rsi,
                        max_rsi,
                    );
                    let min_draft: RwSignal<Decimal> = RwSignal::new(
                        min_signal
                            .map(|signal| signal.get_untracked())
                            .unwrap_or(dec!(0)),
                    );
                    let max_draft: RwSignal<Decimal> = RwSignal::new(
                        max_signal
                            .map(|signal| signal.get_untracked())
                            .unwrap_or(dec!(0)),
                    );
                    view! {
                        <div class="absolute left-0 top-full z-40 mt-1 w-40 rounded border border-border-soft bg-surface-base p-1.5 shadow-lg shadow-black/20 flex flex-col gap-1 text-[11px] text-text-high">
                            {on_sort.map(|apply_sort| {
                                view! {
                                    <button
                                        type="button"
                                        class="text-left hover:bg-surface-subtle"
                                        on:click=move |_| {
                                            apply_sort.run(true);
                                            open.set(None);
                                        }
                                    >
                                        "Sort Ascending"
                                    </button>
                                    <button
                                        type="button"
                                        class="text-left hover:bg-surface-subtle"
                                        on:click=move |_| {
                                            apply_sort.run(false);
                                            open.set(None);
                                        }
                                    >
                                        "Sort Descending"
                                    </button>
                                }
                            })}
                            <button
                                type="button"
                                class="text-left hover:bg-surface-subtle"
                                on:click=move |_| on_move.run(MoveDirection::Left)
                            >
                                "◀ Move Left"
                            </button>
                            <button
                                type="button"
                                class="text-left hover:bg-surface-subtle"
                                on:click=move |_| on_move.run(MoveDirection::Right)
                            >
                                "▶ Move Right"
                            </button>
                            {hide_flag.map(|flag| {
                                view! {
                                    <button
                                        type="button"
                                        class="text-left hover:bg-surface-subtle"
                                        on:click=move |_| {
                                            flag.set(false);
                                            open.set(None);
                                        }
                                    >
                                        "✕ Hide"
                                    </button>
                                }
                            })}
                            {(!min_signal.is_none() || !max_signal.is_none()).then(|| {
                                view! {
                                    <div class="flex flex-col gap-1 border-t border-border-soft pt-1">
                                        <span class="text-[10px] uppercase tracking-wide text-text-muted">
                                            "Filter (0 clears)"
                                        </span>
                                        <div class="grid grid-cols-2 gap-1">
                                            <div class="flex flex-col gap-0.5">
                                                <span class="text-[10px] text-text-muted">"Min"</span>
                                                {(min_signal.is_some())
                                                    .then(|| view! { <HfNumberInput value=min_draft placeholder="0" /> })}
                                            </div>
                                            <div class="flex flex-col gap-0.5">
                                                <span class="text-[10px] text-text-muted">"Max"</span>
                                                {(max_signal.is_some())
                                                    .then(|| view! { <HfNumberInput value=max_draft placeholder="0" /> })}
                                            </div>
                                        </div>
                                        <button
                                            type="button"
                                            title="Shared with the Filters modal"
                                            class="rounded border border-emerald-600 px-1 py-0.5 text-left text-[10px] text-emerald-500 hover:bg-emerald-600/10"
                                            on:click=move |_| {
                                                if let Some(signal) = min_signal {
                                                    signal.set(min_draft.get_untracked());
                                                }
                                                if let Some(signal) = max_signal {
                                                    signal.set(max_draft.get_untracked());
                                                }
                                                open.set(None);
                                            }
                                        >
                                            "Apply"
                                        </button>
                                    </div>
                                }
                            })}
                        </div>
                    }
                })
            }}
        </span>
    }
}
