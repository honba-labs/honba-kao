//! One workspace pane: symbol / timeframe / mode controls on top, the
//! openalgo-charts canvas below, applied-indicator chips on the header.
//!
//! React's `OpenAlgoChart` wrapper re-mounts its widget when pane config
//! changes (`key={symbol}:{timeframe}`). The equivalent here: the mount
//! effect tracks the pane config and destroys + remounts the bridge chart on
//! every change (the engine has no clear-series call). Applied indicators
//! survive the remount because their state lives in the store keyed by pane
//! id and is re-pushed (by instance id) onto every fresh chart.

use core::time::Duration;

use leptos::prelude::*;
use wasm_bindgen::JsValue;

use super::dismiss_on_outside_click;
use super::layouts::{area_name, preset_for};
use crate::components::charts::openalgo_chart as bridge;
use crate::stores::workbench::{
    AppliedIndicator, ChartMode, ChartPaneConfig, PaneField, Timeframe, WorkbenchStore, pane_bars,
};

// ---------------------------------------------------------------------------
// Workspace panes
// ---------------------------------------------------------------------------

/// Workspace pane: config bar + chart canvas. Clicking it makes it the
/// active pane (the order ticket then tracks it).
#[component]
pub(crate) fn ChartPane(index: usize, store: WorkbenchStore) -> impl IntoView {
    let pane = Memo::new(move |_| store.panes.get().get(index).cloned().unwrap_or_default());
    let pane_id = format!("pane-{index}");

    // One handle signal per pane, created up front so the header chips and
    // the chart host share it (the registry owns it for the whole session).
    let chart_handle = chart_handle_for(&pane_id);

    let is_active = move || store.active_pane.get() == index;
    // Panes beyond the current preset's cell count are parked, not shown.
    let is_parked = move || index >= preset_for(store.layout.get()).cell_count;

    view! {
        <div
            class=move || {
                if is_active() {
                    "wb-pane wb-pane--active"
                } else {
                    "wb-pane"
                }
            }
            class:hidden-cell=is_parked
            style:grid-area=area_name(index)
            on:click=move |_| store.set_active_pane(index)
        >
            <PaneHeader store=store index=index pane=pane chart_handle=chart_handle />
            <PaneChart store=store index=index pane=pane chart_handle=chart_handle />
        </div>
    }
}

// ---------------------------------------------------------------------------
// Pane header: symbol / timeframe pills / chart-type / indicator chips
// ---------------------------------------------------------------------------

#[component]
fn PaneHeader(
    store: WorkbenchStore,
    index: usize,
    pane: Memo<ChartPaneConfig>,
    chart_handle: RwSignal<Option<f64>>,
) -> impl IntoView {
    let pane_id = format!("pane-{index}");
    let applied = store.pane_indicators(pane_id.clone());

    view! {
        <div class="wb-pane__bar">
            <div class="wb-pane__symbol">
                <button
                    class="wb-chip wb-chip--symbol"
                    title="Search symbol"
                    on:click=move |_| store.open_symbol_search(Some(index))
                >
                    {move || pane.get().symbol.clone()}
                    <span class="wb-caret">"▾"</span>
                </button>
            </div>

            // One compact pill per unit group instead of 18 inline timeframes
            // (real-estate: the header row stays one line at four panes).
            <TimeframeMenu store=store index=index current=pane />

            <div class="wb-pane__modes">
                {ChartMode::ALL
                    .iter()
                    .filter(|mode| mode.is_pane_pill())
                    .map(|mode| {
                        view! { <ModePill store=store index=index mode=*mode current=pane /> }
                    })
                    .collect_view()}

                <ChartTypeMenu store=store index=index current=pane />

                <IndicatorChips
                    store=store
                    index=index
                    pane_id=pane_id
                    applied=applied
                    chart_handle=chart_handle
                />
            </div>
        </div>
    }
}

/// The timeframe control: a single pill showing the active interval that
/// opens the grouped menu (Minutes / Hours / Days / Weeks / Months).
#[component]
fn TimeframeMenu(store: WorkbenchStore, index: usize, current: Memo<ChartPaneConfig>) -> impl IntoView {
    let menu_open = RwSignal::new(false);
    let menu_host = NodeRef::<leptos::html::Div>::new();
    dismiss_on_outside_click(menu_open, menu_host);

    let is_open = move || menu_open.get();

    view! {
        <div class="wb-pmenu wb-pmenu--timeframe" node_ref=menu_host>
            <button
                class=move || {
                    if is_open() {
                        "wb-pill wb-pill--on wb-pill--timeframe"
                    } else {
                        "wb-pill wb-pill--timeframe"
                    }
                }
                title="Interval"
                on:click=move |_| menu_open.update(|open| *open = !*open)
            >
                {move || current.get().timeframe.label()}
                <span class="wb-caret">"▾"</span>
            </button>
            <Show when=is_open>
                <div class="wb-menu wb-menu--pane wb-menu--grouped" role="menu">
                    {Timeframe::GROUPS
                        .iter()
                        .map(|(group, timeframes)| {
                            let group = group.to_string();
                            let items = timeframes.iter().map(|timeframe| {
                                let timeframe = *timeframe;
                                let active = move || current.get().timeframe == timeframe;
                                view! {
                                    <button
                                        class=move || {
                                            if active() {
                                                "wb-menu__item wb-menu__item--selected wb-menu__item--compact"
                                            } else {
                                                "wb-menu__item wb-menu__item--compact"
                                            }
                                        }
                                        role="menuitem"
                                        title=timeframe.full_label()
                                        on:click=move |_| {
                                            menu_open.set(false);
                                            store.update_pane(index, PaneField::Timeframe(timeframe));
                                        }
                                    >
                                        {timeframe.label()}
                                    </button>
                                }
                            }).collect_view();

                            view! {
                                <div class="wb-menu__group">
                                    <span class="wb-menu__group-label">{group}</span>
                                    <div class="wb-menu__group-items">{items}</div>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </Show>
        </div>
    }
}

/// A quick chart-mode pill (candles / bars / area; the rest in Type menu).
#[component]
fn ModePill(
    store: WorkbenchStore,
    index: usize,
    mode: ChartMode,
    current: Memo<ChartPaneConfig>,
) -> impl IntoView {
    let selected = move || current.get().mode == mode;
    view! {
        <button
            class=move || {
                if selected() { "wb-pill wb-pill--on" } else { "wb-pill" }
            }
            title=mode.label()
            on:click=move |_| {
                store.update_pane(index, PaneField::Mode(mode));
            }
        >
            {short_mode(mode)}
        </button>
    }
}

/// Compact pill labels for the quick mode row.
fn short_mode(mode: ChartMode) -> &'static str {
    match mode {
        ChartMode::Candles => "Candles",
        ChartMode::Bar => "Bars",
        ChartMode::Area => "Area",
        other => other.label(),
    }
}

/// The "chart type" dropdown exposing every engine series kind.
#[component]
fn ChartTypeMenu(store: WorkbenchStore, index: usize, current: Memo<ChartPaneConfig>) -> impl IntoView {
    let menu_open = RwSignal::new(false);
    let menu_host = NodeRef::<leptos::html::Div>::new();
    dismiss_on_outside_click(menu_open, menu_host);

    let is_open = move || menu_open.get();

    view! {
        <div class="wb-pane__type-menu" node_ref=menu_host>
            <button
                class="wb-pill"
                title="Chart type"
                on:click=move |_| menu_open.update(|open| *open = !*open)
            >
                "Type" <span class="wb-caret">"▾"</span>
            </button>
            <Show when=is_open>
                <div class="wb-menu wb-menu--pane" role="menu">
                    {ChartMode::ALL
                        .iter()
                        .map(|mode| {
                            let mode = *mode;
                            let selected = current.get_untracked().mode == mode;
                            let item_class = if selected {
                                "wb-menu__item wb-menu__item--selected"
                            } else {
                                "wb-menu__item"
                            };
                            view! {
                                <button
                                    class=item_class
                                    role="menuitem"
                                    on:click=move |_| {
                                        menu_open.set(false);
                                        store.update_pane(index, PaneField::Mode(mode));
                                    }
                                >
                                    <strong>{mode.label()}</strong>
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
            </Show>
        </div>
    }
}

/// Indicator chips on the pane header — compact: at most three names plus a
/// `+N` chip; each chip carries its own remove button.
#[component]
fn IndicatorChips(
    store: WorkbenchStore,
    index: usize,
    pane_id: String,
    applied: Memo<Vec<AppliedIndicator>>,
    chart_handle: RwSignal<Option<f64>>,
) -> impl IntoView {
    let has_more = move || applied.get().len() > 3;
    view! {
        <div class="wb-pane__indicators">
            {move || {
                applied.get().iter().take(3).map(|item| {
                    let name = item.name.clone();
                    let title = name.clone();
                    let instance_id = item.instance_id.clone();
                    let pane_id = pane_id.clone();
                    view! {
                        <span class="wb-indicator-chip" title=title>
                            <span class="wb-indicator-chip__name">{name}</span>
                            <button
                                class="wb-indicator-chip__remove"
                                title="Remove indicator"
                                on:click=move |_| {
                                    if let Some(id) = chart_handle.get_untracked() {
                                        bridge::remove_indicator(id, &instance_id);
                                    }
                                    store.remove_pane_indicator(&pane_id, &instance_id);
                                }
                            >
                                "×"
                            </button>
                        </span>
                    }
                }).collect_view()
            }}
            <Show when=has_more>
                <span class="wb-indicator-chip" title="More indicators applied">
                    <span class="wb-indicator-chip__name">
                        {format!("+{}", applied.get().len() - 3)}
                    </span>
                </span>
            </Show>
            <button
                class="wb-pill wb-pill--add-indicator"
                title="Indicators"
                on:click=move |_| store.open_indicator_picker(index)
            >
                "ƒx"
            </button>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Chart canvas
// ---------------------------------------------------------------------------

/// The chart host: an openalgo-charts canvas bound through the bridge, keyed
/// on the pane config.
#[component]
fn PaneChart(
    store: WorkbenchStore,
    index: usize,
    pane: Memo<ChartPaneConfig>,
    chart_handle: RwSignal<Option<f64>>,
) -> impl IntoView {
    let host = NodeRef::<leptos::html::Div>::new();
    let handle = chart_handle;
    let attempts = RwSignal::new(0u32);
    let pane_id = format!("pane-{index}");
    let applied = store.pane_indicators(pane_id.clone());

    let spec =
        Memo::new(move |_| pane.with(|pane| (pane.symbol.clone(), pane.timeframe, pane.mode)));

    Effect::new(move |_| {
        let _ = attempts.get();
        let (symbol, timeframe, mode) = spec.get();

        let Some(element) = host.get() else {
            return;
        };

        // Remount on every spec change: cheap enough at 360 bars, and it
        // sidesteps the engine's missing clear-series call.
        if let Some(id) = handle.get_untracked() {
            bridge::destroy_chart(id);
            handle.set(None);
        }

        let bars = pane_bars(&symbol, timeframe);
        let (kind, style) = series_style(mode);

        match bridge::mount_chart(&JsValue::from(element)) {
            Ok(id) => {
                let _ = bridge::push_chart_series(id, &kind, &style, &bridge::encode_bars(&bars));
                handle.set(Some(id));
            }
            // The ESM module may still be loading; retry briefly, matching
            // the static OpenAlgoChart behavior.
            Err(_) if attempts.get_untracked() < 120 => {
                set_timeout(
                    move || attempts.update(|count| *count += 1),
                    Duration::from_millis(25),
                );
            }
            Err(error) => {
                leptos::logging::error!("openalgo-charts bridge unavailable: {error:?}");
            }
        }
    });

    // Applied indicators onto every fresh chart (the remount destroys them).
    // Every item carries a stable instance id (assigned when the store slot
    // was created) that the engine accepts, so chips keep working afterwards.
    Effect::new(move |_| {
        for item in applied.get() {
            if let Some(id) = handle.get_untracked() {
                let _ = bridge::add_indicator(
                    id,
                    &item.indicator_id,
                    &serde_json::json!({}),
                    Some(&item.instance_id),
                );
            }
        }
    });

    on_cleanup(move || {
        if let Some(id) = handle.get_untracked() {
            bridge::destroy_chart(id);
        }
    });

    view! { <div class="wb-pane__chart-host" node_ref=host /> }
}

/// Map a pane's series mode onto `(addSeries kind, style JSON)`.
fn series_style(mode: ChartMode) -> (String, String) {
    let style = match mode {
        // Candle-ish kinds take the theme up/down colours; an empty override
        // keeps the bridge defaults.
        ChartMode::Candles
        | ChartMode::HollowCandle
        | ChartMode::VolumeCandle
        | ChartMode::Bar
        | ChartMode::HighLow => serde_json::json!({}),
        // Line / area families default to the theme line colour too.
        ChartMode::Line
        | ChartMode::LineMarkers
        | ChartMode::Step
        | ChartMode::Area
        | ChartMode::HlcArea
        | ChartMode::Baseline
        | ChartMode::Column
        | ChartMode::Histogram => serde_json::json!({}),
    };

    (mode.as_id().to_string(), style.to_string())
}

// ---------------------------------------------------------------------------
// Shared chart handles (chips remove indicators off the live chart)
// ---------------------------------------------------------------------------

thread_local! {
    static CHART_HANDLES: std::cell::RefCell<std::collections::HashMap<String, RwSignal<Option<f64>>>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// The live bridge handle signal for one pane (`None` = chart not mounted).
pub(crate) fn chart_handle_for(pane_id: &str) -> RwSignal<Option<f64>> {
    CHART_HANDLES.with(|map| {
        let mut map = map.borrow_mut();
        if let Some(handle) = map.get(pane_id) {
            *handle
        } else {
            let handle = RwSignal::new(None);
            map.insert(pane_id.to_string(), handle);
            handle
        }
    })
}
