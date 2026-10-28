# openalgo-charts-rs

[![Crates.io](https://img.shields.io/badge/crates.io-openalgo--charts--rs-orange.svg)](https://crates.io/crates/openalgo-charts-rs)
[![Documentation](https://docs.rs/openalgo-charts-rs/badge.svg)](https://docs.rs/openalgo-charts-rs)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Idiomatic Rust bindings and reactive [Leptos 0.7](https://leptos.dev) components for **[OpenAlgo Charts](https://github.com/marketcalls/openalgo-charts)** (the high-performance TradingView Lightweight Charts fork featuring 502 built-in indicators, drawing tools, trading layers, and WebGL rendering).

Inspired by [`corebreaker/rs-trading-charts`](https://github.com/corebreaker/rs-trading-charts), but elevated with the battle-tested, sophisticated bridging from **honba-kao**:
- 🚀 **Direct `wasm-bindgen` JS API Declarations**: Strongly-typed `Chart`, `SeriesApi`, `TimeScale`, `PriceScale`, and `createChart`.
- ⚡ **Zero-Overhead Data Serialization**: Direct Rust `[Bar]` (OHLCV) and `[LineData]` serialization to WebAssembly JS values using `serde-wasm-bindgen`.
- 🔄 **Reactive Leptos Architecture**: Signal-driven data and spec updates (`symbol`, `interval`, `data`) with automatic differential updates rather than expensive canvas remounts.
- 🛡️ **Leak-Proof Lifecycle Management**: RAII `Subscription` handles and component `on_cleanup` hooks that guarantee `chart.destroy()` is called to free GPU/canvas memory.
- 🎨 **Adaptive Design Token Engine**: Dynamic DOM inspection (`theme_from_skin()`) that maps CSS custom properties (`--pnl-gain`, `--pnl-loss`, `--chart-background`, etc.) directly into `ChartTheme`.
- 📈 **502 Built-in Indicators**: Catalog discovery (`indicator_catalog()`) and live runtime indicator instance management (`add_indicator`, `remove_indicator`).
- ✏️ **Headless & UI Drawing Toolkit**: Programmatic drawing tool arming (`trendline`, `fib-retracement`, `ray`, `rectangle`, `brush`, etc.), undo/redo stacks, and symbol-scoped localStorage persistence.
- 💹 **Trade Overlays**: Corner Buy/Sell bubbles with cursor freeze hit-testing, order click event streams (`honba-order-click`), and volume overlays with dedicated secondary price margins.
- 🧩 **Multi-Framework**: Retargettable to Dioxus, Yew, or Vanilla WebAssembly via the headless `ChartController`.

---

## Installation

Add `openalgo-charts-rs` to your `Cargo.toml`:

```toml
[dependencies]
openalgo-charts-rs = { version = "0.1", features = ["leptos"] }
leptos = { version = "0.7", features = ["csr"] }
```

Ensure OpenAlgo Charts (`openalgo-charts.mjs` or `honba-bridge.js`) is loaded in your `index.html`:

```html
<script type="module" src="/vendor/openalgo-charts/openalgo-charts.mjs"></script>
```

---

## Quick Start

### 1. Simple Reactive Chart (`ChartContainer`)

Use `<ChartContainer />` for pure engine charts (dashboard widgets, equity curves, comparison panels):

```rust
use leptos::prelude::*;
use openalgo_charts::prelude::*;

#[component]
pub fn SimpleChart() -> impl IntoView {
    // Reactive bar signal
    let (bars, set_bars) = signal(vec![
        Bar::new(1700000000, 100.0, 105.0, 99.5, 104.0, 1500.0),
        Bar::new(1700086400, 104.0, 108.0, 103.0, 107.5, 2300.0),
        Bar::new(1700172800, 107.5, 112.0, 106.0, 110.0, 3100.0),
    ]);

    let on_crosshair = Callback::new(move |ev: CrosshairMoveEvent| {
        if let Some(price) = ev.price {
            leptos::logging::log!("Hovered price: {price}");
        }
    });

    view! {
        <div style="width: 100%; height: 400px;">
            <ChartContainer
                data=bars
                series_kind=SeriesKind::Candlestick
                on_crosshair=on_crosshair
            />
        </div>
    }
}
```

---

### 2. Full Terminal Experience (`OpenAlgoChart`)

Use `<OpenAlgoChart />` to mount a full terminal experience with symbol search, intervals, drawing rail, dialogs, and order ticket integration:

```rust
use leptos::prelude::*;
use openalgo_charts::prelude::*;

#[component]
pub fn TradingTerminal() -> impl IntoView {
    let (symbol, set_symbol) = signal("RELIANCE".to_string());
    let (timeframe, set_timeframe) = signal("1d".to_string());
    let chart_handle = RwSignal::new(None::<ChartHandle>);

    let on_order_click = Callback::new(move |ev: OrderClickEvent| {
        leptos::logging::log!("Order clicked: {} {} qty={}", ev.side, ev.symbol, ev.quantity);
    });

    view! {
        <div style="width: 100%; height: 100vh;">
            <OpenAlgoChart
                symbol=symbol.get()
                timeframe=timeframe.get()
                chart_mode="candlestick".to_string()
                quantity=50
                chart_handle=chart_handle
                on_order_click=on_order_click
                class="trading-chart-terminal"
            />
        </div>
    }
}
```

---

### 3. Imperative Control via `ChartHandle`

You can capture a `ChartHandle` and interact with it programmatically from anywhere in your Leptos app:

```rust
use leptos::prelude::*;
use openalgo_charts::prelude::*;

#[component]
pub fn ChartControls(chart: RwSignal<Option<ChartHandle>>) -> impl IntoView {
    let arm_trendline = move |_| {
        if let Some(c) = chart.get() {
            c.set_drawing_tool("trendline");
        }
    };

    let undo = move |_| {
        if let Some(c) = chart.get() {
            c.undo_drawing();
        }
    };

    let add_rsi = move |_| {
        if let Some(c) = chart.get() {
            let _ = c.add_indicator("rsi", &serde_json::json!({ "period": 14 }), None);
        }
    };

    view! {
        <div class="toolbar">
            <button on:click=arm_trendline>"Trendline"</button>
            <button on:click=undo>"Undo"</button>
            <button on:click=add_rsi>"+ RSI"</button>
        </div>
    }
}
```

---

### 4. Retargeting to Dioxus (`ChartController`)

Because `openalgo-charts` has a completely framework-agnostic core (`ChartController`, `ChartHandle`, `SeriesHandle`, and `Bar`), you can retarget it to **Dioxus** with `default-features = false`:

```toml
[dependencies]
openalgo-charts = { version = "0.1", default-features = false }
dioxus = { version = "0.6", features = ["web"] }
```

And mount your chart inside a Dioxus component:

```rust
use dioxus::prelude::*;
use openalgo_charts::prelude::*;

#[component]
pub fn DioxusTradingChart(
    data: ReadOnlySignal<Vec<Bar>>,
    #[props(default = SeriesKind::Candlestick)] series_kind: SeriesKind,
    #[props(default = "width: 100%; height: 500px;".to_string())] style: String,
) -> Element {
    let mut controller = use_signal(|| ChartController::new());
    let element_id = use_hook(|| format!("oac-{}", (js_sys::Math::random() * 10000.0) as u32));

    // Mount and react to data updates
    use_effect(move || {
        let bars = data.read();
        let mut ctrl = controller.write();
        if !ctrl.is_mounted() {
            let _ = ctrl.mount_by_id(&element_id, None, series_kind, None, &bars);
        } else {
            let _ = ctrl.set_data(&bars);
        }
    });

    // Clean up chart and prevent memory leaks on unmount
    use_drop(move || {
        controller.write().destroy();
    });

    rsx! {
        div {
            id: "{element_id}",
            style: "{style}",
        }
    }
}
```

---

## Architecture Breakdown

| Module | Purpose |
|---|---|
| [`sys`](file:///crates/openalgo-charts/src/sys.rs) | Direct `#[wasm_bindgen]` bindings to `createChart`, `Chart`, `SeriesApi`, `TimeScale`, `PriceScale`, and widget bridge functions. |
| [`types`](file:///crates/openalgo-charts/src/types.rs) | Data models: `Bar`, `LineData`, `SeriesKind`, `ChartOptions`, `SeriesOptions`, `ChartTheme`. |
| [`chart`](file:///crates/openalgo-charts/src/chart.rs) | Safe `ChartHandle` & `SeriesHandle` wrappers with `serde-wasm-bindgen` serialization. |
| [`events`](file:///crates/openalgo-charts/src/events.rs) | `CrosshairMoveEvent`, `OrderClickEvent`, and RAII `Subscription` handles. |
| [`indicators`](file:///crates/openalgo-charts/src/indicators.rs) | 502 built-in indicator metadata, inputs, and live instance lifecycle methods. |
| [`drawing`](file:///crates/openalgo-charts/src/drawing.rs) | Drawing tool enums, undo/redo dispatch, and symbol storage isolation. |
| [`theme`](file:///crates/openalgo-charts/src/theme.rs) | Live CSS token sampling (`theme_from_skin()`) for dark/light theme switching. |
| [`leptos`](file:///crates/openalgo-charts/src/leptos/mod.rs) | `<OpenAlgoChart />` and `<ChartContainer />` components. |

---

## License

AGPL-3.0 - OpenAlgo Charts & Honba ecosystem.
