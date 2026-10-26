//! Multi-pane chart page. Each pane owns a symbol, a timeframe and a chart
//! host mounted through the openalgo-charts bridge. Layout presets decide how
//! many panes are rendered; pane state lives on the page so it survives a
//! layout change.

use core::time::Duration;

use honba_kao_ui_core::{
    HfButton, HfInput, HfKpi, HfPageHeader, HfPanel, HfPanelHeader, HfToggleGroup,
    HfToggleGroupItem,
};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::shared::components::charts::openalgo_chart as bridge;

const SNAPSHOT_KEY: &str = "cw:snapshot:v1";
const PANE_COUNT: usize = 4;
const PANE_TITLES: [&str; PANE_COUNT] = ["Pane 1", "Pane 2", "Pane 3", "Pane 4"];
const DEFAULT_SYMBOLS: [&str; PANE_COUNT] = ["NIFTY", "RELIANCE", "TCS", "INFY"];
const TIMEFRAMES: [&str; 3] = ["1m", "5m", "1D"];
const DEFAULT_TIMEFRAME: &str = "1D";
const CHART_MODE: &str = "candlestick";
const MAX_MOUNT_ATTEMPTS: u32 = 120;

/// Grid presets. `cells` is how many of the four pane slots are shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Layout {
    #[default]
    Single,
    Cols2,
    Rows2,
    Grid4,
}

impl Layout {
    fn id(self) -> &'static str {
        match self {
            Layout::Single => "single",
            Layout::Cols2 => "cols2",
            Layout::Rows2 => "rows2",
            Layout::Grid4 => "grid4",
        }
    }

    fn cells(self) -> usize {
        match self {
            Layout::Single => 1,
            Layout::Cols2 | Layout::Rows2 => 2,
            Layout::Grid4 => 4,
        }
    }
}

/// Per-pane state owned by the page. `Copy` so it can be handed to every
/// pane and closure without cloning.
#[derive(Clone, Copy)]
struct PaneState {
    symbol: RwSignal<String>,
    timeframe: RwSignal<&'static str>,
}

/// Wire shape of one pane inside the saved workspace snapshot.
#[derive(Serialize, Deserialize)]
struct PaneSnapshot {
    symbol: String,
    timeframe: String,
}

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn save_snapshot(panes: &[PaneState; PANE_COUNT]) -> Result<(), String> {
    let snapshot: Vec<PaneSnapshot> = panes
        .iter()
        .map(|pane| PaneSnapshot {
            symbol: pane.symbol.get_untracked(),
            timeframe: pane.timeframe.get_untracked().to_string(),
        })
        .collect();
    let json = serde_json::to_string(&snapshot).map_err(|e| e.to_string())?;
    let storage = local_storage().ok_or("localStorage unavailable")?;
    storage
        .set_item(SNAPSHOT_KEY, &json)
        .map_err(|e| format!("{e:?}"))
}

fn load_snapshot(panes: &[PaneState; PANE_COUNT]) -> Result<(), String> {
    let storage = local_storage().ok_or("localStorage unavailable")?;
    let raw = storage
        .get_item(SNAPSHOT_KEY)
        .map_err(|e| format!("{e:?}"))?
        .ok_or("no saved layout")?;
    let snapshot: Vec<PaneSnapshot> = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    for (pane, saved) in panes.iter().zip(snapshot) {
        let symbol = saved.symbol.trim().to_uppercase();
        if !symbol.is_empty() {
            pane.symbol.set(symbol);
        }
        let timeframe = TIMEFRAMES
            .into_iter()
            .find(|tf| *tf == saved.timeframe)
            .unwrap_or(DEFAULT_TIMEFRAME);
        pane.timeframe.set(timeframe);
    }
    Ok(())
}

#[component]
fn ChartPane(
    index: usize,
    pane: PaneState,
    /// Bumped on snapshot load so the toolbar re-renders with loaded values.
    reload: RwSignal<u32>,
) -> impl IntoView {
    let synced_time = expect_context::<RwSignal<Option<f64>>>();
    let host = NodeRef::<leptos::html::Div>::new();
    let handle = RwSignal::new(None::<f64>);
    let epoch = RwSignal::new(0u64);
    let attempts = RwSignal::new(0u32);

    // Remount the chart whenever the pane's symbol or timeframe changes.
    Effect::new(move |_| {
        let _ = attempts.get();
        let symbol = pane.symbol.get();
        let timeframe = pane.timeframe.get();

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
        <HfPanel class="cw-pane">
            <HfPanelHeader title=PANE_TITLES[index] counter=index + 1 />
            <HfPanel class="cw-pane__toolbar">
                {move || {
                    let _ = reload.get();
                    let initial_symbol = pane.symbol.get_untracked();
                    let initial_timeframe = pane.timeframe.get_untracked();
                    view! {
                        <HfInput
                            value=initial_symbol
                            class="cw-pane__symbol"
                            on_change=Callback::new(move |ev: web_sys::Event| {
                                let typed = event_target_value(&ev).trim().to_uppercase();
                                if !typed.is_empty() {
                                    pane.symbol.set(typed);
                                }
                            })
                        />
                        <HfToggleGroup default_value=initial_timeframe class="cw-pane__tf">
                            {TIMEFRAMES
                                .into_iter()
                                .map(|tf| {
                                    view! {
                                        <HfToggleGroupItem
                                            value=tf
                                            on_select=Callback::new(move |picked: &'static str| {
                                                pane.timeframe.set(picked);
                                            })
                                        >
                                            {tf}
                                        </HfToggleGroupItem>
                                    }
                                })
                                .collect_view()}
                        </HfToggleGroup>
                    }
                }}
            </HfPanel>
            <div
                node_ref=host
                class="cw-pane__chart"
                on:mousemove=move |ev: web_sys::MouseEvent| {
                    // Approximate shared crosshair: the pointer's horizontal
                    // position as a fraction of this pane's width.
                    let width = host.get_untracked().map(|el| el.client_width()).unwrap_or(0);
                    if width > 0 {
                        let fraction = f64::from(ev.offset_x()) / f64::from(width);
                        synced_time.set(Some(fraction.clamp(0.0, 1.0)));
                    }
                }
                on:mouseleave=move |_| synced_time.set(None)
            ></div>
        </HfPanel>
    }
}

#[component]
pub fn ChartWorkstationPage() -> impl IntoView {
    let layout = RwSignal::new(Layout::default());
    let reload = RwSignal::new(0u32);
    let status = RwSignal::new(String::new());
    let synced_time = RwSignal::new(None::<f64>);
    provide_context(synced_time);

    let panes: [PaneState; PANE_COUNT] = std::array::from_fn(|index| PaneState {
        symbol: RwSignal::new(DEFAULT_SYMBOLS[index].to_string()),
        timeframe: RwSignal::new(DEFAULT_TIMEFRAME),
    });

    let on_save = Callback::new(move |_: web_sys::MouseEvent| {
        status.set(match save_snapshot(&panes) {
            Ok(()) => "Layout saved".to_string(),
            Err(error) => format!("Save failed: {error}"),
        });
    });
    let on_load = Callback::new(move |_: web_sys::MouseEvent| match load_snapshot(&panes) {
        Ok(()) => {
            reload.update(|count| *count += 1);
            status.set("Layout loaded".to_string());
        }
        Err(error) => status.set(format!("Load failed: {error}")),
    });

    view! {
        <HfPageHeader
            title="Chart Workstation"
            subtitle="Multi-pane charts with shared crosshair and saved workspace snapshots"
        />

        <HfPanel class="cw-toolbar">
            <HfPanelHeader title="Layout" />
            <HfToggleGroup default_value="single">
                <HfToggleGroupItem
                    value="single"
                    on_select=Callback::new(move |_| layout.set(Layout::Single))
                >
                    "1 x 1"
                </HfToggleGroupItem>
                <HfToggleGroupItem
                    value="cols2"
                    on_select=Callback::new(move |_| layout.set(Layout::Cols2))
                >
                    "1 | 2"
                </HfToggleGroupItem>
                <HfToggleGroupItem
                    value="rows2"
                    on_select=Callback::new(move |_| layout.set(Layout::Rows2))
                >
                    "1 / 2"
                </HfToggleGroupItem>
                <HfToggleGroupItem
                    value="grid4"
                    on_select=Callback::new(move |_| layout.set(Layout::Grid4))
                >
                    "2 x 2"
                </HfToggleGroupItem>
            </HfToggleGroup>
            <HfButton on_click=on_save>"Save layout"</HfButton>
            <HfButton on_click=on_load>"Load layout"</HfButton>
            <HfKpi label="Crosshair">
                {move || match synced_time.get() {
                    Some(fraction) => format!("{:.0}%", fraction * 100.0),
                    None => "-".to_string(),
                }}
            </HfKpi>
            <HfKpi label="Status">{move || status.get()}</HfKpi>
        </HfPanel>

        {move || {
            let current = layout.get();
            view! {
                <HfPanel class=format!("cw-grid cw-grid--{}", current.id())>
                    {(0..current.cells())
                        .map(|index| {
                            view! { <ChartPane index=index pane=panes[index] reload=reload /> }
                        })
                        .collect_view()}
                </HfPanel>
            }
        }}
    }
}
