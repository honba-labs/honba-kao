//! `HfQuantityChipInput` — quantity input with percentage chips
//! (`[25%]`, `[50%]`, `[75%]`, `[100%]`) and a `Max` button.
//! Ported from `secured-finance-app` molecule patterns and the restructure spec
//! (Section 7.2).

use leptos::prelude::*;
use crate::primitive::{HfButton, HfOrderInputBox, HfChip, ButtonVariant, ButtonSize};

#[component]
pub fn HfQuantityChipInput(
    #[prop(into)] value: RwSignal<String>,
    #[prop(into)] max_quantity: Signal<f64>,
    #[prop(default = false)] disabled: bool,
) -> impl IntoView {
    let set_percent = move |pct: f64| {
        let max_qty = max_quantity.get();
        if max_qty > 0.0 {
            let target = max_qty * (pct / 100.0);
            value.set(format!("{:.4}", target));
        }
    };

    view! {
        <div class="hf-quantity-chip-input flex flex-col gap-1.5 w-full">
            <div class="hf-qty-input-row flex items-center gap-1.5">
                <HfOrderInputBox
                    value=value
                    placeholder="0.00"
                    disabled=disabled
                />
                <HfButton
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Sm
                    on_click=move |_| set_percent(100.0)
                >
                    "Max"
                </HfButton>
            </div>
            <div class="hf-qty-chips-row flex items-center gap-1">
                <HfChip selected=false on_click=move |_| set_percent(25.0)>"25%"</HfChip>
                <HfChip selected=false on_click=move |_| set_percent(50.0)>"50%"</HfChip>
                <HfChip selected=false on_click=move |_| set_percent(75.0)>"75%"</HfChip>
                <HfChip selected=false on_click=move |_| set_percent(100.0)>"100%"</HfChip>
            </div>
        </div>
    }
}
