//! Simulator control model (HBD-2026-DES-005 P1).
//!
//! The Leptos port of honba-fe `core/simulator/controls.ts`: a pure,
//! dependency-free mapping from the control panel to the run contract. The
//! rules stay pinned to the Rust resolver in `honba-ya/crates/honba-ya-api`
//! `run_resolve.rs` — `strategy`, `universe`, `start`, `end` and a non-zero
//! `seed` are **required**; dates are `YYYY-MM-DD`; `bar_spec` matches
//! `<n><s|m|h|d|w|mo>`; `fill_model` is one of the four server spellings;
//! `commission_per_fill` and `slippage_bps` are transmitted directly. The
//! request denies unknown fields, so any control without a wire field must be
//! surfaced in `not_modelled`, never fabricated (R5).
//!
//! Dates are kept as plain `YYYY-MM-DD` strings end to end and the calendar
//! math is integer-only (no `chrono`), so there is no timezone surface: the
//! vitest expectations below were all verified against UTC dates and transfer
//! verbatim.
//!
//! The resolver remains the source of truth: if a discrepancy between this
//! model and `BacktestRequest::resolve` is ever found, **honor the server**
//! and fix the test here.

use honba_kao_client::wire::{BacktestRequest, MAX_COMMISSION_PER_FILL, MAX_SLIPPAGE_BPS};

use crate::sim::defaults::{
    DEFAULT_BENCHMARK, DEFAULT_CAPITAL, DEFAULT_FILL_MODEL, DEFAULT_SEED, DEFAULT_TIMEFRAME,
    FALLBACK_TIMEFRAME, FILL_MODEL_DESCRIPTORS,
};

/// A period preset chip on the panel (fe `PeriodPreset`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeriodPreset {
    /// Last ~1 month.
    M1,
    /// Last ~3 months.
    M3,
    /// Last ~6 months.
    M6,
    /// Last year.
    Y1,
    /// Last two years.
    Y2,
    /// Last three years.
    Y3,
    /// Last five years.
    Y5,
    /// Year to date.
    Ytd,
    /// An explicit window from the two date inputs.
    Custom,
}

impl PeriodPreset {
    /// The chip's label (`'1M'`, `'YTD'`, `'Custom'`, …).
    pub const fn label(self) -> &'static str {
        match self {
            PeriodPreset::M1 => "1M",
            PeriodPreset::M3 => "3M",
            PeriodPreset::M6 => "6M",
            PeriodPreset::Y1 => "1Y",
            PeriodPreset::Y2 => "2Y",
            PeriodPreset::Y3 => "3Y",
            PeriodPreset::Y5 => "5Y",
            PeriodPreset::Ytd => "YTD",
            PeriodPreset::Custom => "Custom",
        }
    }

    /// Every preset, in panel order.
    pub const ALL: [PeriodPreset; 9] = [
        PeriodPreset::M1,
        PeriodPreset::M3,
        PeriodPreset::M6,
        PeriodPreset::Y1,
        PeriodPreset::Y2,
        PeriodPreset::Y3,
        PeriodPreset::Y5,
        PeriodPreset::Ytd,
        PeriodPreset::Custom,
    ];
}

/// Calendar day, UTC; the calendar-free currency the date arithmetic uses.
///
/// The zone-free semantics matter (the vitest cases are all UTC): the pure date
/// math never derives a wall clock itself — the UI injects the current UTC day
/// so projections and tests agree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Today {
    /// UTC year.
    pub year: i32,
    /// UTC month, 1–12.
    pub month: u32,
    /// UTC day of month.
    pub day: u32,
}

impl Today {
    /// Builds a calendar day from its civil components.
    pub const fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    /// Renders `YYYY-MM-DD` (`isoDate`).
    pub fn to_iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// Parses `YYYY-MM-DD` (the inverse of [`to_iso`](Today::to_iso)).
    pub fn from_iso(iso: &str) -> Option<Self> {
        let bytes = iso.as_bytes();
        if iso.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
            return None;
        }
        let year = iso.get(0..4)?.parse().ok()?;
        let month = iso.get(5..7)?.parse().ok()?;
        let day = iso.get(8..10)?.parse().ok()?;
        if !(1..=12).contains(&month) || day == 0 {
            return None;
        }
        Some(Self { year, month, day })
    }
}

/// The control panel's full state, as one addressable record (fe
/// `SimulatorControls`).
#[derive(Clone, Debug, PartialEq)]
pub struct SimulatorControls {
    /// Trading symbol (`TCS`); empty means "not chosen yet".
    pub symbol: String,
    /// Exchange the symbol trades on (part of the universe).
    pub exchange: String,
    /// Benchmark label the equity overlay uses.
    pub benchmark: String,
    /// Which period the window comes from.
    pub period: PeriodPreset,
    /// Bar specification, e.g. `5m`.
    pub timeframe: String,
    /// Initial capital.
    pub capital: f64,
    /// Flat commission charged per fill (`commission_per_fill`).
    pub commission: f64,
    /// Slippage per fill, in basis points, adverse to the side (`slippage_bps`).
    pub slippage: f64,
    /// Fill model, one of the server spellings.
    pub fill_model: String,
    /// Inclusive window start, used only when `period == Custom`.
    pub start: Option<String>,
    /// Exclusive window end, used only when `period == Custom`.
    pub end: Option<String>,
}

/// The panel state the simulator opens with (fe `DEFAULT_CONTROLS`).
///
/// Note `timeframe` is `5m` (the W3 resolution): it matches the vitest suite's
/// default panel and the server accepts it for every preset.
pub fn default_controls() -> SimulatorControls {
    SimulatorControls {
        symbol: String::new(),
        exchange: "NSE".to_string(),
        benchmark: DEFAULT_BENCHMARK.to_string(),
        period: PeriodPreset::M6,
        timeframe: DEFAULT_TIMEFRAME.to_string(),
        capital: DEFAULT_CAPITAL,
        commission: 0.0,
        slippage: 0.0,
        fill_model: DEFAULT_FILL_MODEL.to_string(),
        start: None,
        end: None,
    }
}

/// The window a fixed preset spans, relative to `today`
/// (inclusive start, exclusive end = today). `presetRange`.
pub fn preset_range(preset: PeriodPreset, today: Today) -> (String, String) {
    let start = match preset {
        PeriodPreset::M1 => Some(minus_months(today, 1)),
        PeriodPreset::M3 => Some(minus_months(today, 3)),
        PeriodPreset::M6 => Some(minus_months(today, 6)),
        PeriodPreset::Y1 => Some(Today::new(today.year - 1, today.month, today.day)),
        PeriodPreset::Y2 => Some(Today::new(today.year - 2, today.month, today.day)),
        PeriodPreset::Y3 => Some(Today::new(today.year - 3, today.month, today.day)),
        PeriodPreset::Y5 => Some(Today::new(today.year - 5, today.month, today.day)),
        PeriodPreset::Ytd => Some(Today::new(today.year, 1, 1)),
        PeriodPreset::Custom => None,
    };
    let start = match start {
        Some(day) => day.to_iso(),
        None => String::new(),
    };
    (start, today.to_iso())
}

/// Subtracts whole months, clamping the day to the target month's length
/// (Jan 31 → Feb 28). fe's `minusMonths`, integer-only.
pub fn minus_months(date: Today, months: u32) -> Today {
    // Zero-based month index, which can go negative while rewinding.
    let absolute = (date.year as i64) * 12 + (date.month as i64 - 1) - (months as i64);
    let year = absolute.div_euclid(12);
    let month_index = absolute.rem_euclid(12); // 0..=11
    let year = year as i32;
    let month = month_index as u32 + 1;
    let day = date.day.min(days_in_month(year, month));
    Today::new(year, month, day)
}

/// Days in a civil month (proleptic Gregorian, matching `Date.UTC`).
fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            // Leap years divisible by 4, except turn-of-century non-400s.
            let mut is_leap = year % 4 == 0;
            if year % 100 == 0 && year % 400 != 0 {
                is_leap = false;
            }
            if is_leap { 29 } else { 28 }
        }
        _ => 30, // unreachable given `(1..=12)` callers
    }
}

/// The effective window for the current control state (`resolveRange`).
pub fn resolve_range(controls: &SimulatorControls, today: Today) -> (String, String) {
    if controls.period == PeriodPreset::Custom {
        return (
            controls.start.clone().unwrap_or_default(),
            controls.end.clone().unwrap_or_default(),
        );
    }
    preset_range(controls.period, today)
}

/// The universe string the run contract expects: `SYMBOL.EXCHANGE`
/// (`universeFor`).
pub fn universe_for(controls: &SimulatorControls) -> String {
    let symbol = controls.symbol.trim().to_uppercase();
    if symbol.is_empty() {
        return String::new();
    }
    format!("{}.{}", symbol, controls.exchange.trim().to_uppercase())
}

/// A non-negative, finite rate within `0..=max`, or a readable complaint.
fn rate_issue(label: &str, value: f64, max: f64) -> Option<String> {
    if !value.is_finite() || value < 0.0 || value > max {
        return Some(format!("{label} must be a number between 0 and {max}."));
    }
    None
}

fn bar_spec_is_valid(bar_spec: &str) -> bool {
    parse_bar_spec(bar_spec).is_some()
}

/// The `bar_spec` grammar the resolver admits: `<n><s|m|h|d|w|mo>`.
///
/// Months are a unit of the grammar with no fixed second count (the server's
/// `parse_timeframe` handles the calendar). Used by the panel validation (P1)
/// and by resampling / marker tolerance (P2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarUnit {
    /// Seconds.
    Second,
    /// Minutes.
    Minute,
    /// Hours.
    Hour,
    /// Days.
    Day,
    /// Weeks.
    Week,
    /// Calendar months.
    Month,
}

/// Parses a bar spec like `"5m"` into `(5, Minute)`; `None` for anything the
/// resolver would reject (empty digits, zero count, unknown unit).
pub fn parse_bar_spec(bar_spec: &str) -> Option<(u32, BarUnit)> {
    let digits_len = bar_spec
        .len()
        - bar_spec
            .trim_start_matches(|c: char| c.is_ascii_digit())
            .len();
    let (digits, unit) = bar_spec.split_at(digits_len);
    if digits.is_empty() || unit.is_empty() {
        return None;
    }
    let count: u32 = digits.parse().ok()?;
    if count == 0 {
        return None;
    }
    let unit = match unit {
        "s" => BarUnit::Second,
        "m" => BarUnit::Minute,
        "h" => BarUnit::Hour,
        "d" => BarUnit::Day,
        "w" => BarUnit::Week,
        "mo" => BarUnit::Month,
        _ => return None,
    };
    Some((count, unit))
}

/// A non-empty string (fe `requiredString`).
fn required_string(value: &Option<String>) -> bool {
    value.as_deref().is_some_and(|value| !value.trim().is_empty())
}

/// Human-readable control problems; empty means the run request can be built
/// (`controlIssues`).
pub fn control_issues(controls: &SimulatorControls, today: Today) -> Vec<String> {
    let mut issues = Vec::new();
    if controls.symbol.trim().is_empty() {
        issues.push("Choose a symbol to simulate.".to_string());
    }
    if !controls.capital.is_finite() || controls.capital <= 0.0 {
        issues.push("Initial capital must be a positive number.".to_string());
    }
    if !bar_spec_is_valid(controls.timeframe.trim()) {
        issues.push(format!(
            "Timeframe must look like 5m or 1d (got {:?}).",
            controls.timeframe
        ));
    }
    if !FILL_MODEL_DESCRIPTORS
        .iter()
        .any(|descriptor| descriptor.id == controls.fill_model)
    {
        issues.push(format!("Unknown fill model {:?}.", controls.fill_model));
    }
    if let Some(issue) = rate_issue("Commission", controls.commission, MAX_COMMISSION_PER_FILL) {
        issues.push(issue);
    }
    if let Some(issue) = rate_issue("Slippage", controls.slippage, MAX_SLIPPAGE_BPS) {
        issues.push(issue);
    }
    if controls.period == PeriodPreset::Custom {
        let (start, end) = resolve_range(controls, today);
        let start_ok = Today::from_iso(&start).is_some();
        let end_ok = Today::from_iso(&end).is_some();
        match (start_ok, end_ok) {
            (true, true) => {
                if start >= end {
                    issues.push("Custom start must be strictly before the end date.".to_string());
                }
            }
            _ => issues.push(
                "Custom period needs both a start and an end date (YYYY-MM-DD).".to_string(),
            ),
        }
    }
    issues
}

/// The options `plan_request` accepts (`SimulationRequestOptions`).
pub struct PlanOptions<'a> {
    /// Strategy name or manifest id, as returned by `POST /strategies`.
    pub strategy: &'a str,
    /// Overrides the symbol-derived universe.
    pub universe: Option<&'a str>,
    /// Overrides the default seed.
    pub seed: Option<u64>,
    /// The current UTC day; the panel injects it, tests pin it.
    pub today: Today,
}

/// The built request plus the panel's warning state (`SimulationRequestPlan` /
/// `ControlPlan` of the port doc).
#[derive(Clone, Debug, PartialEq)]
pub struct SimulationRequestPlan {
    /// The request as it would be submitted (`request`).
    pub request: BacktestRequest,
    /// The resolved window (`range`).
    pub range: (String, String),
    /// Controls with no wire field, deliberately not transmitted (R5). Empty
    /// while commission/slippage are carried; the field stays for parity.
    pub not_modelled: Vec<String>,
    /// Non-empty when the panel is incomplete; `request` is then best-effort.
    pub issues: Vec<String>,
}

/// Maps the control panel to a `BacktestRequest` (`toBacktestRequest` /
/// `plan_request`).
///
/// Commission and slippage go out as the versioned request fields the server
/// resolves; anything else with no wire field is reported in not_modelled,
/// never fabricated (R5). `BacktestRequest` is serialized from the crate's own
/// struct, so `deny_unknown_fields` never bites on a typo.
pub fn plan_request(
    controls: &SimulatorControls,
    options: &PlanOptions<'_>,
) -> SimulationRequestPlan {
    let range = resolve_range(controls, options.today);
    let issues = control_issues(controls, options.today);

    let request = BacktestRequest {
        strategy: Some(options.strategy.to_string()),
        universe: Some(
            options
                .universe
                .map_or_else(|| universe_for(controls), ToString::to_string),
        ),
        start: if range.0.is_empty() {
            None
        } else {
            Some(range.0.clone())
        },
        end: if range.1.is_empty() {
            None
        } else {
            Some(range.1.clone())
        },
        bar_spec: Some(if controls.timeframe.trim().is_empty() {
            FALLBACK_TIMEFRAME.to_string()
        } else {
            controls.timeframe.trim().to_string()
        }),
        initial_capital: Some(controls.capital),
        seed: Some(options.seed.unwrap_or(DEFAULT_SEED)),
        fill_model: Some(if controls.fill_model.is_empty() {
            DEFAULT_FILL_MODEL.to_string()
        } else {
            controls.fill_model.clone()
        }),
        commission_per_fill: (controls.commission > 0.0 && controls.commission.is_finite())
            .then_some(controls.commission),
        slippage_bps: (controls.slippage > 0.0 && controls.slippage.is_finite())
            .then_some(controls.slippage),
    };

    SimulationRequestPlan {
        request,
        range,
        not_modelled: Vec::new(),
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The vitest suite's pinned clock: `new Date('2026-10-17T09:30:00Z')`.
    const NOW: Today = Today::new(2026, 10, 17);

    fn controls_with(patch: impl FnOnce(&mut SimulatorControls)) -> SimulatorControls {
        let mut controls = default_controls();
        controls.symbol = "TCS".to_string();
        patch(&mut controls);
        controls
    }

    #[test]
    fn maps_each_preset_to_an_inclusive_start_and_exclusive_end_of_today() {
        assert_eq!(
            preset_range(PeriodPreset::M1, NOW),
            ("2026-09-17".to_string(), "2026-10-17".to_string())
        );
        assert_eq!(
            preset_range(PeriodPreset::M3, NOW),
            ("2026-07-17".to_string(), "2026-10-17".to_string())
        );
        assert_eq!(
            preset_range(PeriodPreset::M6, NOW),
            ("2026-04-17".to_string(), "2026-10-17".to_string())
        );
        assert_eq!(
            preset_range(PeriodPreset::Y1, NOW),
            ("2025-10-17".to_string(), "2026-10-17".to_string())
        );
        assert_eq!(
            preset_range(PeriodPreset::Y2, NOW),
            ("2024-10-17".to_string(), "2026-10-17".to_string())
        );
        assert_eq!(
            preset_range(PeriodPreset::Y3, NOW),
            ("2023-10-17".to_string(), "2026-10-17".to_string())
        );
        assert_eq!(
            preset_range(PeriodPreset::Y5, NOW),
            ("2021-10-17".to_string(), "2026-10-17".to_string())
        );
        assert_eq!(
            preset_range(PeriodPreset::Ytd, NOW),
            ("2026-01-01".to_string(), "2026-10-17".to_string())
        );
    }

    #[test]
    fn clamps_the_day_when_subtracting_months() {
        // Mar 31 → Feb 28.
        let mar31 = Today::new(2026, 3, 31);
        assert_eq!(minus_months(mar31, 1).to_iso(), "2026-02-28");
        assert_eq!(preset_range(PeriodPreset::M1, mar31).0, "2026-02-28");
        // Jan 31 → Dec 31 of the previous year (wrap).
        let jan31 = Today::new(2026, 1, 31);
        assert_eq!(minus_months(jan31, 1).to_iso(), "2025-12-31");
    }

    #[test]
    fn uses_the_explicit_window_for_a_custom_period() {
        let controls = controls_with(|c| {
            c.period = PeriodPreset::Custom;
            c.start = Some("2024-01-01".to_string());
            c.end = Some("2024-06-01".to_string());
        });
        assert_eq!(
            resolve_range(&controls, NOW),
            ("2024-01-01".to_string(), "2024-06-01".to_string())
        );
    }

    #[test]
    fn flags_a_missing_symbol_bad_capital_bad_timeframe_and_bad_fill_model() {
        let issues = control_issues(
            &controls_with(|c| {
                c.symbol = String::new();
                c.capital = 0.0;
                c.timeframe = "5x".to_string();
                c.fill_model = "magic".to_string();
            }),
            NOW,
        );
        assert!(issues.iter().any(|issue| issue.contains("symbol")));
        assert!(issues.iter().any(|issue| issue.contains("capital")));
        assert!(issues.iter().any(|issue| issue.contains("Timeframe")));
        assert!(issues.iter().any(|issue| issue.contains("fill model")));
    }

    #[test]
    fn requires_a_strictly_increasing_window_for_a_custom_period() {
        let controls = controls_with(|c| {
            c.period = PeriodPreset::Custom;
            c.start = Some("2024-06-01".to_string());
            c.end = Some("2024-01-01".to_string());
        });
        let issues = control_issues(&controls, NOW);
        assert!(issues
            .iter()
            .any(|issue| issue == "Custom start must be strictly before the end date."));
    }

    #[test]
    fn rejects_a_commission_or_slippage_the_server_resolver_would_refuse() {
        let over_slippage = controls_with(|c| {
            c.commission = -1.0;
            c.slippage = MAX_SLIPPAGE_BPS + 1.0;
        });
        let issues = control_issues(&over_slippage, NOW);
        assert!(issues.iter().any(|i| i.contains("ommission")));
        assert!(issues.iter().any(|i| i.contains("lippage")));
        let nan_commission = controls_with(|c| c.commission = f64::NAN);
        assert!(control_issues(&nan_commission, NOW)
            .iter()
            .any(|i| i.contains("ommission")));
    }

    #[test]
    fn accepts_a_commission_or_slippage_at_the_cap_exactly() {
        // Boundary: exactly the max is legal (resolver: `> max` rejects).
        let capped = controls_with(|c| {
            c.commission = MAX_COMMISSION_PER_FILL;
            c.slippage = MAX_SLIPPAGE_BPS as f64;
        });
        assert!(control_issues(&capped, NOW).is_empty());
    }

    #[test]
    fn accepts_the_default_panel() {
        assert!(control_issues(&controls_with(|_| ()), NOW).is_empty());
    }

    #[test]
    fn builds_the_required_run_fields_from_the_panel() {
        let controls = controls_with(|c| {
            c.period = PeriodPreset::M6;
            c.timeframe = "5m".to_string();
            c.capital = 250_000.0;
        });
        let options = PlanOptions {
            strategy: "sma_cross",
            universe: None,
            seed: None,
            today: NOW,
        };
        let plan = plan_request(&controls, &options);
        assert_eq!(plan.request.strategy.as_deref(), Some("sma_cross"));
        assert_eq!(plan.request.universe.as_deref(), Some("TCS.NSE"));
        assert_eq!(plan.request.start.as_deref(), Some("2026-04-17"));
        assert_eq!(plan.request.end.as_deref(), Some("2026-10-17"));
        assert_eq!(plan.request.bar_spec.as_deref(), Some("5m"));
        // Default capital + seed (server-side defaults), no commission/slippage.
        assert_eq!(plan.request.initial_capital, Some(250_000.0));
        assert_eq!(plan.request.seed, Some(DEFAULT_SEED));
        assert_eq!(plan.request.commission_per_fill, None);
        assert_eq!(plan.request.slippage_bps, None);
        assert_eq!(plan.range, ("2026-04-17".to_string(), "2026-10-17".to_string()));
        assert!(plan.not_modelled.is_empty());
        assert!(plan.issues.is_empty());
    }

    #[test]
    fn transmits_commission_and_slippage_as_the_versioned_request_fields() {
        let controls = controls_with(|c| {
            c.commission = 20.0;
            c.slippage = 5.0;
        });
        let options = PlanOptions {
            strategy: "sma",
            universe: None,
            seed: None,
            today: NOW,
        };
        let plan = plan_request(&controls, &options);
        assert_eq!(plan.request.commission_per_fill, Some(20.0));
        assert_eq!(plan.request.slippage_bps, Some(5.0));
        assert!(plan.not_modelled.is_empty());
        assert!(plan.issues.is_empty());
    }

    #[test]
    fn honours_an_explicit_universe_and_a_caller_seed() {
        let options = PlanOptions {
            strategy: "sma",
            universe: Some("nifty50"),
            seed: Some(7),
            today: NOW,
        };
        let plan = plan_request(&controls_with(|_| ()), &options);
        assert_eq!(plan.request.universe.as_deref(), Some("nifty50"));
        assert_eq!(plan.request.seed, Some(7));
    }

    #[test]
    fn derives_an_empty_universe_when_no_symbol_is_chosen() {
        let controls = controls_with(|c| c.symbol = "  ".to_string());
        assert_eq!(universe_for(&controls), "");
    }

    #[test]
    fn serializes_to_the_exact_wire_shape() {
        // `BacktestRequest` denies unknown fields; the crate struct must round
        // trip through serde with no invented keys.
        let controls = controls_with(|c| {
            c.period = PeriodPreset::M6;
            c.timeframe = "5m".to_string();
            c.capital = 250_000.0;
        });
        let options = PlanOptions {
            strategy: "sma_cross",
            universe: None,
            seed: None,
            today: NOW,
        };
        let plan = plan_request(&controls, &options);
        let json = serde_json::to_value(&plan.request).expect("serializes");
        assert!(json.is_object());
        let object = json.as_object().unwrap();
        for expected in ["strategy", "universe", "start", "end", "bar_spec", "fill_model", "initial_capital", "seed"] {
            assert!(object.contains_key(expected), "missing key {expected}");
        }
        assert!(!object.contains_key("commission_per_fill"));
        assert!(!object.contains_key("slippage_bps"));
    }

    #[test]
    fn parses_the_bar_spec_grammar() {
        assert_eq!(parse_bar_spec("5m"), Some((5, BarUnit::Minute)));
        assert_eq!(parse_bar_spec("1d"), Some((1, BarUnit::Day)));
        assert_eq!(parse_bar_spec("30s"), Some((30, BarUnit::Second)));
        assert_eq!(parse_bar_spec("1mo"), Some((1, BarUnit::Month)));
        assert_eq!(parse_bar_spec("90m"), Some((90, BarUnit::Minute)));
        assert_eq!(parse_bar_spec("1h"), Some((1, BarUnit::Hour)));
        assert_eq!(parse_bar_spec("1w"), Some((1, BarUnit::Week)));
        assert_eq!(parse_bar_spec("5x"), None);
        assert_eq!(parse_bar_spec("m"), None);
        assert_eq!(parse_bar_spec("0m"), None);
        assert_eq!(parse_bar_spec(""), None);
        assert_eq!(parse_bar_spec("5"), None);
    }
}

