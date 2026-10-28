//! Workbench layout slice — the grid preset (persisted), chart panes, the
//! symbol-search and indicator-picker dialogs (indicator catalog browsing,
//! favourites/recents and per-pane applied indicators) and the bottom
//! drawer. Owned by [`super::workbench_store::WorkbenchStore`].

use std::time::Duration;

use leptos::prelude::*;

use super::order_draft_store::default_symbol;
use crate::shared::components::charts::openalgo_chart::IndicatorDef;

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
    /// A custom interval from the picker's `Custom…` row — seconds per bar.
    Custom(i64),
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

    /// The TradingView-shaped interval bar: four chips (`1m 1d 1W 1M`), each
    /// opening its denomination's flyout. The chip's own interval doubles as
    /// the smoke-test root row; `Custom…` covers everything else.
    pub const INTERVAL_GROUPS: [(&str, Timeframe, &[Timeframe]); 4] = [
        (
            "Minutes",
            Timeframe::M1,
            &[
                Timeframe::M1,
                Timeframe::M3,
                Timeframe::M5,
                Timeframe::M15,
                Timeframe::M45,
            ],
        ),
        (
            "Day",
            Timeframe::D1,
            &[
                Timeframe::H1,
                Timeframe::H2,
                Timeframe::H3,
                Timeframe::H4,
                Timeframe::D1,
                Timeframe::D3,
                Timeframe::D5,
            ],
        ),
        (
            "Week",
            Timeframe::W1,
            &[Timeframe::W1, Timeframe::W3, Timeframe::W4],
        ),
        (
            "Month",
            Timeframe::Mo1,
            &[Timeframe::Mo1, Timeframe::Mo3, Timeframe::Mo6],
        ),
    ];

    /// Compact label. `Custom` renders its canonicalised id (`5 550s` →
    /// `90m`, `86 400s` → `1D`), so `label → from_id` round-trips.
    pub fn label(self) -> String {
        match self {
            Timeframe::Custom(seconds) => custom_label(seconds),
            Timeframe::M1 => "1m".to_string(),
            Timeframe::M3 => "3m".to_string(),
            Timeframe::M5 => "5m".to_string(),
            Timeframe::M15 => "15m".to_string(),
            Timeframe::M30 => "30m".to_string(),
            Timeframe::M45 => "45m".to_string(),
            Timeframe::H1 => "1h".to_string(),
            Timeframe::H2 => "2h".to_string(),
            Timeframe::H3 => "3h".to_string(),
            Timeframe::H4 => "4h".to_string(),
            Timeframe::D1 => "1D".to_string(),
            Timeframe::D3 => "3D".to_string(),
            Timeframe::D5 => "5D".to_string(),
            Timeframe::W1 => "1W".to_string(),
            Timeframe::W3 => "3W".to_string(),
            Timeframe::W4 => "4W".to_string(),
            Timeframe::Mo1 => "1M".to_string(),
            Timeframe::Mo3 => "3M".to_string(),
            Timeframe::Mo6 => "6M".to_string(),
        }
    }

    /// Long form for the flyout row title (`15m` → `"15 Minute"`).
    pub fn full_label(self) -> String {
        match self {
            Timeframe::Custom(_) => format!("Custom {} interval", self.label()),
            Timeframe::M1 => "1 Minute".to_string(),
            Timeframe::M3 => "3 Minute".to_string(),
            Timeframe::M5 => "5 Minute".to_string(),
            Timeframe::M15 => "15 Minute".to_string(),
            Timeframe::M30 => "30 Minute".to_string(),
            Timeframe::M45 => "45 Minute".to_string(),
            Timeframe::H1 => "1 Hour".to_string(),
            Timeframe::H2 => "2 Hour".to_string(),
            Timeframe::H3 => "3 Hour".to_string(),
            Timeframe::H4 => "4 Hour".to_string(),
            Timeframe::D1 => "1 Day".to_string(),
            Timeframe::D3 => "3 Day".to_string(),
            Timeframe::D5 => "5 Day".to_string(),
            Timeframe::W1 => "1 Week".to_string(),
            Timeframe::W3 => "3 Week".to_string(),
            Timeframe::W4 => "4 Week".to_string(),
            Timeframe::Mo1 => "1 Month".to_string(),
            Timeframe::Mo3 => "3 Month".to_string(),
            Timeframe::Mo6 => "6 Month".to_string(),
        }
    }

    pub const fn interval_seconds(self) -> i64 {
        match self {
            Timeframe::Custom(seconds) => {
                if seconds < 1 {
                    1
                } else {
                    seconds
                }
            }
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
            _ => Timeframe::parse_custom(id).unwrap_or(Timeframe::D1),
        }
    }

    /// Parse a custom interval string: `30s`, `7m`, `2h`, `3d`, `2w`, `2M`
    /// (case matters: `m` = minutes, `M` = months), or a bare number which
    /// counts minutes (`875` → `14h35m`).
    pub fn parse_custom(text: &str) -> Option<Timeframe> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        let digits: String = text.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            return None;
        }
        let count: i64 = digits.parse().ok()?;
        let seconds = match &text[digits.len()..] {
            "s" => count,
            "" | "m" => count.checked_mul(60)?,
            "h" => count.checked_mul(3_600)?,
            "d" | "D" => count.checked_mul(86_400)?,
            "w" | "W" => count.checked_mul(604_800)?,
            "M" => count.checked_mul(2_592_000)?,
            _ => return None,
        };
        if (1..=63_072_000).contains(&seconds) {
            Some(Timeframe::Custom(seconds))
        } else {
            None
        }
    }
}

/// Canonical id for a custom interval (seconds → the tightest exact unit).
fn custom_label(seconds: i64) -> String {
    if seconds < 1 {
        return "custom".to_string();
    }
    if seconds % 2_592_000 == 0 {
        return format!("{}M", seconds / 2_592_000);
    }
    if seconds % 604_800 == 0 {
        return format!("{}W", seconds / 604_800);
    }
    if seconds % 86_400 == 0 {
        return format!("{}D", seconds / 86_400);
    }
    if seconds % 3_600 == 0 {
        return format!("{}h", seconds / 3_600);
    }
    if seconds % 60 == 0 {
        return format!("{}m", seconds / 60);
    }
    format!("{}s", seconds)
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

/// Layout, pane and dialog signals of the workbench.
#[derive(Clone, Copy)]
pub struct LayoutStore {
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
    pub drawer_tab: RwSignal<DrawerTab>,
    pub drawer_open: RwSignal<bool>,
}

impl LayoutStore {
    pub(super) fn new() -> Self {
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
            drawer_tab: RwSignal::new(DrawerTab::Holdings),
            drawer_open: RwSignal::new(false),
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

    // -- Layout ---------------------------------------------------------------

    pub fn set_layout(&self, layout: WorkbenchLayout) {
        self.layout.set(layout);
        save_layout(layout);
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
            crate::shared::components::charts::openalgo_chart::indicator_catalog().ok()
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

    // -- Drawer ---------------------------------------------------------------

    pub fn set_drawer_tab(&self, tab: DrawerTab) {
        self.drawer_tab.set(tab);
    }

    pub fn toggle_drawer(&self) {
        self.drawer_open.update(|open| *open = !*open);
    }
}
