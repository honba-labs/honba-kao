//! `HfQuantityChipInput` — quantity input with percentage chips
//! (`[25%]`, `[50%]`, `[75%]`, `[100%]`) and a `Max` button.
//! Ported from `secured-finance-app` molecule patterns and the restructure spec
//! (Section 7.2).

use crate::primitive::{ButtonSize, ButtonVariant, HfButton, HfChip, HfOrderInputBox};
use leptos::prelude::*;

/// Quantity for a percentage of the maximum. Pure so it can be unit-tested.
pub(crate) fn percent_of(max_qty: f64, pct: f64) -> f64 {
    if max_qty > 0.0 {
        max_qty * (pct / 100.0)
    } else {
        0.0
    }
}

#[component]
pub fn HfQuantityChipInput(
    #[prop(into)] value: RwSignal<String>,
    #[prop(into)] max_quantity: Signal<f64>,
    #[prop(default = false)] disabled: bool,
) -> impl IntoView {
    let set_percent = move |pct: f64| {
        let max_qty = max_quantity.get();
        value.set(format!("{:.4}", percent_of(max_qty, pct)));
    };
    let selected_pct = RwSignal::new(100.0);
    let chips = [25.0, 50.0, 75.0, 100.0];
    let chip_indices = move || (0..chips.len()).collect::<Vec<usize>>();

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
                    on_click=Callback::new(move |_| {
                        selected_pct.set(100.0);
                        set_percent(100.0)
                    })
                >
                    "Max"
                </HfButton>
            </div>
            <div class="hf-qty-chips-row flex items-center gap-1">
                <For
                    each=chip_indices
                    key=|i| *i
                    children=move |i| {
                        let pct = chips[i];
                        let active = Signal::derive(move || selected_pct.get() == pct);
                        let label = format!("{:.0}%", pct);
                        view! {
                            <HfChip
                                active=active
                                on_click=Callback::new(move |_| {
                                    selected_pct.set(pct);
                                    set_percent(pct);
                                })
                            >
                                {label}
                            </HfChip>
                        }
                    }
                />
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_of_computes_quantity_from_percentage() {
        let tol = 1e-6;
        assert!((percent_of(1000.0, 25.0) - 250.0).abs() < tol);
        assert!((percent_of(1000.0, 100.0) - 1000.0).abs() < tol);
        assert!((percent_of(0.0, 50.0) - 0.0).abs() < tol);
        assert!((percent_of(100.0, 0.0) - 0.0).abs() < tol);
    }
}
