//! Screener API client bindings (`plan-screener.md` §2.2). There is no real
//! screener/scan HTTP backend reachable from this app today, so this module
//! defines the client contract as a trait — `ScreenerApiClient` — plus a
//! synchronous, bundled-data-backed default implementation,
//! `LocalScreenerApiClient`, that reuses the existing filter pipeline in
//! [`super::filter_engine`] rather than fetching anything over the network.
//! A future network-backed client can implement the same trait without the
//! store or its callers changing.

use serde::{Deserialize, Serialize};

use super::columns::ScreenerType;
use super::filter_engine::{AdvancedFilters, filter_instruments};
use super::screener_store::{ScreenDefinition, builtin_screens_registry};
use super::universes::{Universe, default_universes};
use crate::features::screener::data::instruments;

/// One row of a screen execution result.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenResultRow {
    pub symbol: String,
    pub name: String,
    /// Placeholder composite score; real scoring is Milestone 3/4 scope.
    pub score: f64,
    pub price: f64,
    pub change_percent: f64,
    pub market_cap: f64,
}

/// The current screen request, built from the store's signal state.
#[derive(Clone, Debug, PartialEq)]
pub struct ScreenQuery {
    pub screener_type: ScreenerType,
    pub market: &'static str,
    pub universe_id: String,
    pub search_query: String,
    pub quick_preset: String,
    pub filters: AdvancedFilters,
}

impl Default for ScreenQuery {
    fn default() -> Self {
        Self {
            screener_type: ScreenerType::Stocks,
            market: "IN",
            universe_id: String::new(),
            search_query: String::new(),
            quick_preset: "all".to_string(),
            filters: AdvancedFilters::default(),
        }
    }
}

/// The result of running a screen: the row set plus a timing placeholder
/// (always `0` for the local, synchronous client).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ScreenExecution {
    pub rows: Vec<ScreenResultRow>,
    pub exec_ms: u64,
}

/// A request to scan several universes at once (`plan-screener.md`'s
/// multi-market scan). Cross-universe aggregation is Milestone 3 scope.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct MultiMarketScanRequest {
    pub universe_ids: Vec<String>,
    pub query: ScreenQuery,
}

/// The screener API contract. Swappable: a future network-backed client
/// implements this trait without the store or its callers changing.
pub trait ScreenerApiClient {
    fn presets(&self) -> Vec<ScreenDefinition>;
    fn universes(&self) -> Vec<Universe>;
    fn run_screen(&self, query: &ScreenQuery) -> ScreenExecution;
    fn run_custom_formula(
        &self,
        expr: &str,
        query: &ScreenQuery,
    ) -> Result<ScreenExecution, String>;
    fn run_multi_market_scan(&self, request: &MultiMarketScanRequest) -> ScreenExecution;
}

/// The default, synchronous client backed by the bundled static instrument
/// data and the existing [`filter_instruments`] pipeline — no I/O.
pub struct LocalScreenerApiClient;

impl ScreenerApiClient for LocalScreenerApiClient {
    fn presets(&self) -> Vec<ScreenDefinition> {
        builtin_screens_registry().to_vec()
    }

    fn universes(&self) -> Vec<Universe> {
        default_universes()
    }

    fn run_screen(&self, query: &ScreenQuery) -> ScreenExecution {
        let universe =
            instruments::universe_for_screener(query.market, query.screener_type.as_id());
        let matched = filter_instruments(
            &universe,
            &query.search_query,
            &query.quick_preset,
            &query.filters,
            &[],
        );
        let rows: Vec<ScreenResultRow> = matched
            .into_iter()
            .map(|inst| ScreenResultRow {
                symbol: inst.symbol.clone(),
                name: inst.name.clone(),
                // Composite scoring is Milestone 3/4 scope.
                score: 0.0,
                price: inst.price,
                change_percent: inst.change_percent,
                market_cap: inst.market_cap,
            })
            .collect();
        ScreenExecution { rows, exec_ms: 0 }
    }

    fn run_custom_formula(
        &self,
        expr: &str,
        query: &ScreenQuery,
    ) -> Result<ScreenExecution, String> {
        if expr.trim().is_empty() {
            return Err("formula expression is empty".to_string());
        }
        // Formula parsing/evaluation (`CustomFormulaScreener`) is Milestone 3
        // scope; for now fall back to the current filter state so the
        // plumbing is exercisable end to end.
        Ok(self.run_screen(query))
    }

    fn run_multi_market_scan(&self, request: &MultiMarketScanRequest) -> ScreenExecution {
        // Cross-universe aggregation is Milestone 3 scope; for now run
        // against the current query only (a scaffold, not a real scan).
        self.run_screen(&request.query)
    }
}
