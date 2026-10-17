//! 256-Color Palette Studio and Inspector Modal.

use crate::theme::palette::{ColorToken, get_palette_256};
use crate::ui::HfModal;
use leptos::prelude::*;

#[component]
pub fn ThemeStudioModal(is_open: RwSignal<bool>) -> impl IntoView {
    let tokens = get_palette_256();
    let selected_token = RwSignal::new(Option::<ColorToken>::None);

    let on_close = Callback::new(move |_| {
        is_open.set(false);
    });

    view! {
        <HfModal
            title="256-Color Palette Studio & Theme Inspector"
            is_open=is_open.get()
            on_close=on_close
        >
            <div style="display: flex; flex-direction: column; gap: 12px;">
                <p style="color: var(--shade-6); font-size: 12px; margin: 0;">
                    "Every color token in honba-kao is canonicalized into the 256-token space (--c-000 to --c-255). Custom themes override these tokens or their semantic aliases."
                </p>

                <div class="palette-grid">
                    {tokens.into_iter().map(|tok| {
                        let idx = tok.index;
                        let var_name = format!("var(--c-{:03})", idx);
                        let tok_clone = tok.clone();

                        view! {
                            <div
                                class="palette-swatch"
                                style=format!("background-color: {};", var_name)
                                title=format!("#{}: {}", idx, tok.name)
                                on:click=move |_| selected_token.set(Some(tok_clone.clone()))
                            />
                        }
                    }).collect_view()}
                </div>

                {move || selected_token.get().map(|tok| {
                    let var_str = format!("--c-{:03}", tok.index);
                    view! {
                        <div style="background: var(--background-color); border: 1px solid var(--border-color); border-radius: 5px; padding: 10px; display: flex; justify-content: space-between; align-items: center;">
                            <div>
                                <strong style="color: var(--shade-9); font-size: 13px;">{tok.name}</strong>
                                <span style="margin-left: 8px; color: var(--shade-5); font-family: monospace;">{var_str}</span>
                            </div>
                            <div style="display: flex; align-items: center; gap: 8px;">
                                <span class="tnum" style="color: var(--shade-6);">"Group: " {tok.group}</span>
                                <div style=format!("width: 20px; height: 20px; border-radius: 3px; background-color: var(--c-{:03}); border: 1px solid var(--border-color);", tok.index) />
                            </div>
                        </div>
                    }
                })}
            </div>
        </HfModal>
    }
}
