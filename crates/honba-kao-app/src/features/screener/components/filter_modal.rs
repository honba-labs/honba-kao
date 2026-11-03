//! `ScreenerFilterModal` — granular numeric filter grid (price band, P/E,
//! RVOL, RSI) plus universe checkboxes. Reset delegates to the page-level
//! `on_reset` callback (shared with the filter bar).

use honba_kao_ui_core::primitive::{ButtonSize, ButtonVariant, HfButton};
use honba_kao_ui_core::{HfCheckbox, HfModal, HfNumberInput};
use leptos::prelude::*;
use rust_decimal::Decimal;

#[component]
pub fn ScreenerFilterModal(
    open: RwSignal<bool>,
    min_price: RwSignal<Decimal>,
    max_price: RwSignal<Decimal>,
    min_pe: RwSignal<Decimal>,
    max_pe: RwSignal<Decimal>,
    min_rvol: RwSignal<Decimal>,
    min_rsi: RwSignal<Decimal>,
    max_rsi: RwSignal<Decimal>,
    fno_only: RwSignal<bool>,
    watchlist_only: RwSignal<bool>,
    on_reset: Callback<()>,
) -> impl IntoView {
    view! {
        <HfModal
            open=open
            title="Granular Filters"
            on_confirm=Callback::new(move |_| open.set(false))
            on_cancel=Callback::new(move |_| {
                on_reset.run(());
                open.set(false);
            })
        >
            <div class="flex flex-col gap-3">
                <div class="flex items-center justify-between">
                    <span class="text-[11px] font-semibold uppercase tracking-wide text-text-muted">
                        "Price Band ₹ (dec! 0 clears)"
                    </span>
                    <HfButton
                        variant=ButtonVariant::Tertiary
                        size=ButtonSize::Xs
                        on_click=on_reset
                    >
                        "Reset"
                    </HfButton>
                </div>
                <div class="grid grid-cols-2 gap-2">
                    <div class="flex flex-col gap-1">
                        <span class="text-[10px] uppercase text-text-muted">"Min"</span>
                        <HfNumberInput value=min_price placeholder="0" />
                    </div>
                    <div class="flex flex-col gap-1">
                        <span class="text-[10px] uppercase text-text-muted">"Max"</span>
                        <HfNumberInput value=max_price placeholder="0" />
                    </div>
                </div>

                <span class="text-[11px] font-semibold text-text-high">"P/E range"</span>
                <div class="grid grid-cols-2 gap-2">
                    <div class="flex flex-col gap-1">
                        <span class="text-[10px] uppercase text-text-muted">"Min"</span>
                        <HfNumberInput value=min_pe placeholder="0" />
                    </div>
                    <div class="flex flex-col gap-1">
                        <span class="text-[10px] uppercase text-text-muted">"Max"</span>
                        <HfNumberInput value=max_pe placeholder="0" />
                    </div>
                </div>

                <span class="text-[11px] font-semibold text-text-high">"RVOL ≥"</span>
                <HfNumberInput value=min_rvol placeholder="0" />

                <span class="text-[11px] font-semibold text-text-high">"RSI band"</span>
                <div class="grid grid-cols-2 gap-2">
                    <div class="flex flex-col gap-1">
                        <span class="text-[10px] uppercase text-text-muted">"Min"</span>
                        <HfNumberInput value=min_rsi placeholder="0" />
                    </div>
                    <div class="flex flex-col gap-1">
                        <span class="text-[10px] uppercase text-text-muted">"Max"</span>
                        <HfNumberInput value=max_rsi placeholder="0" />
                    </div>
                </div>

                <div class="flex items-center gap-4 border-t border-border-soft pt-2">
                    <HfCheckbox checked=fno_only label="F&O Universe" />
                    <HfCheckbox checked=watchlist_only label="★ Watchlist only" />
                </div>
            </div>
        </HfModal>
    }
}
