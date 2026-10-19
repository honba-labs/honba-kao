//! Workbench store — the Leptos translation of the labs app's
//! `core/workbench/store.ts` (Zustand). The layout grid, chart panes, the
//! order ticket draft and the local trading book are one reactive struct
//! provided on the Workbench page.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use leptos::prelude::*;

use crate::components::charts::openalgo_chart::IndicatorDef;
use crate::data::demo_data;
use crate::data::symbol_search;
use crate::types::Bar;

// ---------------------------------------------------------------------------
// Layout + chart pane configuration
// ---------------------------------------------------------------------------

/// Workspace grid preset (`single` / `cols2` / `rows2` / `oneTwo` / `grid4`).
/// Legacy `1x1…2x2` ids from the labs app are not carried over — the store
/// seeds (and persists nothing) so canonical ids are enough.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum WorkbenchLayout {
    Single,
    #[default]
    Cols2,
    Rows2,
    OneTwo,
    Grid4,
}

impl WorkbenchLayout {
    pub const fn as_id(self) -> &'static str {
        match self {
            WorkbenchLayout::Single => "single",
            WorkbenchLayout::Cols2 => "cols2",
            WorkbenchLayout::Rows2 => "rows2",
            WorkbenchLayout::OneTwo => "oneTwo",
            WorkbenchLayout::Grid4 => "grid4",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id {
            "single" => WorkbenchLayout::Single,
            "rows2" => WorkbenchLayout::Rows2,
            "oneTwo" => WorkbenchLayout::OneTwo,
            "grid4" => WorkbenchLayout::Grid4,
            _ => WorkbenchLayout::Cols2,
        }
    }
}

/// Chart interval, grouped the way the labs toolbar groups them
/// (Minutes / Hours / Days / Weeks / Months — `1m-45m`, `1h-4h`,
/// `1d 3d 5d`, `1w 3w 4w`, `1M 3M 6M`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Timeframe {
    M1,
    M3,
    M5,
    M15,
    M30,
    M45,
    H1,
    H2,
    H3,
    H4,
    #[default]
    D1,
    D3,
    D5,
    W1,
    W3,
    W4,
    /// Month `1` — label `1M` (upper M keeps it unambiguous against `1m`).
    #[allow(non_camel_case_types)]
    Mo1,
    #[allow(non_camel_case_types)]
    Mo3,
    #[allow(non_camel_case_types)]
    Mo6,
}

impl Timeframe {
    /// Display order for the pane dropdown: one flat list per group.
    pub const ALL: [Timeframe; 19] = [
        Timeframe::M1,
        Timeframe::M3,
        Timeframe::M5,
        Timeframe::M15,
        Timeframe::M30,
        Timeframe::M45,
        Timeframe::H1,
        Timeframe::H2,
        Timeframe::H3,
        Timeframe::H4,
        Timeframe::D1,
        Timeframe::D3,
        Timeframe::D5,
        Timeframe::W1,
        Timeframe::W3,
        Timeframe::W4,
        Timeframe::Mo1,
        Timeframe::Mo3,
        Timeframe::Mo6,
    ];

    /// The grouped presentation (`INTERVAL_GROUPS` in the labs app, extended
    /// to the day/week/month denominations the user asked for).
    pub const GROUPS: [(&str, &[Timeframe]); 5] = [
        (
            "Minutes",
            &[
                Timeframe::M1,
                Timeframe::M3,
                Timeframe::M5,
                Timeframe::M15,
                Timeframe::M30,
                Timeframe::M45,
            ],
        ),
        (
            "Hours",
            &[Timeframe::H1, Timeframe::H2, Timeframe::H3, Timeframe::H4],
        ),
        ("Days", &[Timeframe::D1, Timeframe::D3, Timeframe::D5]),
        ("Weeks", &[Timeframe::W1, Timeframe::W3, Timeframe::W4]),
        ("Months", &[Timeframe::Mo1, Timeframe::Mo3, Timeframe::Mo6]),
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Timeframe::M1 => "1m",
            Timeframe::M3 => "3m",
            Timeframe::M5 => "5m",
            Timeframe::M15 => "15m",
            Timeframe::M30 => "30m",
            Timeframe::M45 => "45m",
            Timeframe::H1 => "1h",
            Timeframe::H2 => "2h",
            Timeframe::H3 => "3h",
            Timeframe::H4 => "4h",
            Timeframe::D1 => "1D",
            Timeframe::D3 => "3D",
            Timeframe::D5 => "5D",
            Timeframe::W1 => "1W",
            Timeframe::W3 => "3W",
            Timeframe::W4 => "4W",
            Timeframe::Mo1 => "1M",
            Timeframe::Mo3 => "3M",
            Timeframe::Mo6 => "6M",
        }
    }

    /// Long form for the dropdown row title (`15m` → `"15 Minute"`).
    pub const fn full_label(self) -> &'static str {
        match self {
            Timeframe::M1 => "1 Minute",
            Timeframe::M3 => "3 Minute",
            Timeframe::M5 => "5 Minute",
            Timeframe::M15 => "15 Minute",
            Timeframe::M30 => "30 Minute",
            Timeframe::M45 => "45 Minute",
            Timeframe::H1 => "1 Hour",
            Timeframe::H2 => "2 Hour",
            Timeframe::H3 => "3 Hour",
            Timeframe::H4 => "4 Hour",
            Timeframe::D1 => "1 Day",
            Timeframe::D3 => "3 Day",
            Timeframe::D5 => "5 Day",
            Timeframe::W1 => "1 Week",
            Timeframe::W3 => "3 Week",
            Timeframe::W4 => "4 Week",
            Timeframe::Mo1 => "1 Month",
            Timeframe::Mo3 => "3 Month",
            Timeframe::Mo6 => "6 Month",
        }
    }

    pub const fn interval_seconds(self) -> i64 {
        match self {
            Timeframe::M1 => 60,
            Timeframe::M3 => 180,
            Timeframe::M5 => 300,
            Timeframe::M15 => 900,
            Timeframe::M30 => 1_800,
            Timeframe::M45 => 2_700,
            Timeframe::H1 => 3_600,
            Timeframe::H2 => 7_200,
            Timeframe::H3 => 10_800,
            Timeframe::H4 => 14_400,
            Timeframe::D1 => 86_400,
            Timeframe::D3 => 259_200,
            Timeframe::D5 => 432_000,
            Timeframe::W1 => 604_800,
            Timeframe::W3 => 1_814_400,
            Timeframe::W4 => 2_419_200,
            Timeframe::Mo1 => 2_592_000,
            Timeframe::Mo3 => 7_776_000,
            Timeframe::Mo6 => 15_552_000,
        }
    }

    pub fn from_id(id: &str) -> Self {
        // Case matters: `1m` is minutes, `1M` is months.
        match id {
            "1m" => Timeframe::M1,
            "3m" => Timeframe::M3,
            "5m" => Timeframe::M5,
            "15m" => Timeframe::M15,
            "30m" => Timeframe::M30,
            "45m" => Timeframe::M45,
            "1h" => Timeframe::H1,
            "2h" => Timeframe::H2,
            "3h" => Timeframe::H3,
            "4h" => Timeframe::H4,
            "1d" | "1D" => Timeframe::D1,
            "3d" | "3D" => Timeframe::D3,
            "5d" | "5D" => Timeframe::D5,
            "1w" | "1W" => Timeframe::W1,
            "3w" | "3W" => Timeframe::W3,
            "4w" | "4W" => Timeframe::W4,
            "1M" => Timeframe::Mo1,
            "3M" => Timeframe::Mo3,
            "6M" => Timeframe::Mo6,
            _ => Timeframe::D1,
        }
    }
}

/// Series renderer for a pane — the openalgo-charts `addSeries` kinds the
/// engine actually ships (candles, bars, lines, areas, movement).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ChartMode {
    #[default]
    Candles,
    HollowCandle,
    VolumeCandle,
    Bar,
    HighLow,
    Line,
    LineMarkers,
    Step,
    Area,
    HlcArea,
    Baseline,
    Column,
    Histogram,
}

impl ChartMode {
    pub const ALL: [ChartMode; 13] = [
        ChartMode::Candles,
        ChartMode::HollowCandle,
        ChartMode::VolumeCandle,
        ChartMode::Bar,
        ChartMode::HighLow,
        ChartMode::Line,
        ChartMode::LineMarkers,
        ChartMode::Step,
        ChartMode::Area,
        ChartMode::HlcArea,
        ChartMode::Baseline,
        ChartMode::Column,
        ChartMode::Histogram,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            ChartMode::Candles => "Candles",
            ChartMode::HollowCandle => "Hollow Candles",
            ChartMode::VolumeCandle => "Volume Candles",
            ChartMode::Bar => "Bars (OHLC)",
            ChartMode::HighLow => "High-Low",
            ChartMode::Line => "Line",
            ChartMode::LineMarkers => "Line + Markers",
            ChartMode::Step => "Step",
            ChartMode::Area => "Area",
            ChartMode::HlcArea => "HLC Area",
            ChartMode::Baseline => "Baseline",
            ChartMode::Column => "Column",
            ChartMode::Histogram => "Histogram",
        }
    }

    /// The `addSeries` kind string the engine registers.
    pub const fn as_id(self) -> &'static str {
        match self {
            ChartMode::Candles => "candlestick",
            ChartMode::HollowCandle => "hollow-candle",
            ChartMode::VolumeCandle => "volume-candle",
            ChartMode::Bar => "bar",
            ChartMode::HighLow => "high-low",
            ChartMode::Line => "line",
            ChartMode::LineMarkers => "line-markers",
            ChartMode::Step => "step",
            ChartMode::Area => "area",
            ChartMode::HlcArea => "hlc-area",
            ChartMode::Baseline => "baseline",
            ChartMode::Column => "column",
            ChartMode::Histogram => "histogram",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id {
            "hollow-candle" => ChartMode::HollowCandle,
            "volume-candle" => ChartMode::VolumeCandle,
            "bar" => ChartMode::Bar,
            "high-low" => ChartMode::HighLow,
            "line" => ChartMode::Line,
            "line-markers" => ChartMode::LineMarkers,
            "step" => ChartMode::Step,
            "area" => ChartMode::Area,
            "hlc-area" => ChartMode::HlcArea,
            "baseline" => ChartMode::Baseline,
            "column" => ChartMode::Column,
            "histogram" => ChartMode::Histogram,
            _ => ChartMode::Candles,
        }
    }

    /// Quick chart-mode pills; everything else lives in the Type menu to keep
    /// the pane header on one line (real-estate).
    pub const fn is_pane_pill(self) -> bool {
        matches!(self, ChartMode::Candles | ChartMode::Bar | ChartMode::Area)
    }
}

/// One workspace pane.
#[derive(Clone, Debug, PartialEq)]
pub struct ChartPaneConfig {
    pub id: &'static str,
    pub symbol: String,
    pub timeframe: Timeframe,
    pub mode: ChartMode,
}

impl Default for ChartPaneConfig {
    fn default() -> Self {
        ChartPaneConfig {
            id: "pane-0",
            symbol: default_symbol().to_string(),
            timeframe: Timeframe::D1,
            mode: ChartMode::Candles,
        }
    }
}

fn default_symbol() -> &'static str {
    "RELIANCE"
}

/// A single editable field on the order ticket (the `setTicketField` union).
#[derive(Clone, Debug, PartialEq)]
pub enum TicketField {
    Symbol(String),
    Side(OrderSide),
    OrderType(OrderType),
    Product(Product),
    Quantity(i64),
    Price(f64),
    TriggerPrice(f64),
    TimeInForce(TimeInForce),
}

/// One indicator instance applied to a pane (a slot in the store so it
/// survives the chart remounts that a symbol/timeframe switch performs).
#[derive(Clone, Debug, PartialEq)]
pub struct AppliedIndicator {
    pub instance_id: String,
    pub indicator_id: String,
    pub name: String,
    /// `onchart` overlays share the price scale; `pane` render below.
    pub placement: String,
}

/// A single editable field on one chart pane (`updatePane`).
#[derive(Clone, Debug, PartialEq)]
pub enum PaneField {
    Symbol(String),
    Timeframe(Timeframe),
    Mode(ChartMode),
}

// ---------------------------------------------------------------------------
// Order domain — the local, optimistic trading book
// ---------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum OrderSide {
    #[default]
    Buy,
    Sell,
}

impl OrderSide {
    pub const fn is_buy(self) -> bool {
        matches!(self, OrderSide::Buy)
    }

    pub const fn label(self) -> &'static str {
        match self {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        }
    }

    pub const fn as_id(self) -> &'static str {
        match self {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell",
        }
    }

    /// Position side wording used in the drawer (`LONG` / `SHORT`).
    pub const fn position_label(self) -> &'static str {
        match self {
            OrderSide::Buy => "LONG",
            OrderSide::Sell => "SHORT",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum OrderType {
    #[default]
    Limit,
    Market,
    StopMarket,
    StopLimit,
}

impl OrderType {
    /// `market | limit | stop_market | stop_limit` for the wire.
    pub const fn as_id(self) -> &'static str {
        match self {
            OrderType::Market => "market",
            OrderType::Limit => "limit",
            OrderType::StopMarket => "stop_market",
            OrderType::StopLimit => "stop_limit",
        }
    }

    /// Short pill labels.
    pub const fn short(self) -> &'static str {
        match self {
            OrderType::Market => "MKT",
            OrderType::Limit => "LMT",
            OrderType::StopMarket => "SL-M",
            OrderType::StopLimit => "SL",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Product {
    #[default]
    Intraday,
    Delivery,
}

impl Product {
    pub const fn as_id(self) -> &'static str {
        match self {
            Product::Intraday => "intraday",
            Product::Delivery => "delivery",
        }
    }

    /// Ticket pill labels.
    pub const fn label(self) -> &'static str {
        match self {
            Product::Intraday => "Intraday (MIS 5x)",
            Product::Delivery => "Delivery (CNC)",
        }
    }

    /// Margin fraction: MIS needs 20%, CNC needs full notional.
    pub const fn margin_factor(self) -> f64 {
        match self {
            Product::Intraday => 0.2,
            Product::Delivery => 1.0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TimeInForce {
    #[default]
    Day,
    Ioc,
}

impl TimeInForce {
    pub const fn as_id(self) -> &'static str {
        match self {
            TimeInForce::Day => "day",
            TimeInForce::Ioc => "ioc",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            TimeInForce::Day => "Day",
            TimeInForce::Ioc => "IOC",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TradingState {
    #[default]
    Active,
    Reducing,
    Halted,
}

impl TradingState {
    pub const fn as_id(self) -> &'static str {
        match self {
            TradingState::Active => "active",
            TradingState::Reducing => "reducing",
            TradingState::Halted => "halted",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum DrawerTab {
    #[default]
    Holdings,
    Positions,
    Orders,
    Trades,
    Analytics,
    Depth,
}

impl DrawerTab {
    pub const fn label(self) -> &'static str {
        match self {
            DrawerTab::Holdings => "Holdings",
            DrawerTab::Positions => "Positions",
            DrawerTab::Orders => "Open Orders",
            DrawerTab::Trades => "Trade Book",
            DrawerTab::Analytics => "Analytics",
            DrawerTab::Depth => "Market Depth",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct OrderTicketDraft {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub product: Product,
    pub quantity: i64,
    pub price: f64,
    pub trigger_price: f64,
    pub tif: TimeInForce,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WbPosition {
    pub symbol: String,
    /// `Buy` = long, `Sell` = short.
    pub side: OrderSide,
    pub quantity: i64,
    pub avg_price: f64,
    pub ltp: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WbOrder {
    pub order_id: String,
    pub symbol: String,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub quantity: i64,
    pub price: f64,
    pub status: String,
    pub product: Product,
    pub tif: TimeInForce,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WbTrade {
    pub order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub price: f64,
    pub quantity: i64,
    /// Brokerage/fees charged for the fill (0.03% of notional, like the labs
    /// mock book).
    pub costs: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    Info(String),
    Error(String),
}

// ---------------------------------------------------------------------------
// Sequencing + pricing helpers
// ---------------------------------------------------------------------------

static ORDER_SEQ: AtomicU64 = AtomicU64::new(10_928);

/// Sequential demo book ids (`ORD-10929`, …) standing in for backend ids.
fn next_order_id() -> String {
    format!("ORD-{}", ORDER_SEQ.fetch_add(1, Ordering::Relaxed) + 1)
}

/// LTP used to seed the ticket and depth: the bundled universe (the search
/// pool carries `?tab`-free real prices), then demo quotes, then ₹1000.
pub fn instrument_price(symbol: &str) -> f64 {
    let upper = symbol.to_uppercase();
    if let Some(item) = symbol_search::metadata(&upper) {
        if item.price > 0.0 {
            return item.price;
        }
    }
    demo_data::demo_quotes()
        .iter()
        .find(|quote| quote.symbol.eq_ignore_ascii_case(symbol))
        .map_or(1000.0, |quote| quote.last)
}

/// Deterministic OHLCV bars for one pane, ending (within a tick) on the
/// instrument's demo LTP so charts, tickets and depth agree.
pub fn pane_bars(symbol: &str, timeframe: Timeframe) -> Vec<Bar> {
    let price = instrument_price(symbol);
    let mut bars = demo_data::demo_ohlc_bars(price, 360, timeframe.interval_seconds());

    if let Some(last_close) = bars.last().map(|bar| bar.close)
        && last_close.abs() > f64::EPSILON
    {
        let scale = price / last_close;
        for bar in &mut bars {
            bar.open = (bar.open * scale).round_to_paise();
            bar.high = (bar.high * scale).round_to_paise();
            bar.low = (bar.low * scale).round_to_paise();
            bar.close = (bar.close * scale).round_to_paise();
        }
    }

    bars
}

trait RoundPaise {
    fn round_to_paise(self) -> f64;
}

impl RoundPaise for f64 {
    fn round_to_paise(self) -> f64 {
        (self * 100.0).round() / 100.0
    }
}

// ---------------------------------------------------------------------------
// localStorage + URL helpers (labs parity: layout preset, picker lists, ?symbol)
// ---------------------------------------------------------------------------

fn storage() -> Option<web_sys::Storage> {
    web_sys::window().and_then(|window| window.local_storage().ok().flatten())
}

const LAYOUT_KEY: &str = "honba_layout_preset_v2";

/// The persisted workspace grid; absent/unknown ids fall back to `Cols2`.
fn persisted_layout() -> WorkbenchLayout {
    storage()
        .and_then(|storage| storage.get_item(LAYOUT_KEY).ok().flatten())
        .map(|raw| WorkbenchLayout::from_id(&raw))
        .unwrap_or(WorkbenchLayout::Cols2)
}

fn save_layout(layout: WorkbenchLayout) {
    if let Some(storage) = storage() {
        let _ = storage.set_item(LAYOUT_KEY, layout.as_id());
    }
}

/// Read a `["a","b"]` string list out of localStorage.
fn load_stored_strings(key: &str) -> Vec<String> {
    storage()
        .and_then(|storage| storage.get_item(key).ok().flatten())
        .and_then(|raw| serde_json::from_str::<Vec<String>>(&raw).ok())
        .unwrap_or_default()
}

fn save_stored_strings(key: &str, values: &[String]) {
    if let Some(storage) = storage() {
        let json = serde_json::to_string(values).unwrap_or_else(|_| "[]".to_string());
        let _ = storage.set_item(key, &json);
    }
}

/// `?symbol=` / `?symbols=`: uppercase, trimmed, de-duplicated, max 4.
fn requested_symbols() -> Vec<String> {
    web_sys::window()
        .and_then(|window| window.location().search().ok())
        .and_then(|search| {
            let query = search.strip_prefix('?')?;
            let mut symbols: Vec<String> = Vec::new();
            for pair in query.split('&') {
                let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
                if key == "symbol" || key == "symbols" {
                    for token in value.split(',') {
                        let clean = token.trim().to_uppercase();
                        if !clean.is_empty() && !symbols.contains(&clean) && symbols.len() < 4 {
                            symbols.push(clean);
                        }
                    }
                }
            }
            Some(symbols)
        })
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

/// The workbench reactive state. Every field is a signal, so components can
/// bind to exactly what they show (`get`/`set` interplay ≈ Zustand actions).
#[derive(Clone, Copy)]
pub struct WorkbenchStore {
    pub layout: RwSignal<WorkbenchLayout>,
    pub active_pane: RwSignal<usize>,
    pub panes: RwSignal<Vec<ChartPaneConfig>>,
    /// Indicator instances per pane id (`"pane-0"`, …).
    pub applied_indicators: RwSignal<Vec<(String, Vec<AppliedIndicator>)>>,
    /// The symbol-search modal (`search_target` decides which pane receives
    /// the picked symbol; `None` targets the active pane).
    pub search_open: RwSignal<bool>,
    pub search_target: RwSignal<Option<usize>>,
    /// The indicator-picker dialog and the pane it serves.
    pub picker_open: RwSignal<bool>,
    pub picker_target: RwSignal<usize>,
    pub picker_catalog: RwSignal<Option<Vec<IndicatorDef>>>,
    pub fav_indicators: RwSignal<Vec<String>>,
    pub recent_indicators: RwSignal<Vec<String>>,
    pub ticket: RwSignal<OrderTicketDraft>,
    pub positions: RwSignal<Vec<WbPosition>>,
    pub orders: RwSignal<Vec<WbOrder>>,
    pub trades: RwSignal<Vec<WbTrade>>,
    pub trading_state: RwSignal<TradingState>,
    pub drawer_tab: RwSignal<DrawerTab>,
    pub drawer_open: RwSignal<bool>,
    pub ticket_open: RwSignal<bool>,
    pub modal_open: RwSignal<bool>,
    pub status: RwSignal<Option<Status>>,
}

impl WorkbenchStore {
    pub fn new() -> Self {
        let ticket = OrderTicketDraft {
            symbol: default_symbol().to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            product: Product::Intraday,
            quantity: 25,
            price: instrument_price(default_symbol()),
            trigger_price: 0.0,
            tif: TimeInForce::Day,
        };

        let positions = vec![
            // Seeded like the labs mock book: RELIANCE + INFY longs.
            WbPosition {
                symbol: default_symbol().to_string(),
                side: OrderSide::Buy,
                quantity: 50,
                avg_price: 2920.5,
                ltp: 2954.2,
                realized_pnl: 1450.0,
                unrealized_pnl: (2954.2 - 2920.5) * 50.0,
            },
            WbPosition {
                symbol: "INFY".to_string(),
                side: OrderSide::Buy,
                quantity: 100,
                avg_price: 1880.0,
                ltp: 1895.4,
                realized_pnl: -320.0,
                unrealized_pnl: (1895.4 - 1880.0) * 100.0,
            },
        ];

        let orders = vec![
            WbOrder {
                order_id: "ORD-10927".to_string(),
                symbol: "HDFCBANK".to_string(),
                order_type: OrderType::Limit,
                side: OrderSide::Sell,
                quantity: 40,
                price: 1720.0,
                status: "accepted".to_string(),
                product: Product::Intraday,
                tif: TimeInForce::Day,
            },
            WbOrder {
                order_id: "ORD-10928".to_string(),
                symbol: "TCS".to_string(),
                order_type: OrderType::Limit,
                side: OrderSide::Buy,
                quantity: 20,
                price: 3950.0,
                status: "accepted".to_string(),
                product: Product::Intraday,
                tif: TimeInForce::Day,
            },
        ];

        Self {
            layout: RwSignal::new(persisted_layout()),
            active_pane: RwSignal::new(0),
            panes: RwSignal::new(Self::default_panes()),
            applied_indicators: RwSignal::new(Vec::new()),
            search_open: RwSignal::new(false),
            search_target: RwSignal::new(None),
            picker_open: RwSignal::new(false),
            picker_target: RwSignal::new(0),
            picker_catalog: RwSignal::new(None),
            fav_indicators: RwSignal::new(load_stored_strings("honba-workbench-fav-indicators")),
            recent_indicators: RwSignal::new(load_stored_strings(
                "honba-workbench-recent-indicators",
            )),
            ticket: RwSignal::new(ticket),
            positions: RwSignal::new(positions),
            orders: RwSignal::new(orders),
            trades: RwSignal::new(Vec::new()),
            trading_state: RwSignal::new(TradingState::Active),
            drawer_tab: RwSignal::new(DrawerTab::Holdings),
            drawer_open: RwSignal::new(false),
            ticket_open: RwSignal::new(false),
            modal_open: RwSignal::new(false),
            status: RwSignal::new(None),
        }
    }

    fn default_panes() -> Vec<ChartPaneConfig> {
        let mut symbols = ["RELIANCE", "INFY", "HDFCBANK", "TCS"];

        // `?symbol=RELIANCE` / `?symbols=RELIANCE,TCS,ITC` (from the screener's
        // "Open in Workbench" buttons) seed the panes in order.
        let wanted = requested_symbols();
        for (index, slot) in symbols.iter_mut().enumerate() {
            if let Some(symbol) = wanted.get(index) {
                *slot = symbol;
            }
        }

        symbols
            .into_iter()
            .enumerate()
            .map(|(index, symbol)| ChartPaneConfig {
                id: match index {
                    0 => "pane-0",
                    1 => "pane-1",
                    2 => "pane-2",
                    _ => "pane-3",
                },
                symbol: symbol.to_string(),
                timeframe: Timeframe::D1,
                mode: ChartMode::Candles,
            })
            .collect()
    }

    /// Provide on the page scope (`provide_context`).
    pub fn provide(self) {
        provide_context(self);
    }

    /// Fetch from context, defined by [`crate::pages::workbench_page`]'s mount.
    pub fn take() -> Self {
        use_context().expect("WorkbenchStore: provide it above the component")
    }

    // -- Layout + panes ------------------------------------------------------

    pub fn set_layout(&self, layout: WorkbenchLayout) {
        self.layout.set(layout);
        save_layout(layout);
    }

    pub fn set_active_pane(&self, index: usize) {
        if let Some(pane) = self.panes.get_untracked().get(index) {
            self.active_pane.set(index);
            self.sync_ticket_with_symbol(&pane.symbol);
        }
    }

    pub fn update_pane(&self, index: usize, field: PaneField) {
        // The pane edit and the ticket follow-up both consume `field`, so the
        // changed symbol is captured first.
        let changed_symbol = match &field {
            PaneField::Symbol(symbol) => Some(symbol.to_uppercase()),
            _ => None,
        };

        self.panes.update(|panes| {
            if let Some(pane) = panes.get_mut(index) {
                match field {
                    PaneField::Symbol(symbol) => pane.symbol = symbol.to_uppercase(),
                    PaneField::Timeframe(timeframe) => pane.timeframe = timeframe,
                    PaneField::Mode(mode) => pane.mode = mode,
                }
            }
        });

        if let Some(symbol) = changed_symbol
            && self.active_pane.get_untracked() == index
        {
            self.sync_ticket_with_symbol(&symbol);
        }
    }

    // -- Symbol search modal --------------------------------------------------

    /// Open the search modal; `target` picks the pane that receives the
    /// symbol (`None` = the active pane, like the labs toolbar shortcut).
    pub fn open_symbol_search(&self, target: Option<usize>) {
        if let Some(index) = target {
            self.search_target.set(Some(index));
        } else {
            self.search_target
                .set(Some(self.active_pane.get_untracked()));
        }
        self.search_open.set(true);
    }

    pub fn close_symbol_search(&self) {
        self.search_open.set(false);
    }

    /// Commit a picked symbol to the pane the modal was opened from.
    pub fn pick_symbol(&self, symbol: &str) {
        let index = self
            .search_target
            .get_untracked()
            .unwrap_or_else(|| self.active_pane.get_untracked());
        self.update_pane(index, PaneField::Symbol(symbol.to_string()));
        self.close_symbol_search();
    }

    // -- Indicator picker -----------------------------------------------------

    pub fn open_indicator_picker(&self, target: usize) {
        self.picker_target.set(target);
        self.picker_open.set(true);
        self.ensure_picker_catalog();
    }

    pub fn close_indicator_picker(&self) {
        self.picker_open.set(false);
    }

    /// Fetch the engine registry once per session (the bridge JSON parses
    /// into `IndicatorDef`s; failures retry briefly like the chart mount).
    fn ensure_picker_catalog(&self) {
        if self.picker_catalog.get_untracked().is_some() {
            return;
        }
        let (catalog_signal, retry_attempt) = (self.picker_catalog, RwSignal::new(0u32));

        fn try_fetch() -> Option<Vec<IndicatorDef>> {
            crate::components::charts::openalgo_chart::indicator_catalog().ok()
        }

        if let Some(defs) = try_fetch() {
            catalog_signal.set(Some(defs));
            return;
        }

        Effect::new(move |_| {
            let _ = retry_attempt.get();
            if catalog_signal.get_untracked().is_some() {
                return;
            }
            if retry_attempt.get_untracked() >= 40 {
                return;
            }
            match try_fetch() {
                Some(defs) => catalog_signal.set(Some(defs)),
                None => set_timeout(
                    move || retry_attempt.update(|count| *count += 1),
                    Duration::from_millis(125),
                ),
            }
        });
    }

    /// Indicators applied to one pane (read shape for the pane header).
    pub fn pane_indicators(&self, pane_id: String) -> Memo<Vec<AppliedIndicator>> {
        let indicators = self.applied_indicators;
        Memo::new(move |_| {
            indicators
                .get()
                .iter()
                .find(|(id, _)| *id == pane_id)
                .map_or_else(Vec::new, |(_, list)| list.clone())
        })
    }

    /// Apply (add) an indicator on one pane and bump the recents list.
    pub fn add_pane_indicator(&self, pane_id: &str, applied: AppliedIndicator) {
        self.recent_indicators.update(|recents| {
            recents.retain(|id| *id != applied.indicator_id);
            recents.insert(0, applied.indicator_id.clone());
            recents.truncate(15);
        });
        save_stored_strings(
            "honba-workbench-recent-indicators",
            &self.recent_indicators.get_untracked(),
        );

        self.applied_indicators.update(|per_pane| {
            match per_pane.iter_mut().find(|(id, _)| id == pane_id) {
                Some((_, list)) => list.push(applied),
                None => per_pane.push((pane_id.to_string(), vec![applied])),
            }
        });
    }

    pub fn remove_pane_indicator(&self, pane_id: &str, instance_id: &str) {
        self.applied_indicators.update(|per_pane| {
            if let Some((_, list)) = per_pane.iter_mut().find(|(id, _)| id == pane_id) {
                list.retain(|applied| applied.instance_id != instance_id);
            }
        });
    }

    pub fn toggle_fav_indicator(&self, indicator_id: &str) {
        self.fav_indicators.update(|favs| {
            if favs.iter().any(|id| id == indicator_id) {
                favs.retain(|id| id != indicator_id);
            } else {
                favs.insert(0, indicator_id.to_string());
            }
        });
        save_stored_strings(
            "honba-workbench-fav-indicators",
            &self.fav_indicators.get_untracked(),
        );
    }

    // -- Order ticket --------------------------------------------------------

    pub fn set_ticket_field(&self, field: TicketField) {
        self.ticket.update(|ticket| match field {
            TicketField::Symbol(symbol) => ticket.symbol = symbol.to_uppercase(),
            TicketField::Side(side) => ticket.side = side,
            TicketField::OrderType(order_type) => ticket.order_type = order_type,
            TicketField::Product(product) => ticket.product = product,
            TicketField::Quantity(quantity) => ticket.quantity = quantity.max(1),
            TicketField::Price(price) => ticket.price = price.max(0.0),
            TicketField::TriggerPrice(trigger_price) => {
                ticket.trigger_price = trigger_price.max(0.0)
            }
            TicketField::TimeInForce(tif) => ticket.tif = tif,
        });
    }

    /// Quantity quick-increment from the stepper chips.
    pub fn shift_quantity(&self, delta: i64) {
        self.ticket.update(|ticket| {
            ticket.quantity = (ticket.quantity + delta).max(1);
        });
    }

    /// Point the ticket at a new symbol at its LTP (pane selection).
    pub fn sync_ticket_with_symbol(&self, symbol: &str) {
        let upper = symbol.to_uppercase();
        self.ticket.update(|ticket| {
            ticket.symbol = upper.clone();
            ticket.price = instrument_price(&upper);
        });
    }

    pub fn toggle_order_ticket(&self) {
        self.ticket_open.update(|open| *open = !*open);
    }

    pub fn open_order_ticket(&self, side: Option<OrderSide>, symbol: Option<&str>) {
        self.ticket_open.set(true);
        self.ticket.update(|ticket| {
            if let Some(side) = side {
                ticket.side = side;
            }
            if let Some(symbol) = symbol {
                let upper = symbol.to_uppercase();
                ticket.symbol = upper.clone();
                ticket.price = instrument_price(&upper);
            }
        });
    }

    pub fn close_order_ticket(&self) {
        self.ticket_open.set(false);
    }

    pub fn open_order_modal(
        &self,
        symbol: Option<&str>,
        side: Option<OrderSide>,
        price: Option<f64>,
        quantity: Option<i64>,
    ) {
        self.modal_open.set(true);
        self.ticket.update(|ticket| {
            let upper = symbol.map(|symbol| symbol.to_uppercase());
            let symbol = upper.as_ref().map_or(ticket.symbol.clone(), String::clone);
            ticket.symbol = symbol.clone();
            ticket.price = price.unwrap_or_else(|| instrument_price(&symbol));
            if let Some(side) = side {
                ticket.side = side;
            }
            if let Some(quantity) = quantity {
                ticket.quantity = quantity.max(1);
            }
        });
    }

    pub fn close_order_modal(&self) {
        self.modal_open.set(false);
    }

    // -- The local (optimistic) trading book ---------------------------------

    /// Submit the ticket: market orders fill into a trade + position; limit /
    /// stop orders land in the open-orders list as `accepted`.
    pub fn place_order(&self) {
        if self.trading_state.get_untracked() == TradingState::Halted {
            self.status.set(Some(Status::Error(
                "Cannot place order: Trading state is HALTED".to_string(),
            )));
            return;
        }

        let ticket = self.ticket.get_untracked();
        let order_id = next_order_id();
        let price = if ticket.order_type == OrderType::Market {
            instrument_price(&ticket.symbol)
        } else {
            ticket.price
        };

        if ticket.order_type == OrderType::Market {
            let trade = WbTrade {
                order_id: order_id.clone(),
                symbol: ticket.symbol.clone(),
                side: ticket.side,
                price,
                quantity: ticket.quantity,
                costs: (price * ticket.quantity as f64 * 0.0003 * 100.0).round() / 100.0,
            };
            self.trades.update(|trades| trades.insert(0, trade));
            self.apply_fill(&ticket.symbol, ticket.side, ticket.quantity, price);
            self.status.set(Some(Status::Info(format!(
                "Market order {} filled for {}x {} @ ₹{:.2}",
                order_id, ticket.quantity, ticket.symbol, price
            ))));
        } else {
            let order = WbOrder {
                order_id: order_id.clone(),
                symbol: ticket.symbol.clone(),
                order_type: ticket.order_type,
                side: ticket.side,
                quantity: ticket.quantity,
                price,
                status: "accepted".to_string(),
                product: ticket.product,
                tif: ticket.tif,
            };
            self.orders.update(|orders| orders.insert(0, order));
            self.status.set(Some(Status::Info(format!(
                "Limit order {} placed for {}x {} @ ₹{:.2}",
                order_id, ticket.quantity, ticket.symbol, price
            ))));
        }
    }

    /// Buy/sell aggregate into the position book (average-price naive, like
    /// the labs optimistic mock).
    fn apply_fill(&self, symbol: &str, side: OrderSide, quantity: i64, price: f64) {
        self.positions.update(|positions| {
            let existing = positions
                .iter_mut()
                .find(|position| position.symbol == symbol);
            match existing {
                Some(position) => {
                    let new_qty = if side.is_buy() {
                        position.quantity + quantity
                    } else {
                        position.quantity - quantity
                    };
                    if new_qty <= 0 {
                        positions.retain(|position| position.symbol != symbol);
                    } else {
                        position.quantity = new_qty;
                        position.ltp = price;
                        position.unrealized_pnl = (price - position.avg_price) * new_qty as f64;
                    }
                }
                None => {
                    if side.is_buy() {
                        positions.push(WbPosition {
                            symbol: symbol.to_string(),
                            side: OrderSide::Buy,
                            quantity,
                            avg_price: price,
                            ltp: price,
                            realized_pnl: 0.0,
                            unrealized_pnl: 0.0,
                        });
                    }
                }
            }
        });
    }

    pub fn cancel_order(&self, order_id: &str) {
        self.orders.update(|orders| {
            orders.retain(|order| order.order_id != order_id);
        });
        self.status
            .set(Some(Status::Info(format!("Order {order_id} cancelled"))));
    }

    pub fn close_position(&self, symbol: &str) {
        self.positions.update(|positions| {
            positions.retain(|position| position.symbol != symbol);
        });
        self.status
            .set(Some(Status::Info(format!("Closed position for {symbol}"))));
    }

    pub fn close_all_positions(&self) {
        self.positions.set(Vec::new());
        self.status.set(Some(Status::Info(
            "All open positions have been closed".to_string(),
        )));
    }

    pub fn set_trading_state(&self, state: TradingState) {
        self.trading_state.set(state);
        self.status.set(Some(Status::Info(format!(
            "Trading state transitioned to {}",
            state.as_id().to_uppercase()
        ))));
    }

    /// Toolbar kill switch: halt ↔ resume.
    pub fn toggle_kill_switch(&self) {
        let next = match self.trading_state.get_untracked() {
            TradingState::Halted => TradingState::Active,
            _ => TradingState::Halted,
        };
        self.set_trading_state(next);
    }

    // -- Drawer + status -----------------------------------------------------

    pub fn set_drawer_tab(&self, tab: DrawerTab) {
        self.drawer_tab.set(tab);
    }

    pub fn toggle_drawer(&self) {
        self.drawer_open.update(|open| *open = !*open);
    }

    pub fn dismiss_status(&self) {
        self.status.set(None);
    }
}

impl Default for WorkbenchStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Status {
    pub fn text(&self) -> &str {
        match self {
            Status::Info(text) | Status::Error(text) => text,
        }
    }

    pub fn as_class(&self) -> &'static str {
        match self {
            Status::Info(_) => "wb-status wb-status--info",
            Status::Error(_) => "wb-status wb-status--error",
        }
    }
}
