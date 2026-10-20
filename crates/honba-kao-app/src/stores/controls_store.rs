//! Simulator controls slice — the active selection (strategy graph, name,
//! origin), the control panel and its `localStorage` persistence. Owned by
//! [`super::simulator_store::SimulatorStore`].
//!
//! The memoised request plan derives from this slice alone: the selection,
//! the panel and the injected UTC day.

use std::sync::Arc;

use leptos::prelude::*;

use crate::sim::controls::{
    PeriodPreset, PlanOptions, SimulationRequestPlan, SimulatorControls, Today, default_controls,
    plan_request,
};
use crate::sim::persistence::{
    LayoutStorage, SimulatorPersisted, load_persisted, persist_selection,
};
use crate::sim::selection::{
    SelectionOrigin, StrategyGraphShape, builtin_template, controls_for_graph, universe_string,
};

// ---------------------------------------------------------------------------
// Controls patch (fe's `Partial<SimulatorControls>`)
// ---------------------------------------------------------------------------

/// A partial control update (`setControls(patch)`) — every field optional,
/// `None` leaves the current value in place.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ControlsPatch {
    /// Trading symbol.
    pub symbol: Option<String>,
    /// Exchange.
    pub exchange: Option<String>,
    /// Benchmark label.
    pub benchmark: Option<String>,
    /// Period preset.
    pub period: Option<PeriodPreset>,
    /// Bar spec (`5m`).
    pub timeframe: Option<String>,
    /// Initial capital.
    pub capital: Option<f64>,
    /// Flat commission per fill.
    pub commission: Option<f64>,
    /// Slippage, basis points.
    pub slippage: Option<f64>,
    /// Fill model id.
    pub fill_model: Option<String>,
    /// Custom start — `Some(None)` clears the field.
    pub start: Option<Option<String>>,
    /// Custom end — `Some(None)` clears the field.
    pub end: Option<Option<String>>,
}

impl ControlsPatch {
    /// Applies the present fields over `controls` (fe's `{ …current, …patch }`).
    pub fn apply(&self, controls: &mut SimulatorControls) {
        if let Some(value) = &self.symbol {
            controls.symbol = value.clone();
        }
        if let Some(value) = &self.exchange {
            controls.exchange = value.clone();
        }
        if let Some(value) = &self.benchmark {
            controls.benchmark = value.clone();
        }
        if let Some(value) = self.period {
            controls.period = value;
        }
        if let Some(value) = &self.timeframe {
            controls.timeframe = value.clone();
        }
        if let Some(value) = self.capital {
            controls.capital = value;
        }
        if let Some(value) = self.commission {
            controls.commission = value;
        }
        if let Some(value) = self.slippage {
            controls.slippage = value;
        }
        if let Some(value) = &self.fill_model {
            controls.fill_model = value.clone();
        }
        if let Some(value) = &self.start {
            controls.start = value.clone();
        }
        if let Some(value) = &self.end {
            controls.end = value.clone();
        }
    }
}

/// The current UTC calendar day from the browser's clock (fe read
/// `new Date()`'s UTC fields inside `toBacktestRequest`).
#[cfg(target_arch = "wasm32")]
pub fn utc_today() -> Today {
    let date = js_sys::Date::new_0();
    Today::new(
        date.get_utc_full_year() as i32,
        date.get_utc_month() + 1,
        date.get_utc_date(),
    )
}

// ---------------------------------------------------------------------------
// The slice
// ---------------------------------------------------------------------------

/// Selection + control panel signals and the request plan they derive.
///
/// Cloning shares the same signals; the storage handle is the only non-`Copy`
/// field.
#[derive(Clone)]
pub struct ControlsStore {
    /// The active strategy graph (`strategy`).
    pub strategy: RwSignal<StrategyGraphShape>,
    /// Its display name (`strategyName`).
    pub strategy_name: RwSignal<String>,
    /// Where the selection came from (`origin`).
    pub origin: RwSignal<SelectionOrigin>,
    /// The injected UTC day the date math runs on (fe computed `new Date()`
    /// per plan build; a signal keeps the memos correct across midnight).
    pub today: RwSignal<Today>,
    /// The control panel state (`controls`).
    pub panel: RwSignal<SimulatorControls>,
    /// Best-effort handoff storage (fe: `localStorage`); `None` in tests.
    pub storage: Option<Arc<dyn LayoutStorage>>,
    /// The memoised request plan for the current controls (fe's `plan()` was
    /// rebuilt on every read; a memo makes `lastPlan`/payload previews share
    /// one allocation per edit).
    pub plan: Memo<SimulationRequestPlan>,
}

impl ControlsStore {
    /// Template selection and default controls. fe's `initialState()` ran
    /// `resolveSelection({})`, whose template fallback is all this build
    /// resolves without the library.
    pub(super) fn new(today: Today, storage: Option<Arc<dyn LayoutStorage>>) -> Self {
        let (graph, name) = builtin_template();
        let strategy = RwSignal::new(graph);
        let strategy_name = RwSignal::new(name.to_string());
        let origin = RwSignal::new(SelectionOrigin::Template);
        let today_signal = RwSignal::new(today);
        let panel = RwSignal::new(default_controls());

        let plan = Memo::new(move |_| {
            let graph = strategy.get();
            let name = strategy_name.get();
            let controls = panel.get();
            let today = today_signal.get();
            plan_request(
                &controls,
                &PlanOptions {
                    strategy: &name,
                    universe: Some(&universe_string(&graph, &controls)),
                    seed: None,
                    today,
                },
            )
        });

        Self {
            strategy,
            strategy_name,
            origin,
            today: today_signal,
            panel,
            storage,
            plan,
        }
    }

    /// Boots the selection from whatever persistence offers (`hydrate`).
    ///
    /// fe's branches over `?strategy=` slugs, Designer hand-offs and library
    /// manifests belong to the studio port (P8). Here: a persisted selection
    /// re-installs its name/origin/controls over the template graph; nothing
    /// available means the template stays untouched. fe validated the
    /// persisted name against the library before trusting it — deferred with
    /// the library itself.
    pub fn hydrate(&self) {
        let Some(storage) = self.storage.as_ref() else {
            return;
        };
        let Some(persisted) = load_persisted(storage.as_ref()) else {
            return;
        };
        let (graph, _) = builtin_template();
        self.strategy.set(graph);
        self.strategy_name.set(persisted.strategy_name.clone());
        self.origin.set(persisted.source);
        self.panel.set(persisted.controls.clone());
    }

    /// Patches the control panel (`setControls`) — the memos re-derive.
    pub fn patch(&self, patch: ControlsPatch) {
        self.panel.update(|controls| patch.apply(controls));
    }

    /// Adopts a new selection (`select`): the graph's universe/timeframe
    /// pre-fill the panel, the rest of the controls stay, and the selection
    /// persists — a strategy switch never resets the operator's window.
    pub fn select(&self, graph: StrategyGraphShape, name: &str, origin: SelectionOrigin) {
        let controls = {
            let current = self.panel.get_untracked();
            controls_for_graph(&graph, &current)
        };
        self.strategy.set(graph);
        self.strategy_name.set(name.to_string());
        self.origin.set(origin);
        self.panel.set(controls);
        self.persist();
    }

    /// Stashes the current selection (fe's `persistSelection`, best-effort
    /// and silent in a storage-less environment).
    pub fn persist(&self) {
        if let Some(storage) = self.storage.as_ref() {
            let selection = SimulatorPersisted {
                strategy_name: self.strategy_name.get_untracked(),
                source: self.origin.get_untracked(),
                controls: self.panel.get_untracked(),
            };
            persist_selection(&selection, storage.as_ref());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// The storage double (fe's localStorage stubs).
    #[derive(Default)]
    struct FakeStorage(Mutex<String>);

    impl LayoutStorage for FakeStorage {
        fn get_item(&self, _key: &str) -> Option<String> {
            let raw = self.0.lock().expect("fake storage lock");
            (!raw.is_empty()).then(|| raw.clone())
        }

        fn set_item(&self, _key: &str, value: &str) {
            *self.0.lock().expect("fake storage lock") = value.to_string();
        }
    }

    /// Runs the body inside a reactive owner so the plan memo roots.
    fn with_controls<T>(
        storage: Option<Arc<dyn LayoutStorage>>,
        f: impl FnOnce(&ControlsStore) -> T,
    ) -> T {
        let owner = leptos::prelude::Owner::new();
        let result = owner.with(|| {
            let controls = ControlsStore::new(Today::new(2025, 1, 2), storage);
            f(&controls)
        });
        owner.unset();
        result
    }

    #[test]
    fn set_controls_patches_the_panel_and_the_plan_refollows() {
        with_controls(None, |controls| {
            // Default controls have no symbol → the plan is blocked.
            assert!(!controls.plan.get_untracked().issues.is_empty());
            controls.patch(ControlsPatch {
                symbol: Some("TCS".to_string()),
                ..Default::default()
            });
            let plan = controls.plan.get_untracked();
            assert!(plan.issues.is_empty());
            assert_eq!(plan.request.universe, Some("TCS.NSE".to_string()));
            assert_eq!(controls.panel.get_untracked().symbol, "TCS");
            // The graph's template timeframe drove the request's bar spec.
            assert_eq!(plan.request.bar_spec, Some("5m".to_string()));
        });
    }

    #[test]
    fn select_adopts_a_graph_and_persists_the_selection() {
        let storage = FakeStorage::default();
        with_controls(Some(Arc::new(storage)), |controls| {
            let (graph, _) = builtin_template();
            {
                let mut named = graph.clone();
                named.name = "Renamed".to_string();
                named.driving_interval = 15;
                controls.select(named, "Renamed", SelectionOrigin::Designer);
            }
            assert_eq!(controls.strategy_name.get_untracked(), "Renamed");
            assert_eq!(controls.origin.get_untracked(), SelectionOrigin::Designer);
            // The driving timeframe pre-filled the panel; the window stayed.
            assert_eq!(controls.panel.get_untracked().timeframe, "15m");
            // The selection landed in storage.
            assert!(
                controls
                    .storage
                    .as_ref()
                    .and_then(
                        |storage| storage.get_item(crate::sim::persistence::SIMULATOR_STORAGE_KEY)
                    )
                    .is_some()
            );
        });
    }

    #[test]
    fn hydrate_restores_a_persisted_selection() {
        let storage = FakeStorage::default();
        let panel = {
            let mut controls = default_controls();
            controls.symbol = "RELIANCE".to_string();
            controls.timeframe = "1d".to_string();
            controls
        };
        persist_selection(
            &SimulatorPersisted {
                strategy_name: "My TCS".to_string(),
                source: SelectionOrigin::Designer,
                controls: panel.clone(),
            },
            &storage,
        );
        with_controls(Some(Arc::new(storage)), |controls| {
            controls.hydrate();
            assert_eq!(controls.strategy_name.get_untracked(), "My TCS");
            assert_eq!(controls.origin.get_untracked(), SelectionOrigin::Designer);
            assert_eq!(controls.panel.get_untracked().symbol, "RELIANCE");
            // The graph itself is the template until the studio lands (P8):
            // the panel's universe field names the template's own instrument.
            assert_eq!(
                controls.strategy.get_untracked().universe_explicit,
                vec![("TCS".to_string(), "NSE".to_string())]
            );
        });
    }
}
