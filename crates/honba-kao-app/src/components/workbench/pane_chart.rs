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
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;

use super::dismiss_on_outside_click;
use super::drawing_rail::DrawingRail;
use super::layouts::{area_name, preset_for};
use crate::components::charts::openalgo_chart as bridge;
use crate::stores::workbench::{
    AppliedIndicator, ChartMode, ChartPaneConfig, PaneField, Timeframe, WorkbenchStore, pane_bars,
};
use crate::types::Bar;

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

            // TradingView-shaped interval chips: `1m 1d 1W 1M`, each opening
            // its denomination's flyout (plus a `Custom…` input row).
            <IntervalBar store=store index=index current=pane />

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

/// The interval control: four TradingView-style chips (`1m 1d 1W 1M`) whose
/// labels snap to the active interval when it belongs to their group. One
/// click on a chip opens that denomination's flyout — the root interval,
/// its siblings, and a `Custom…` row accepting `7m` / `2h` / `3d` / a bare
/// minute count.
#[component]
fn IntervalBar(
    store: WorkbenchStore,
    index: usize,
    current: Memo<ChartPaneConfig>,
) -> impl IntoView {
    view! {
        <div class="wb-interval-bar">
            {Timeframe::INTERVAL_GROUPS
                .iter()
                .map(|(name, root, items)| {
                    view! {
                        <IntervalChip
                            store
                            index
                            current
                            group_name=*name
                            root=*root
                            items=*items
                        />
                    }
                })
                .collect_view()}
        </div>
    }
}

/// One interval chip plus its flyout. The four chips are independent
/// components with their own open state — the shared `dismiss_on_outside_click`
/// keeps them mutually exclusive (a mousedown elsewhere closes the last one
/// before the new chip's click lands).
#[component]
fn IntervalChip(
    store: WorkbenchStore,
    index: usize,
    current: Memo<ChartPaneConfig>,
    group_name: &'static str,
    root: Timeframe,
    items: &'static [Timeframe],
) -> impl IntoView {
    let menu_open = RwSignal::new(false);
    let custom_editing = RwSignal::new(false);
    let custom_text = RwSignal::new(String::new());
    let menu_host = NodeRef::<leptos::html::Div>::new();
    dismiss_on_outside_click(menu_open, menu_host);

    let is_open = move || menu_open.get();

    // The chip reads the active interval when it lives in this group's list,
    // so an active `15m` shows under the `1m` chip and `3D` under `1d`.
    let chip_label = move || {
        let frame = current.get().timeframe;
        if items.contains(&frame) || frame == root {
            frame.label()
        } else {
            root.label()
        }
    };

    let commit_custom = move || {
        let text = custom_text.get_untracked();
        if let Some(timeframe) = Timeframe::parse_custom(&text) {
            custom_editing.set(false);
            menu_open.set(false);
            store.update_pane(index, PaneField::Timeframe(timeframe));
        } else {
            // The parse refused it; leave the field up so it can be edited.
        }
    };

    view! {
        <div class="wb-pmenu wb-pmenu--timeframe" node_ref=menu_host>
            <button
                class=move || {
                    if items.contains(&current.get().timeframe) {
                        "wb-pill wb-pill--on wb-pill--timeframe"
                    } else if is_open() {
                        "wb-pill wb-pill--open wb-pill--timeframe"
                    } else {
                        "wb-pill wb-pill--timeframe"
                    }
                }
                title="Interval"
                on:click=move |_| menu_open.update(|open| *open = !*open)
            >
                {chip_label}
                <span class="wb-caret">"▾"</span>
            </button>
            <Show when=is_open>
                <div class="wb-menu wb-menu--pane wb-menu--grouped" role="menu">
                    <div class="wb-menu__group">
                        <span class="wb-menu__group-label">{group_name}</span>
                        <div class="wb-menu__group-items">
                            {items.iter().map(|timeframe| {
                                let active = move || current.get().timeframe == *timeframe;
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
                                            store.update_pane(index, PaneField::Timeframe(*timeframe));
                                        }
                                    >
                                        {timeframe.label()}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                    <Show
                        when=move || custom_editing.get()
                        fallback=move || {
                            view! {
                                <button
                                    class="wb-menu__item wb-menu__custom-row"
                                    role="menuitem"
                                    title="Custom interval: 7m, 2h, 3d, 2w, 2M — a bare number counts minutes"
                                    on:click=move |_| {
                                        custom_text.set(String::new());
                                        custom_editing.set(true);
                                    }
                                >
                                    "Custom…"
                                </button>
                            }
                        }
                    >
                        <div class="wb-menu__custom-row">
                            <input
                                class="wb-menu__custom-input"
                                type="text"
                                placeholder="7m · 2h · 3d"
                                prop:value=custom_text
                                on:input=move |event| {
                                    let input = event_target::<web_sys::HtmlInputElement>(&event);
                                    custom_text.set(input.value());
                                }
                                on:keydown=move |event| {
                                    if event.key() == "Enter" {
                                        commit_custom();
                                    } else if event.key() == "Escape" {
                                        custom_editing.set(false);
                                    }
                                }
                            />
                            <button class="wb-menu__custom-apply" title="Apply interval" on:click=move |_| commit_custom()>
                                "OK"
                            </button>
                        </div>
                    </Show>
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
fn ChartTypeMenu(
    store: WorkbenchStore,
    index: usize,
    current: Memo<ChartPaneConfig>,
) -> impl IntoView {
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
                // The workbench extras ride every fresh chart: volume overlay
                // (from the bars just pushed), the corner Buy/Sell bubble and
                // the live 2s tick streamer seeded from the last bar.
                bridge::add_volume_overlay(id);
                bridge::add_buy_sell_bubble(id, store.ticket.get_untracked().quantity, &symbol);
                // Drawing persistence publishes the instrument before any
                // `data:context` reaches the engine's `InstrumentDrawings`.
                bridge::set_symbol_context(id, &symbol);
                let ticker = start_tick_streamer(
                    id,
                    &symbol,
                    bars.last().cloned(),
                    timeframe.interval_seconds(),
                );
                on_cleanup(move || {
                    if let Some(id) = ticker {
                        cancel_interval(id);
                    }
                });
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

    // The bubble's qty label follows the order ticket.
    Effect::new(move |_| {
        let quantity = store.ticket.get().quantity;
        if let Some(id) = handle.get_untracked() {
            bridge::set_bubble_quantity(id, quantity);
        }
    });

    on_cleanup(move || {
        if let Some(id) = handle.get_untracked() {
            bridge::destroy_chart(id);
        }
    });

    view! {
        <div class="wb-pane__chart-wrap">
            <div class="wb-pane__chart-host" node_ref=host />
            <DrawingRail chart_handle=handle />
        </div>
    }
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

// ---------------------------------------------------------------------------
// Live tick streamer (labs' `subscribeBars`, adapted)
// ---------------------------------------------------------------------------

/// Deterministic seeded PRNG (labs' mulberry32).
struct TickRng(u32);

impl TickRng {
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x6d2b79f5);
        let s = self.0;
        let mut t = (s ^ (s >> 15)).wrapping_mul(1 | s);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
        ((t ^ (t >> 14)) as f64) / 4294967296.0
    }
}

/// One pane's in-flight bar plus its pump state. Owned by the interval
/// callback, cancelled with the returned handle on remount.
struct Ticker {
    chart_id: f64,
    bar: Bar,
    step: i64,
    rng: TickRng,
}

impl Ticker {
    fn new(chart_id: f64, bar: Bar, step: i64, seed: u32) -> Self {
        Self {
            chart_id,
            bar,
            step,
            rng: TickRng(seed),
        }
    }

    /// One pump tick: either update the in-flight bar in place or roll a
    /// fresh one for the new interval bucket, then repaint the chart.
    fn tick(&mut self) {
        let now = (js_sys::Date::now() / 1000.0) as i64;
        let bucket = now.div_euclid(self.step) * self.step;

        if bucket > self.bar.time {
            // Interval rolled over: start a new bar with continuous price.
            let open = self.bar.close;
            let delta = (self.rng.next() - 0.495) * (open * 0.0008);
            let close = (open + delta).max(0.1);
            let high = open.max(close).max(open + self.rng.next() * open * 0.0004);
            let low = open.min(close).min(open - self.rng.next() * open * 0.0004);
            let volume = 100.0 + (self.rng.next() * 500.0).floor();
            self.bar = Bar::new(
                bucket,
                round2(open),
                round2(high),
                round2(low),
                round2(close),
                volume,
            );
        } else {
            // Update the current in-flight candle with realistic micro-ticks.
            let delta = (self.rng.next() - 0.495) * (self.bar.close * 0.0006);
            let close = (self.bar.close + delta).max(0.1);
            self.bar.close = close;
            self.bar.high = self.bar.high.max(close);
            self.bar.low = self.bar.low.min(close);
            self.bar.volume += 10.0 + (self.rng.next() * 50.0).floor();
        }

        bridge::update_chart_last(self.chart_id, &self.bar);
    }
}

/// labs' `hashString`, for a stable per-instrument stream seed.
fn seed_for(symbol: &str, step: i64) -> u32 {
    let mut hash: u32 = 0;
    for byte in symbol.to_uppercase().bytes() {
        hash = (hash << 5).wrapping_sub(hash).wrapping_add(byte as u32);
    }
    hash ^ (step as u32)
}

/// Start the 2s live pump for one freshly mounted chart (labs pumps every
/// 2s regardless of the interval — bars bucket on `interval_seconds`).
///
/// Returns the interval id so the pane effect cancels it on remount.
fn start_tick_streamer(
    chart_id: f64,
    symbol: &str,
    seed: Option<Bar>,
    step_seconds: i64,
) -> Option<i32> {
    let bar = seed?;
    if step_seconds <= 0 {
        return None;
    }

    let state = std::cell::RefCell::new(Ticker::new(
        chart_id,
        bar,
        step_seconds,
        seed_for(symbol, step_seconds),
    ));

    let callback = Closure::<dyn FnMut()>::new(move || state.borrow_mut().tick());

    web_sys::window()
        .and_then(|window| {
            window
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    2_000,
                )
                .ok()
        })
        .map(|handle| {
            std::mem::forget(callback); // the closure lives as long as the timer
            handle
        })
}

/// `window.clearInterval` for a handle [`start_tick_streamer`] returned.
fn cancel_interval(id: i32) {
    if let Some(window) = web_sys::window() {
        let _ = window.clear_interval_with_handle(id);
    }
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
