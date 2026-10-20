//! 256-colour palette studio + theme inspector.
//!
//! The modal is mounted only while `is_open` is set, so the 256 swatches are
//! constructed on demand and open/close is genuinely reactive.

use leptos::prelude::*;

use honba_kao_ui_core::composed::modal::HfModal;
use crate::theme::palette::{ColorToken, get_palette_256};

#[component]
pub fn PaletteStudio(
    /// Read-only open flag — the flag lives in the chrome-level store.
    is_open: Signal<bool>,
    /// Closes the modal via the same store that opened it.
    on_close: Callback<()>,
) -> impl IntoView {
    let tokens = get_palette_256();
    let selected = RwSignal::new(Option::<ColorToken>::None);

    view! {
        {move || is_open.get().then(|| {
            let tokens = tokens.clone();
            view! {
                <HfModal
                    title="256-colour palette studio"
                    on_close=on_close
                >
                    <div class="palette-studio">
                        <p class="palette-studio__hint">
                            "Every token in honba-kao canonicalises into a 256-token space (--c-000 … --c-255). Themes override these tokens or their semantic aliases."
                        </p>

                        <div class="palette-grid">
                            {tokens
                                .into_iter()
                                .map(|tok| {
                                    let idx = tok.index;
                                    let var_name = format!("var(--c-{:03})", idx);
                                    let picked = tok.clone();
                                    view! {
                                        <button
                                            class="palette-swatch"
                                            style=format!("background-color: {};", var_name)
                                            title=format!("#{}: {}", idx, tok.name)
                                            on:click=move |_| selected.set(Some(picked.clone()))
                                        />
                                    }
                                })
                                .collect_view()}
                        </div>

                        {move || selected.get().map(|tok| view! {
                            <div class="palette-studio__readout">
                                <div>
                                    <strong>{tok.name}</strong>
                                    <span class="mono">{format!("--c-{:03}", tok.index)}</span>
                                </div>
                                <div class="palette-studio__meta">
                                    <span class="mono">"group: "{tok.group}</span>
                                    <span
                                        class="palette-studio__chip"
                                        style=format!("background-color: var(--c-{:03});", tok.index)
                                    />
                                </div>
                            </div>
                        })}
                    </div>
                </HfModal>
            }
        })}
    }
}
