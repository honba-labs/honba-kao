//! Run-mode & trust badges + equity stats (HBD-2026-DES-005 P3/P5 helpers).
//!
//! The Leptos port of honba-fe `core/simulator/ops.ts`: the badge helpers the
//! run header and the validation panel render, plus the pivot-free drawdown
//! statistics the P6 equity chart reuses. Like fe, all of it is pure: badges
//! never mutate state, and the drawdown series is a run-to-run fold over the
//! equity values with no hidden timestamps.
//!
//! fe's `ops.ts` kept its validation record loosely typed
//! (`Record<string, unknown>`) because the Responses screenshot of the wire
//! carried it that way; on the Rust side the client decodes
//! [`HonbaWireValidationBlock`] directly, so [`trust_badges`] reads the typed
//! optional blocks and is exhaustive — a bad key cannot exist, and a missing
//! block yields no badges ("no validation block ⇒ nothing is inferred").
//!
//! fe's `runModeBadge` threw on an unknown mode string; the typed [`RunMode`]
//! makes that state unrepresentable, so this port never panics at the call
//! site — the string→mode mapping is the caller's parse (P4 run lifecycle),
//! and [`run_mode_badge`] takes the enum.

use honba_kao_client::wire::{Trade as WireTrade, ValidationBlock};

/// Lifecycle modes a run can ask for (`RunMode`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunMode {
    /// Simulator-only paper run (default).
    Paper,
    /// Signals are emitted but no fills are simulated.
    SignalOnly,
    /// Live execution (gated by the kill switch / approval).
    Live,
}

impl RunMode {
    /// fe's wire spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            RunMode::Paper => "paper",
            RunMode::SignalOnly => "signal_only",
            RunMode::Live => "live",
        }
    }

    /// fe's `RUN_MODES`, in order.
    pub const RUN_MODES: [RunMode; 3] = [RunMode::Paper, RunMode::SignalOnly, RunMode::Live];

    /// fe's `runModeBadge(mode)`: parse the wire's snake_case spelling; never
    /// throws — `None` means "unknown run mode" and the caller surfaces it.
    pub fn from_wire(value: &str) -> Option<RunMode> {
        match value {
            "paper" => Some(RunMode::Paper),
            "signal_only" => Some(RunMode::SignalOnly),
            "live" => Some(RunMode::Live),
            _ => {
                // fe: `throw new Error(`unknown run mode '${mode}'`)`. The
                // typed port reports the miss instead of throwing mid-render.
                None
            }
        }
    }
}

/// Visual severity of a mode badge (`RunModeBadge.level`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunModeBadgeLevel {
    /// Paper — the safe default.
    Paper,
    /// Signal only — a warning shade.
    Warning,
    /// Live — the attention shade, paired with the kill switch.
    Live,
}

/// What the run header's mode chip says (`RunModeBadge`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunModeBadge {
    /// The mode.
    pub mode: RunMode,
    /// Display label (`Paper` / `Signal only` / `Live`).
    pub label: &'static str,
    /// Visual severity.
    pub level: RunModeBadgeLevel,
    /// Live is blocked while the kill switch is on (or unapproved).
    pub blocked: bool,
    /// Whether the kill switch is currently engaged.
    pub kill_switch: bool,
}

/// The run header's mode chip (`runModeBadge`).
///
/// fe defaulted `{ killSwitch: false, approved: true }`; the typed port makes
/// both explicit so no call site can forget to pass the switch state.
pub fn run_mode_badge(mode: RunMode, kill_switch: bool, approved: bool) -> RunModeBadge {
    let blocked = mode == RunMode::Live && (kill_switch || !approved);
    let (label, level) = match mode {
        RunMode::Paper => ("Paper", RunModeBadgeLevel::Paper),
        RunMode::SignalOnly => ("Signal only", RunModeBadgeLevel::Warning),
        RunMode::Live => ("Live", RunModeBadgeLevel::Live),
    };
    RunModeBadge {
        mode,
        label,
        level,
        blocked,
        kill_switch,
    }
}

/// One validation gate chip (`TrustBadge`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrustBadge {
    /// Stable key (`dsr`, `pbo`, `holdout`, `walk_forward`, `monte_carlo`,
    /// `cost_stress`).
    pub key: &'static str,
    /// Display label.
    pub label: &'static str,
    /// Whether the gate passed.
    pub passed: bool,
}

/// fe's `TRUST_LABELS`, iterated in fe's insertion order (`ops.ts`).
const TRUST_LABELS: [(&str, &str); 6] = [
    ("dsr", "Deflated Sharpe"),
    ("pbo", "Prob. backtest overfitting"),
    ("holdout", "Hold-out"),
    ("walk_forward", "Walk-forward"),
    ("monte_carlo", "Monte Carlo"),
    ("cost_stress", "Cost stress"),
];

/// The validation panel's trust chips (`trustBadges`): one per *presented*
/// gate block, in fe's key order. A missing / not-headed block contributes no
/// badge; the panel's "no validation block ⇒ nothing is inferred" copy covers
/// the absent case.
///
/// fe read these off a loose record; here the typed wire block is matched —
/// each result wrapper carries its own `passed` verdict. The block-level
/// `passed` is deliberately *not* a badge: the panel shows per-gate chips and
/// that gate owns the aggregate decision (P5).
pub fn trust_badges(validation: Option<&ValidationBlock>) -> Vec<TrustBadge> {
    let Some(block) = validation else {
        return Vec::new();
    };
    let mut badges: Vec<TrustBadge> = Vec::new();
    let gates: [(&str, Option<bool>, &str); 6] = [
        ("dsr", block.dsr.as_ref().map(|result| result.passed), TRUST_LABELS[0].1),
        ("pbo", block.pbo.as_ref().map(|result| result.passed), TRUST_LABELS[1].1),
        (
            "holdout",
            block.holdout.as_ref().map(|result| result.passed),
            TRUST_LABELS[2].1,
        ),
        (
            "walk_forward",
            block.walk_forward.as_ref().map(|result| result.passed),
            TRUST_LABELS[3].1,
        ),
        (
            "monte_carlo",
            block.monte_carlo.as_ref().map(|result| result.passed),
            TRUST_LABELS[4].1,
        ),
        (
            "cost_stress",
            block.cost_stress.as_ref().map(|result| result.passed),
            TRUST_LABELS[5].1,
        ),
    ];
    for (key, passed, label) in gates {
        // fe: a key present (a block object) is shown, `Boolean(passed)` read
        // off it; a block that is absent is not shown. Same here.
        if let Some(passed) = passed {
            badges.push(TrustBadge {
                key,
                label,
                passed,
            });
        }
    }
    badges
}

/// Running drawdown of the equity curve, percent of running peak
/// (`drawdownSeries`). A peak ≤ 0 is reported as 0 — fe's
/// `peak <= 0 ? 0 : ((peak - value) / peak) * 100`.
pub fn drawdown_series(equity: &[f64]) -> Vec<f64> {
    let mut out: Vec<f64> = Vec::with_capacity(equity.len());
    let mut peak = f64::NEG_INFINITY;
    for &value in equity {
        peak = peak.max(value);
        out.push(if peak <= 0.0 {
            0.0
        } else {
            ((peak - value) / peak) * 100.0
        });
    }
    out
}

/// Split the run payload's curve into parallel axes (`equityFromCurve`).
///
/// fe's curve was `Array<[number, number]>` — timestamp, equity — which on
/// this side is the decoded `BacktestResponse` curve (or a run-event fold the
/// store assembles): timestamps UTC seconds, equity major units.
pub fn equity_from_curve(curve: &[(f64, f64)]) -> (Vec<f64>, Vec<f64>) {
    let ts: Vec<f64> = curve.iter().map(|(t, _)| *t).collect();
    let equity: Vec<f64> = curve.iter().map(|(_, e)| *e).collect();
    (ts, equity)
}

/// Peak-to-trough drawdown, the maximum of [`drawdown_series`]
/// (`maxDrawdownPct`). Always non-negative; `0` for an empty or all-peak
/// curve.
pub fn max_drawdown_pct(equity: &[f64]) -> f64 {
    drawdown_series(equity)
        .iter()
        .cloned()
        .fold(0.0f64, f64::max)
}

/// Count closed round trips from the raw journal fills (`closedRoundTrips`).
///
/// fe's signature was `Array<{quantity: number; side: string}>`; the typed
/// port reuses the journal's own [`WireTrade`]: the running position is
/// signed by side (`buy` positive), and every crossing to zero from a
/// non-zero balance closes one trip. A flip (position sign change without a
/// flat crossing) is fe's behaviour here too — it only counts exact flats, an
/// overrun past flat does not re-count until the next crossing — so the
/// header count can differ from the pairing module's trips (one pairing trip
/// whose `exit_reason` is `flipped` covers two of these crossings). Keep fe's
/// semantics: the header is a counter, the pairing is the truth.
///
/// (fe's `side: string` loose fill type made the guard trust a lowercase
/// `side`; the wire `Trade` is validated at its door, so the side match below
/// is exhaustive.)
pub fn closed_round_trips(fills: &[WireTrade]) -> u32 {
    let mut held = 0.0f64;
    let mut trips = 0u32;
    for fill in fills {
        let before = held;
        let sign = match fill.side() {
            honba_kao_client::wire::WireOrderSide::Buy => 1.0,
            honba_kao_client::wire::WireOrderSide::Sell => -1.0,
            // Guarded by the Trade door; a sideless fill would not parse.
            _ => 0.0,
        };
        held += sign * fill.quantity();
        if held.abs() <= 1e-9 {
            held = 0.0;
            if before.abs() > 1e-9 {
                trips += 1;
            }
        }
    }
    trips
}

#[cfg(test)]
mod tests {
    use super::*;

    // No vitest `ops.test.ts` exists in fe, so these mirror the source's own
    // examples plus the invariants P5/P6 rely on.

    #[test]
    fn run_mode_badges_cover_the_three_modes() {
        assert_eq!(
            run_mode_badge(RunMode::Paper, false, true),
            RunModeBadge {
                mode: RunMode::Paper,
                label: "Paper",
                level: RunModeBadgeLevel::Paper,
                blocked: false,
                kill_switch: false,
            }
        );
        assert_eq!(
            run_mode_badge(RunMode::SignalOnly, false, true).label,
            "Signal only"
        );
        let live = run_mode_badge(RunMode::Live, true, true);
        assert!(live.blocked);
        assert_eq!(live.level, RunModeBadgeLevel::Live);
        // Approved + switch released is the only live-ready shape.
        assert!(!run_mode_badge(RunMode::Live, false, true).blocked);
    }

    #[test]
    fn wire_mode_strings_round_trip_and_unknowns_report_none() {
        for mode in RunMode::RUN_MODES {
            assert_eq!(RunMode::from_wire(mode.as_str()), Some(mode));
        }
        // fe threw here; the typed port reports None instead.
        assert_eq!(RunMode::from_wire("nope"), None);
        // Parity with fe's 'no_order_side'-style spellings: no silent guess.
        assert_eq!(RunMode::from_wire("signal"), None);
        assert!(RunMode::RUN_MODES.contains(&RunMode::Live));
    }

    #[test]
    fn trust_badges_reflect_only_the_blocks_the_wire_presented() {
        // fe's ops.ts had no dedicated suite; this pins the P5 contract.
        assert!(trust_badges(None).is_empty());
        let mut block = ValidationBlock {
            dsr: None,
            pbo: None,
            walk_forward: None,
            holdout: None,
            monte_carlo: None,
            cost_stress: None,
            passed: false,
            warnings: Vec::new(),
        };
        assert!(trust_badges(Some(&block)).is_empty());

        // The result blocks are not re-exported on the wire (the panel only
        // reads `passed`), so the fixtures build them the way the API serves
        // them: JSON through the block's own deserializer.
        let passed_dsr = r#"{"dsr":{"observed_sharpe":1.2,"expected_max_sharpe":0.8,
            "standard_error":0.3,"z_stat":1.3,"dsr":0.9,"passed":true},"passed":true}"#;
        block = serde_json::from_str::<ValidationBlock>(passed_dsr).expect("valid block");
        let badges = trust_badges(Some(&block));
        assert_eq!(badges.len(), 1);
        assert_eq!(badges[0].key, "dsr");
        assert_eq!(badges[0].label, "Deflated Sharpe");
        assert!(badges[0].passed);

        let failed_blocks = format!(
            r#"{{"dsr":{{"observed_sharpe":1.2,"expected_max_sharpe":0.8,
            "standard_error":0.3,"z_stat":1.3,"dsr":0.9,"passed":true}},
            "pbo":{{"pbo":0.7,"combinations_evaluated":8,
            "median_oos_percentile":0.6,"passed":false}},"passed":false}}"#
        );
        block = serde_json::from_str::<ValidationBlock>(&failed_blocks).expect("valid block");
        let badges = trust_badges(Some(&block));
        assert_eq!(badges.len(), 2);
        assert_eq!(badges[1].key, "pbo");
        assert_eq!(badges[1].label, "Prob. backtest overfitting");
        assert!(!badges[1].passed);
    }

    #[test]
    fn drawdown_series_folds_the_running_peak() {
        let series = drawdown_series(&[100.0, 110.0, 88.0, 110.0]);
        assert_eq!(series.len(), 4);
        assert_eq!(series[0], 0.0);
        assert_eq!(series[1], 0.0);
        assert!((series[2] - (110.0 - 88.0) / 110.0 * 100.0).abs() < 1e-9);
        assert_eq!(series[3], 0.0);
        // A curve that spends time below its (non-positive) peak never lies.
        assert_eq!(drawdown_series(&[-5.0, -10.0]), vec![0.0, 0.0]);
        assert!(drawdown_series(&[]).is_empty());
    }

    #[test]
    fn max_drawdown_pct_is_the_worst_running_drawdown() {
        assert_eq!(max_drawdown_pct(&[]), 0.0);
        let equity = [100.0, 120.0, 60.0, 110.0];
        // Worst point is 60 against the 120 peak: 50%.
        assert!((max_drawdown_pct(&equity) - 50.0).abs() < 1e-9);
    }

    #[test]
    fn equity_from_curve_splits_the_pairs() {
        let (ts, equity) = equity_from_curve(&[(1.0, 10.0), (2.0, 12.0)]);
        assert_eq!(ts, vec![1.0, 2.0]);
        assert_eq!(equity, vec![10.0, 12.0]);
    }

    #[test]
    fn closed_round_trips_counts_only_the_flattening_crossings() {
        use honba_kao_client::wire::{
            Currency, Exchange, InstrumentId, OrderId, WireOrderSide, WireUnixNanos,
        };
        let fill = |side: WireOrderSide, quantity: f64, minute: i64| {
            let ts = 1_704_153_600 + minute * 60;
            WireTrade::new(
                OrderId::new("o"),
                InstrumentId::new("TCS", Exchange::new("NSE")),
                side,
                quantity,
                100.0,
                Currency::Inr,
                WireUnixNanos::from_u64(ts as u64 * 1_000_000_000),
                WireUnixNanos::from_u64(ts as u64 * 1_000_000_000),
            )
        };
        let fills: &[honba_kao_client::wire::Trade] = &[
            fill(WireOrderSide::Buy, 10.0, 10),
            fill(WireOrderSide::Sell, 10.0, 20),
            fill(WireOrderSide::Buy, 5.0, 30),
            fill(WireOrderSide::Sell, 3.0, 40), // partial: not closed
        ];
        // fe: buy 10, sell 10 (flat → 1), buy 5, sell 3 (still open → 1 trip).
        assert_eq!(closed_round_trips(fills), 1);
        let empty: &[honba_kao_client::wire::Trade] = &[];
        assert_eq!(closed_round_trips(empty), 0);
    }
}
