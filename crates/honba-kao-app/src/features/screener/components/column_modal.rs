//! `ScreenerColumnModal` — per-column visibility toggles bound to the page's
//! `column_flags` signals; toggles update the results table instantly.

use honba_kao_ui_core::primitive::{ButtonSize, ButtonVariant, HfButton};
use honba_kao_ui_core::{HfCheckbox, HfDialog};
use leptos::prelude::*;

#[component]
pub fn ScreenerColumnModal(
    open: RwSignal<bool>,
    column_flags: Vec<(&'static str, RwSignal<bool>)>,
) -> impl IntoView {
    view! {
        <HfDialog open=open title="Column Configuration">
            <div class="flex flex-col gap-3">
                <div class="grid grid-cols-2 gap-2 p-1">
                    {column_flags
                        .iter()
                        .map(|(k, f)| {
                            view! { <HfCheckbox checked=*f label=*k /> }
                        })
                        .collect::<Vec<_>>()}
                </div>
                <div class="flex justify-end">
                    <HfButton
                        variant=ButtonVariant::Primary
                        size=ButtonSize::Xs
                        on_click=Callback::new(move |_| open.set(false))
                    >
                        "Done"
                    </HfButton>
                </div>
            </div>
        </HfDialog>
    }
}
