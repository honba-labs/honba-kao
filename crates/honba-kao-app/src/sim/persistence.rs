//! Simulator selection persistence (HBD-2026-DES-005 P2).
//!
//! The Leptos port of honba-fe `core/simulator/persistence.ts`: a reload of
//! `/simulator` carries no query string, so the last selection is stashed
//! locally and restored on boot (acceptance: "reload preserves the last run
//! selection"). Storage is best-effort, mirroring the Designer layout and
//! library slices — a storage failure is never a feature failure.

use serde::{Deserialize, Serialize};
use web_sys::Storage;

use crate::sim::controls::{default_controls, SimulatorControls};
use crate::sim::selection::SelectionOrigin;

/// `localStorage` key the portal shares with honba-fe semantics (stable
/// spelling).
pub const SIMULATOR_STORAGE_KEY: &str = "honba.simulator.selection";

/// A [`LayoutStorage`] implementation the tests can fake.
pub trait LayoutStorage {
    /// Reads one key.
    fn get_item(&self, key: &str) -> Option<String>;
    /// Writes one key.
    fn set_item(&self, key: &str, value: &str);
}

/// The browser's `window.localStorage`, best-effort (`defaultLayoutStorage`).
pub struct LocalStorage;

impl LayoutStorage for LocalStorage {
    fn get_item(&self, key: &str) -> Option<String> {
        default_storage()?.get_item(key).ok().flatten()
    }

    fn set_item(&self, key: &str, value: &str) {
        // Storage may be full or blocked; persistence is best-effort.
        let _ = default_storage().map(|storage| storage.set_item(key, value));
    }
}

/// `window.localStorage` or `None` when unavailable (SSR, sandboxed, …).
fn default_storage() -> Option<Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

/// What the simulator stashes (`SimulatorPersisted`).
pub struct SimulatorPersisted {
    /// The selected strategy's name.
    pub strategy_name: String,
    /// Where it came from.
    pub source: SelectionOrigin,
    /// The last control state.
    pub controls: SimulatorControls,
}

/// The JSON shape — mirror of fe's (field names, `source` spelling, and the
/// `SimulatorControls` keys). Unknown fields are ignored on read and unknown
/// preset spellings fall to the default rather than failing.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedRepr {
    #[serde(default)]
    strategy_name: String,
    #[serde(default)]
    source: String,
    #[serde(default)]
    controls: ControlsRepr,
}

/// The controls as stored. Only fields the JSON carries override the defaults
/// (`{ ...DEFAULT_CONTROLS, ...parsed.controls }`).
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ControlsRepr {
    symbol: Option<String>,
    exchange: Option<String>,
    benchmark: Option<String>,
    period: Option<String>,
    timeframe: Option<String>,
    capital: Option<f64>,
    commission: Option<f64>,
    slippage: Option<f64>,
    fill_model: Option<String>,
    start: Option<Option<String>>,
    end: Option<Option<String>>,
}

fn period_from_label(label: &str) -> Option<crate::sim::controls::PeriodPreset> {
    crate::sim::controls::PeriodPreset::ALL
        .iter()
        .copied()
        .find(|preset| preset.label() == label)
}

fn controls_from_repr(repr: ControlsRepr) -> SimulatorControls {
    let mut controls = default_controls();
    if let Some(symbol) = repr.symbol {
        controls.symbol = symbol;
    }
    if let Some(exchange) = repr.exchange {
        controls.exchange = exchange;
    }
    if let Some(benchmark) = repr.benchmark {
        controls.benchmark = benchmark;
    }
    if let Some(period) = repr.period.as_deref().and_then(period_from_label) {
        controls.period = period;
    }
    if let Some(timeframe) = repr.timeframe {
        controls.timeframe = timeframe;
    }
    if let Some(capital) = repr.capital {
        controls.capital = capital;
    }
    if let Some(commission) = repr.commission {
        controls.commission = commission;
    }
    if let Some(slippage) = repr.slippage {
        controls.slippage = slippage;
    }
    if let Some(fill_model) = repr.fill_model {
        controls.fill_model = fill_model;
    }
    if let Some(start) = repr.start {
        controls.start = start;
    }
    if let Some(end) = repr.end {
        controls.end = end;
    }
    controls
}

fn controls_to_repr(controls: &SimulatorControls) -> ControlsRepr {
    ControlsRepr {
        symbol: Some(controls.symbol.clone()),
        exchange: Some(controls.exchange.clone()),
        benchmark: Some(controls.benchmark.clone()),
        period: Some(controls.period.label().to_string()),
        timeframe: Some(controls.timeframe.clone()),
        capital: Some(controls.capital),
        commission: Some(controls.commission),
        slippage: Some(controls.slippage),
        fill_model: Some(controls.fill_model.clone()),
        start: Some(controls.start.clone()),
        end: Some(controls.end.clone()),
    }
}

/// Reads the persisted selection (`loadPersisted`); `None` when storage is
/// unavailable, empty, malformed or anonymous.
pub fn load_persisted(storage: &dyn LayoutStorage) -> Option<SimulatorPersisted> {
    let raw = storage.get_item(SIMULATOR_STORAGE_KEY)?;
    let parsed: PersistedRepr = serde_json::from_str(&raw).ok()?;
    if parsed.strategy_name.is_empty() {
        return None;
    }
    Some(SimulatorPersisted {
        strategy_name: parsed.strategy_name,
        source: SelectionOrigin::from_id(&parsed.source),
        controls: controls_from_repr(parsed.controls),
    })
}

/// Writes the selection (`persistSelection`); storage failures are dropped.
pub fn persist_selection(selection: &SimulatorPersisted, storage: &dyn LayoutStorage) {
    let repr = PersistedRepr {
        strategy_name: selection.strategy_name.clone(),
        source: selection.source.id().to_string(),
        controls: controls_to_repr(&selection.controls),
    };
    if let Ok(json) = serde_json::to_string(&repr) {
        storage.set_item(SIMULATOR_STORAGE_KEY, &json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::controls::{default_controls, PeriodPreset};

    /// In-memory stand-in for `window.localStorage` (fe's test double).
    #[derive(Default)]
    struct FakeStorage {
        items: std::cell::RefCell<std::collections::HashMap<String, String>>,
    }

    impl LayoutStorage for FakeStorage {
        fn get_item(&self, key: &str) -> Option<String> {
            self.items.borrow().get(key).cloned()
        }

        fn set_item(&self, key: &str, value: &str) {
            self.items
                .borrow_mut()
                .insert(key.to_string(), value.to_string());
        }
    }

    fn persisted_with(controls: SimulatorControls) -> SimulatorPersisted {
        SimulatorPersisted {
            strategy_name: "SMA Cross".to_string(),
            source: SelectionOrigin::Designer,
            controls,
        }
    }

    #[test]
    fn round_trips_a_selection() {
        let mut controls = default_controls();
        controls.symbol = "TCS".to_string();
        controls.period = PeriodPreset::Custom;
        controls.start = Some("2024-01-01".to_string());
        controls.end = Some("2024-06-01".to_string());
        controls.commission = 20.0;
        controls.slippage = 5.0;

        let storage = FakeStorage::default();
        persist_selection(&persisted_with(controls.clone()), &storage);
        let loaded = load_persisted(&storage).expect("round trip");
        assert_eq!(loaded.strategy_name, "SMA Cross");
        assert_eq!(loaded.source, SelectionOrigin::Designer);
        assert_eq!(loaded.controls, controls);
    }

    #[test]
    fn a_malformed_document_reads_as_absent() {
        let storage = FakeStorage::default();
        storage.set_item(SIMULATOR_STORAGE_KEY, "{not json}");
        assert!(load_persisted(&storage).is_none());
    }

    #[test]
    fn an_anonymous_selection_reads_as_absent() {
        let storage = FakeStorage::default();
        storage.set_item(SIMULATOR_STORAGE_KEY, r#"{"source":"template"}"#);
        assert!(load_persisted(&storage).is_none());
    }

    #[test]
    fn an_empty_document_only_carries_defaults() {
        let storage = FakeStorage::default();
        storage.set_item(SIMULATOR_STORAGE_KEY, r#"{"strategyName":"t"}"#);
        let loaded = load_persisted(&storage).expect("strategy name kept");
        assert_eq!(loaded.source, SelectionOrigin::Template);
        assert_eq!(loaded.controls, default_controls());
    }
}
