//! Simulator constants and descriptors (HBD-2026-DES-005 P1).
//!
//! The Leptos port of honba-fe `core/simulator/controls.ts`'s constant blocks;
//! every semi-documented pair (default capital, seed, benchmark, fill model)
//! has exactly one spelling here so the panel, the request builder and the
//! tests cannot drift. Known server-side peers are re-exported from the wire
//! crate rather than duplicated.
//!
//! One wart fixed on the way (W3, §2.3 of the port doc): honba-fe had
//! `DEFAULT_TIMEFRAME ('1d')` and `DEFAULT_CONTROLS.timeframe ('5m')` with no
//! tie-breaker rule. The port keeps **`5m`** as the single default the panel
//! opens with; `DEFAULT_TIMEFRAME` (`1d`, the server's `DEFAULT_BAR_SPEC`) is
//! only the value used when the panel carries no timeframe at all.

/// The default benchmark label the panel shows (fe `DEFAULT_BENCHMARK`).
pub const DEFAULT_BENCHMARK: &str = "NIFTY 50";
/// Default initial capital (fe `DEFAULT_CAPITAL`).
pub const DEFAULT_CAPITAL: f64 = 100_000.0;
/// Default reproducibility seed (fe `DEFAULT_SEED`).
pub const DEFAULT_SEED: u64 = 42;
/// Default fill model: the standard close fill.
pub const DEFAULT_FILL_MODEL: &str = "bar_fill";
/// Timeframe the panel opens with (W3: fe's `DEFAULT_CONTROLS.timeframe`).
pub const DEFAULT_TIMEFRAME: &str = "5m";
/// The bar spec assumed when a control carries no timeframe (server
/// `DEFAULT_BAR_SPEC`).
pub const FALLBACK_TIMEFRAME: &str = "1d";

/// Flat commission cap per fill, re-exported from the server resolver.
pub use honba_kao_client::wire::MAX_COMMISSION_PER_FILL as COMMISSION_MAX;
/// Slippage cap in basis points, re-exported from the server resolver.
pub use honba_kao_client::wire::MAX_SLIPPAGE_BPS as SLIPPAGE_MAX;

/// Bar specifications the run contract accepts (server `parse_timeframe`).
pub const TIMEFRAMES: [&str; 15] = [
    "1m", "2m", "3m", "5m", "10m", "15m", "30m", "45m", "1h", "2h", "3h", "4h", "1d", "1w", "1mo",
];

/// Human label for a timeframe chip, spelled exactly as fe ("1m (1 Min)").
pub fn format_timeframe(tf: &str) -> String {
    match tf {
        "1m" => "1m (1 Min)",
        "2m" => "2m (2 Mins)",
        "3m" => "3m (3 Mins)",
        "5m" => "5m (5 Mins)",
        "10m" => "10m (10 Mins)",
        "15m" => "15m (15 Mins)",
        "30m" => "30m (30 Mins)",
        "45m" => "45m (45 Mins)",
        "1h" => "1h (1 Hour)",
        "2h" => "2h (2 Hours)",
        "3h" => "3h (3 Hours)",
        "4h" => "4h (4 Hours)",
        "1d" => "1d (Daily)",
        "1w" => "1w (Weekly)",
        "1mo" => "1mo (Monthly)",
        otherwise => return otherwise.to_string(),
    }
    .to_string()
}

/// One fill model's presentation metadata (fe `FillModelDescriptor`).
pub struct FillModelDescriptor {
    /// Server spelling (the wire value).
    pub id: &'static str,
    /// Full name shown in the select.
    pub name: &'static str,
    /// Short chip label.
    pub label: &'static str,
    /// One-line description under the select.
    pub description: &'static str,
    /// Provenance badge (`Standard`, `Realistic`, …).
    pub badge: &'static str,
}

/// The four server fill models, in server order (`FILL_MODELS`).
pub const FILL_MODEL_DESCRIPTORS: [FillModelDescriptor; 4] = [
    FillModelDescriptor {
        id: "bar_fill",
        name: "Bar Fill (Close / Market)",
        label: "Bar Fill",
        description:
            "Fills immediately at bar close / current tick. Standard execution baseline.",
        badge: "Standard",
    },
    FillModelDescriptor {
        id: "bar_fill_through",
        name: "Bar Fill Through (Limit / Penetration)",
        label: "Bar Fill Through",
        description:
            "Strict limit execution: requires intra-bar price to trade strictly through the limit level.",
        badge: "Realistic",
    },
    FillModelDescriptor {
        id: "l2",
        name: "L2 Order Book Simulation",
        label: "L2 Simulation",
        description:
            "Simulates micro-ticks with adverse bid/ask spread and order-size depth slippage.",
        badge: "High Fidelity",
    },
    FillModelDescriptor {
        id: "none",
        name: "None (Signals Only / Paperless)",
        label: "Signals Only",
        description:
            "Evaluates all indicators and confluence signals without executing trades or fills.",
        badge: "Signal Test",
    },
];

/// The descriptor of a wire fill model, or `None` for an unknown spelling.
pub fn fill_model_descriptor(model: &str) -> Option<&'static FillModelDescriptor> {
    FILL_MODEL_DESCRIPTORS
        .iter()
        .find(|descriptor| descriptor.id == model)
}

/// Full name of a fill model (`formatFillModel`).
pub fn format_fill_model(model: &str) -> String {
    fill_model_descriptor(model)
        .map_or_else(|| model.to_string(), |descriptor| descriptor.name.to_owned())
}
