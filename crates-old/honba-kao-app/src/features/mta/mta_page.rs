//! Multi-Timeframe Analysis page. Four panes share one symbol; a preset
//! decides the timeframe each pane shows (higher timeframe first, lower last).

use core::time::Duration;

use honba_kao_ui_core::{
    HfInput, HfPageHeader, HfPanel, HfPanelHeader, HfToggleGroup, HfToggleGroupItem,
};
use leptos::prelude::*;
use wasm_bindgen::JsValue;

use crate::shared::components::charts::openalgo_chart as bridge;

const PANE_COUNT: usize = 4;
const PANE_TITLES: [&str; PANE_COUNT] = ["Pane 1", "Pane 2", "Pane 3", "Pane 4"];
const DEFAULT_SYMBOL: &str = "NIFTY";
const CHART_MODE: &str = "candlestick";
const MAX_MOUNT_ATTEMPTS: u32 = 120;

/// A preset maps each of the four panes to one timeframe label.
struct Preset {
    id: &'static str,
    label: &'static str,
    timeframes: [&'static str; PANE_COUNT],
}

const PRESETS: [Preset; 4] = [
    Preset {
        id: "position",
        label: "Position",
        timeframes: ["Monthly", "Weekly", "Daily", "4H"],
    },
    Preset {
        id: "swing",
        label: "Swing",
        timeframes: ["Daily", "4H", "1H", "15m"],
    },
    Preset {
        id: "day-trade",
        label: "Day-Trade",
        timeframes: ["1H", "15m", "5m", "1m"],
    },
    Preset {
        id: "scalp",
        label: "Scalp",
        timeframes: ["15m", "5m", "1m", "Tick"],
    },
];

#[component]
fn MtaPane(
    index: usize,
    symbol: RwSignal<String>,
    timeframe: Signal<&'static str>,
) -> impl IntoView {
    let host = NodeRef::<leptos::html::Div>::new();
    let handle = RwSignal::new(None::<f64>);
    let epoch = RwSignal::new(0u64);
    let attempts = RwSignal::new(0u32);

    // Remount the chart whenever the pane's symbol or timeframe changes.
    Effect::new(move |_| {
        let _ = attempts.get();
        let symbol = symbol.get();
        let timeframe = timeframe.get();

        let Some(element) = host.get() else {
            return;
        };

        if let Some(id) = handle.get_untracked() {
            bridge::destroy_chart(id);
            handle.set(None);
        }

        // Epoch guard: a mount that resolves after a newer spec change, or
        // after unmount, is destroyed instead of stored.
        let current = epoch.get_untracked() + 1;
        epoch.set(current);

        let element = JsValue::from(element);
        leptos::task::spawn_local(async move {
            match bridge::mount_chart_with_options(&element, &symbol, timeframe, CHART_MODE, 0)
                .await
            {
                Ok(id) => {
                    if epoch.try_get_untracked() == Some(current) {
                        handle.set(Some(id));
                    } else {
                        bridge::destroy_chart(id);
                    }
                }
                // The ESM module may still be loading; retry briefly.
                Err(_)
                    if epoch.try_get_untracked() == Some(current)
                        && attempts
                            .try_get_untracked()
                            .is_some_and(|count| count < MAX_MOUNT_ATTEMPTS) =>
                {
                    set_timeout(
                        move || {
                            let _ = attempts.try_update(|count| *count += 1);
                        },
                        Duration::from_millis(25),
                    );
                }
                Err(error) => {
                    leptos::logging::error!("openalgo-charts widget unavailable: {error:?}");
                }
            }
        });
    });

    on_cleanup(move || {
        let _ = epoch.try_update(|value| *value += 1);
        if let Some(id) = handle.try_get_untracked().flatten() {
            bridge::destroy_chart(id);
        }
    });

    view! {
        <HfPanel class="mta-pane">
            <HfPanelHeader title=PANE_TITLES[index] counter=index + 1>
                <span class="mta-pane__tf">{move || timeframe.get()}</span>
            </HfPanelHeader>
            <div node_ref=host class="mta-pane__chart"></div>
        </HfPanel>
    }
}

#[component]
pub fn MtaPage() -> impl IntoView {
    let symbol = RwSignal::new(DEFAULT_SYMBOL.to_string());
    let timeframes = RwSignal::new(PRESETS[0].timeframes);

    view! {
        <HfPageHeader
            title="Multi-Timeframe Analysis"
            subtitle="One symbol across four timeframes, chosen by trading horizon"
        />

        <HfPanel class="mta-toolbar">
            <HfInput
                value=DEFAULT_SYMBOL.to_string()
                class="mta-toolbar__symbol"
                on_change=Callback::new(move |ev: web_sys::Event| {
                    let typed = event_target_value(&ev).trim().to_uppercase();
                    if !typed.is_empty() {
                        symbol.set(typed);
                    }
                })
            />
            <HfToggleGroup default_value=PRESETS[0].id class="mta-toolbar__presets">
                {PRESETS
                    .iter()
                    .map(|preset| {
                        view! {
                            <HfToggleGroupItem
                                value=preset.id
                                on_select=Callback::new(move |picked: &'static str| {
                                    if let Some(found) = PRESETS.iter().find(|p| p.id == picked) {
                                        timeframes.set(found.timeframes);
                                    }
                                })
                            >
                                {preset.label}
                            </HfToggleGroupItem>
                        }
                    })
                    .collect_view()}
            </HfToggleGroup>
        </HfPanel>

        <div class="mta-grid">
            {(0..PANE_COUNT)
                .map(|index| {
                    let timeframe = Signal::derive(move || timeframes.get()[index]);
                    view! { <MtaPane index=index symbol=symbol timeframe=timeframe /> }
                })
                .collect_view()}
        </div>
    }
}
