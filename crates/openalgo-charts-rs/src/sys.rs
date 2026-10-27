//! Raw `#[wasm_bindgen]` extern declarations for OpenAlgo Charts (TradingView Lightweight Charts fork)
//! and the terminal widget bridge.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // ------------------------------------------------------------------------
    // Core Engine Types
    // ------------------------------------------------------------------------

    /// The top-level Chart instance returned by `createChart`.
    #[wasm_bindgen(is_type_of = is_chart)]
    pub type Chart;

    /// A series handle returned by `chart.addSeries(...)`.
    #[wasm_bindgen(is_type_of = is_series_api)]
    pub type SeriesApi;

    /// TimeScale controller handle returned by `chart.timeScale()`.
    #[wasm_bindgen(is_type_of = is_time_scale)]
    pub type TimeScale;

    /// PriceScale controller handle returned by `series.priceScale()`.
    #[wasm_bindgen(is_type_of = is_price_scale)]
    pub type PriceScale;

    // ------------------------------------------------------------------------
    // Core createChart entrypoint
    // ------------------------------------------------------------------------

    #[wasm_bindgen(catch, js_name = createChart)]
    pub fn create_chart(
        container: &web_sys::HtmlElement,
        options: &JsValue,
    ) -> Result<Chart, JsValue>;

    #[wasm_bindgen(catch, js_namespace = window, js_name = createChart)]
    pub fn create_chart_window(
        container: &web_sys::HtmlElement,
        options: &JsValue,
    ) -> Result<Chart, JsValue>;

    // ------------------------------------------------------------------------
    // Chart methods
    // ------------------------------------------------------------------------

    #[wasm_bindgen(method, js_name = addSeries)]
    pub fn add_series(this: &Chart, series_type: &str, options: &JsValue) -> SeriesApi;

    #[wasm_bindgen(method, js_name = removeSeries)]
    pub fn remove_series(this: &Chart, series: &SeriesApi);

    #[wasm_bindgen(method, js_name = fitContent)]
    pub fn fit_content(this: &Chart);

    #[wasm_bindgen(method, js_name = timeScale)]
    pub fn time_scale(this: &Chart) -> TimeScale;

    #[wasm_bindgen(method, js_name = timeToCoordinate)]
    pub fn time_to_coordinate(this: &Chart, time: f64) -> f64;

    #[wasm_bindgen(method, js_name = coordinateToTime)]
    pub fn coordinate_to_time(this: &Chart, x: f64) -> f64;

    #[wasm_bindgen(method, js_name = subscribeClick)]
    pub fn subscribe_click(this: &Chart, cb: &js_sys::Function) -> js_sys::Function;

    #[wasm_bindgen(method, js_name = subscribeCrosshairMove)]
    pub fn subscribe_crosshair_move(this: &Chart, cb: &js_sys::Function) -> js_sys::Function;

    #[wasm_bindgen(method, js_name = subscribeDrag)]
    pub fn subscribe_drag(
        this: &Chart,
        on_drag: &js_sys::Function,
        on_drag_end: &JsValue,
    ) -> js_sys::Function;

    #[wasm_bindgen(method, js_name = on)]
    pub fn on(this: &Chart, event: &str, cb: &js_sys::Function) -> js_sys::Function;

    #[wasm_bindgen(method, js_name = off)]
    pub fn off(this: &Chart, event: &str, cb: &JsValue);

    #[wasm_bindgen(method, js_name = destroy)]
    pub fn destroy(this: &Chart);

    #[wasm_bindgen(method, js_name = applyOptions)]
    pub fn apply_options(this: &Chart, options: &JsValue);

    #[wasm_bindgen(method, js_name = setDataContext)]
    pub fn set_data_context(this: &Chart, context: &JsValue);

    #[wasm_bindgen(method, js_name = primarySeries)]
    pub fn primary_series(this: &Chart) -> Option<SeriesApi>;

    // ------------------------------------------------------------------------
    // SeriesApi methods
    // ------------------------------------------------------------------------

    #[wasm_bindgen(method, js_name = setData)]
    pub fn set_data(this: &SeriesApi, data: &JsValue);

    #[wasm_bindgen(method, js_name = update)]
    pub fn update(this: &SeriesApi, bar: &JsValue);

    #[wasm_bindgen(method, js_name = prependData)]
    pub fn prepend_data(this: &SeriesApi, data: &JsValue);

    #[wasm_bindgen(method, js_name = getData)]
    pub fn get_data(this: &SeriesApi) -> JsValue;

    #[wasm_bindgen(method, js_name = applyOptions)]
    pub fn series_apply_options(this: &SeriesApi, options: &JsValue);

    #[wasm_bindgen(method, js_name = priceScale)]
    pub fn price_scale(this: &SeriesApi) -> PriceScale;

    #[wasm_bindgen(method, js_name = remove)]
    pub fn remove(this: &SeriesApi);

    // ------------------------------------------------------------------------
    // TimeScale methods
    // ------------------------------------------------------------------------

    #[wasm_bindgen(method, js_name = fitContent)]
    pub fn time_scale_fit_content(this: &TimeScale);

    #[wasm_bindgen(method, js_name = applyOptions)]
    pub fn time_scale_apply_options(this: &TimeScale, options: &JsValue);

    #[wasm_bindgen(method, js_name = scrollToPosition)]
    pub fn scroll_to_position(this: &TimeScale, position: f64, animated: bool);

    #[wasm_bindgen(method, js_name = scrollToRealTime)]
    pub fn scroll_to_real_time(this: &TimeScale);

    #[wasm_bindgen(method, js_name = resetTimeScale)]
    pub fn reset_time_scale(this: &TimeScale);

    // ------------------------------------------------------------------------
    // PriceScale methods
    // ------------------------------------------------------------------------

    #[wasm_bindgen(method, js_name = applyOptions)]
    pub fn price_scale_apply_options(this: &PriceScale, options: &JsValue);

    #[wasm_bindgen(method, js_name = setOptions)]
    pub fn price_scale_set_options(this: &PriceScale, options: &JsValue);

    // ------------------------------------------------------------------------
    // Bridge Window namespace bindings (window.honbaCharts / window.openalgoCharts)
    // ------------------------------------------------------------------------

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = mount, catch)]
    pub fn bridge_mount_with_options(
        element: &JsValue,
        symbol: &str,
        timeframe: &str,
        chart_mode: &str,
        quantity: f64,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = mountStatic, catch)]
    pub fn bridge_mount_static(element: &JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = setSeries, catch)]
    pub fn bridge_set_series(
        id: f64,
        kind: &str,
        style: &str,
        bars: &str,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = updateLast, catch)]
    pub fn bridge_update_last(id: f64, bar: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = addVolumeOverlay, catch)]
    pub fn bridge_add_volume_overlay(id: f64) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = addBuySell, catch)]
    pub fn bridge_add_buy_sell(id: f64, qty: f64, symbol: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = setQuantity, catch)]
    pub fn bridge_set_quantity(id: f64, qty: f64) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = setSymbolContext, catch)]
    pub fn bridge_set_symbol_context(id: f64, symbol: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = setDrawingTool, catch)]
    pub fn bridge_set_drawing_tool(id: f64, tool: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = drawingTool, catch)]
    pub fn bridge_drawing_tool(id: f64) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = undoDrawing, catch)]
    pub fn bridge_undo_drawing(id: f64) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = redoDrawing, catch)]
    pub fn bridge_redo_drawing(id: f64) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = clearDrawings, catch)]
    pub fn bridge_clear_drawings(id: f64) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = destroy, catch)]
    pub fn bridge_destroy(id: f64) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = indicatorCatalog, catch)]
    pub fn bridge_indicator_catalog() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = chartIndicators, catch)]
    pub fn bridge_chart_indicators(id: f64) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = addIndicator, catch)]
    pub fn bridge_add_indicator(
        id: f64,
        indicator_id: &str,
        settings_json: &str,
        instance_id: &str,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = removeIndicator, catch)]
    pub fn bridge_remove_indicator(id: f64, instance_id: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = registerCustomIndicatorPicker, catch)]
    pub fn bridge_register_custom_indicator_picker(mount_fn: &JsValue) -> Result<JsValue, JsValue>;
}

fn is_chart(value: &JsValue) -> bool {
    value.is_object()
        && js_sys::Reflect::has(value, &JsValue::from_str("addSeries")).unwrap_or(false)
}

fn is_series_api(value: &JsValue) -> bool {
    value.is_object() && js_sys::Reflect::has(value, &JsValue::from_str("setData")).unwrap_or(false)
}

fn is_time_scale(value: &JsValue) -> bool {
    value.is_object()
        && js_sys::Reflect::has(value, &JsValue::from_str("fitContent")).unwrap_or(false)
}

fn is_price_scale(value: &JsValue) -> bool {
    value.is_object()
        && js_sys::Reflect::has(value, &JsValue::from_str("setOptions")).unwrap_or(false)
}

impl Clone for Chart {
    fn clone(&self) -> Self {
        use wasm_bindgen::JsCast;
        self.unchecked_ref::<JsValue>().clone().unchecked_into()
    }
}

impl Clone for SeriesApi {
    fn clone(&self) -> Self {
        use wasm_bindgen::JsCast;
        self.unchecked_ref::<JsValue>().clone().unchecked_into()
    }
}

impl Clone for TimeScale {
    fn clone(&self) -> Self {
        use wasm_bindgen::JsCast;
        self.unchecked_ref::<JsValue>().clone().unchecked_into()
    }
}

impl Clone for PriceScale {
    fn clone(&self) -> Self {
        use wasm_bindgen::JsCast;
        self.unchecked_ref::<JsValue>().clone().unchecked_into()
    }
}

unsafe impl Send for Chart {}
unsafe impl Sync for Chart {}
unsafe impl Send for SeriesApi {}
unsafe impl Sync for SeriesApi {}
unsafe impl Send for TimeScale {}
unsafe impl Sync for TimeScale {}
unsafe impl Send for PriceScale {}
unsafe impl Sync for PriceScale {}
