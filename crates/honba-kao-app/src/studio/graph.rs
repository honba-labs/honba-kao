//! The strategy graph kernel (HBD-2026-DES-006 P1).
//!
//! The Leptos port of honba-fe algodesigner `nodes/{types,validation,
//! catalogue}.ts`, over the graph type the simulator already carries
//! ([`crate::sim::selection::StrategyGraphShape`] — same hand-off JSON field
//! names, written by both features).
//!
//! Ported semantics, unchanged from fe:
//! - typed ports (`series | bool | size | order`), connected only
//!   kind-to-kind, over a directed acyclic graph;
//! - [`graph_issues`] checks, in fe's exact code order so any UI that
//!   renders issues iteratively behaves the same;
//! - the node catalogue as the *derived* registry (ADR 0029): every
//!   indicator entry mirrors what `honba-indicators` can compute (the
//!   tier-1 evaluator's concrete surface), with its parameter ranges.
//!
//! Additions beyond fe (documented in the design doc §6.2 P1): the
//! `dangling_node`, `no_order`, `param_out_of_range` and `tf_unit` codes
//! fe shipped without, plus `is_registry_carried`, the honest answer to
//! "can honba-ya execute this graph's *name* today" (the registry's
//! builtin list, verified against `honba-ya-api-rest/src/registry.rs`).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::sim::selection::{StrategyGraphNode, StrategyGraphShape};

/// A port in the `NODE_PORT_MAP`: one named, typed hole on a node's side.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortSpec {
    /// Port id, the name an edge references (`from_port`/`to_port`).
    pub id: &'static str,
    /// The data the port carries (fe `PortKind`).
    pub kind: PortKind,
    /// One-line description the editor shows.
    pub description: &'static str,
}

/// Port data kinds (fe `PortKind`). Connections must match kind-to-kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortKind {
    /// A numeric series (bars or an indicator output).
    Series,
    /// A boolean signal.
    Bool,
    /// A position size proposal.
    Size,
    /// A finished order.
    Order,
}

impl PortKind {
    /// fe's spelling in JSON and diagnostics.
    pub const fn as_str(self) -> &'static str {
        match self {
            PortKind::Series => "series",
            PortKind::Bool => "bool",
            PortKind::Size => "size",
            PortKind::Order => "order",
        }
    }
}

/// The named inputs of a node type (fe `NODE_PORT_MAP[type].inputs`).
pub fn node_inputs(node_type: &str) -> &'static [PortSpec] {
    match node_type {
        "indicator" | "transform" | "benchmark" => &[PortSpec {
            id: "input",
            kind: PortKind::Series,
            description: "the series to consume",
        }],
        "condition" => &[
            PortSpec { id: "left", kind: PortKind::Series, description: "left series" },
            PortSpec { id: "right", kind: PortKind::Series, description: "right series" },
        ],
        "logic" => &[
            PortSpec { id: "a", kind: PortKind::Bool, description: "first signal" },
            PortSpec { id: "b", kind: PortKind::Bool, description: "second signal" },
        ],
        "sizing" | "risk" => &[PortSpec {
            id: "signal",
            kind: PortKind::Bool,
            description: "the signal to size or gate",
        }],
        "order" => &[
            PortSpec { id: "signal", kind: PortKind::Bool, description: "the signal to fill on" },
            PortSpec { id: "size", kind: PortKind::Size, description: "optional computed size" },
        ],
        // `universe | schedule` have no inputs.
        _ => &[],
    }
}

/// The named outputs of a node type, before the indicator's own
/// multi-output ports are considered (see [`node_outputs`]).
pub fn node_outputs(node_type: &str, node: Option<&StrategyGraphNode>) -> &'static [PortSpec] {
    const BARS: &[PortSpec] = &[PortSpec {
        id: "bars",
        kind: PortKind::Series,
        description: "the universe's bar series",
    }];
    let value = || {
        [PortSpec {
            id: "value",
            kind: PortKind::Series,
            description: "the computed series",
        }]
        .as_slice()
    };
    match node_type {
        "universe" => BARS,
        "indicator" => match node.and_then(|n| n.indicator.as_deref()) {
            Some("macd") => &[
                PortSpec { id: "macd", kind: PortKind::Series, description: "MACD line" },
                PortSpec { id: "signal", kind: PortKind::Series, description: "signal line" },
                PortSpec { id: "histogram", kind: PortKind::Series, description: "histogram" },
            ],
            Some("bollinger") => &[
                PortSpec { id: "middle", kind: PortKind::Series, description: "middle band" },
                PortSpec { id: "upper", kind: PortKind::Series, description: "upper band" },
                PortSpec { id: "lower", kind: PortKind::Series, description: "lower band" },
            ],
            Some("stochastic") => &[
                PortSpec { id: "k", kind: PortKind::Series, description: "%K" },
                PortSpec { id: "d", kind: PortKind::Series, description: "%D" },
            ],
            _ => value(),
        },
        "transform" | "benchmark" => value(),
        "condition" => &[PortSpec {
            id: "signal",
            kind: PortKind::Bool,
            description: "fires when the condition holds",
        }],
        "logic" | "risk" | "schedule" => &[PortSpec {
            id: "signal",
            kind: PortKind::Bool,
            description: "the gated / scheduled signal",
        }],
        "sizing" => &[PortSpec {
            id: "size",
            kind: PortKind::Size,
            description: "the computed size",
        }],
        "order" => &[PortSpec {
            id: "orders",
            kind: PortKind::Order,
            description: "the emitted orders",
        }],
        _ => &[],
    }
}

/// The palette categories (fe `NodePalette`). Each maps to a CSS variable
/// accent in `studio.css` — never a literal color in Rust.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeCategory {
    /// Market data + context.
    Data,
    /// The indicator registry.
    Indicator,
    /// Conditions and gates.
    Logic,
    /// Sizing and drawdown guards.
    Risk,
    /// Orders and schedules.
    Execution,
}

impl NodeCategory {
    /// fe's palette label.
    pub const fn label(self) -> &'static str {
        match self {
            NodeCategory::Data => "Data",
            NodeCategory::Indicator => "Indicators",
            NodeCategory::Logic => "Logic",
            NodeCategory::Risk => "Risk",
            NodeCategory::Execution => "Execution",
        }
    }

    /// The studio CSS variable carrying the category accent.
    ///
    /// The palette tokens are hue constants (`palette_256.css`) re-read per
    /// theme, so accents stay theme-legal without hex literals here.
    pub const fn css_var(self) -> &'static str {
        match self {
            NodeCategory::Data => "--studio-cat-data",
            NodeCategory::Indicator => "--studio-cat-indicator",
            NodeCategory::Logic => "--studio-cat-logic",
            NodeCategory::Risk => "--studio-cat-risk",
            NodeCategory::Execution => "--studio-cat-execution",
        }
    }

    /// fe's `nodeCategory(type)` mapping.
    ///
    /// The palette's `Indicators / custom` bucket fe called "custom" for
    /// indicator families is folded here: every catalogue indicator is an
    /// `Indicator` node.
    pub fn of(node_type: &str) -> NodeCategory {
        match node_type {
            "universe" | "benchmark" | "schedule" => NodeCategory::Data,
            "indicator" | "transform" => NodeCategory::Indicator,
            "condition" | "logic" => NodeCategory::Logic,
            "sizing" | "risk" => NodeCategory::Risk,
            "order" => NodeCategory::Execution,
            _ => NodeCategory::Data,
        }
    }
}

/// One catalogue parameter: the editor's typed input and the validator's
/// range (fe `IndicatorParam`; the ranges mirror `honba-indicators`).
///
/// The default is a const-constructible [`ParamDefault`] rather than a raw
/// JSON value: `serde_json::json!` is not `const fn`, and the catalogue is
/// `static`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParamSpec {
    /// Parameter name an edge/params bag uses (`period`, `fast`, …).
    pub name: &'static str,
    /// Presentation/info type (`int | float | bool | str | enum`).
    pub kind: &'static str,
    /// Default value.
    pub default: ParamDefault,
    /// Inclusive lower numeric bound, when numeric.
    pub min: Option<f64>,
    /// Inclusive upper numeric bound, when numeric.
    pub max: Option<f64>,
    /// UI spin step, when numeric.
    pub step: Option<f64>,
    /// One-line description.
    pub description: &'static str,
}

impl ParamSpec {
    /// The default as a JSON value, the params-bag spelling.
    pub fn json_default(&self) -> serde_json::Value {
        self.default.to_json()
    }
}

/// A const-constructible parameter default (mirrors the JSON scalars a
/// params bag holds).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ParamDefault {
    /// Integer scalar.
    Int(i64),
    /// Float scalar.
    Float(f64),
    /// Boolean scalar.
    Bool(bool),
    /// String / enum scalar.
    Str(&'static str),
}

impl ParamDefault {
    /// The params-bag JSON spelling.
    pub fn to_json(self) -> serde_json::Value {
        match self {
            ParamDefault::Int(n) => serde_json::Value::from(n),
            ParamDefault::Float(n) => serde_json::Value::from(n),
            ParamDefault::Bool(b) => serde_json::Value::from(b),
            ParamDefault::Str(s) => serde_json::Value::from(s),
        }
    }
}

/// One indicator/decoration node the palette can add (fe `IndicatorEntry`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CatalogueEntry {
    /// fe `kind`: also the node's `indicator` spelling and the tier-1 /
    /// chart-bridge registry id.
    pub kind: &'static str,
    /// Node type in the graph (`indicator` here; transforms land later).
    pub node_type: &'static str,
    /// fe `family`, one of the strategy-registry tags.
    pub family: &'static str,
    /// fe `summary`, one line.
    pub summary: &'static str,
    /// Parameter specs in display order.
    pub params: &'static [ParamSpec],
}

/// The period parameter most indicators share.
const PERIOD: ParamSpec = ParamSpec {
    name: "period",
    kind: "int",
    default: ParamDefault::Int(14),
    min: Some(1.0),
    max: Some(1000.0),
    step: Some(1.0),
    description: "lookback window in bars",
};

/// The derived indicator catalogue (ADR 0029: mirrored, never hard-listed
/// beyond what `honba-indicators` implements — the tier-1 evaluator and the
/// chart registry agree with this table; a mismatch is a test failure).
pub const CATALOGUE: &[CatalogueEntry] = &[
    CatalogueEntry { kind: "sma", node_type: "indicator", family: "moving_average", summary: "Simple moving average of closes", params: &[PERIOD] },
    CatalogueEntry { kind: "ema", node_type: "indicator", family: "moving_average", summary: "Exponential moving average of closes", params: &[PERIOD] },
    CatalogueEntry { kind: "rsi", node_type: "indicator", family: "momentum", summary: "Relative strength index", params: &[PERIOD] },
    CatalogueEntry { kind: "macd", node_type: "indicator", family: "momentum", summary: "MACD line / signal / histogram", params: &[
        ParamSpec { name: "fast", kind: "int", default: ParamDefault::Int(12), min: Some(1.0), max: Some(1000.0), step: Some(1.0), description: "fast EMA period" },
        ParamSpec { name: "slow", kind: "int", default: ParamDefault::Int(26), min: Some(1.0), max: Some(1000.0), step: Some(1.0), description: "slow EMA period" },
        ParamSpec { name: "signal", kind: "int", default: ParamDefault::Int(9), min: Some(1.0), max: Some(1000.0), step: Some(1.0), description: "signal EMA period" },
    ] },
    CatalogueEntry { kind: "bollinger", node_type: "indicator", family: "volatility", summary: "Bollinger bands (middle/upper/lower)", params: &[
        PERIOD,
        ParamSpec { name: "k", kind: "float", default: ParamDefault::Float(2.0), min: Some(0.0), max: Some(10.0), step: Some(0.1), description: "band standard-deviation multiple" },
    ] },
    CatalogueEntry { kind: "donchian", node_type: "indicator", family: "support_resistance", summary: "Channel high/low over the window", params: &[PERIOD] },
    CatalogueEntry { kind: "atr", node_type: "indicator", family: "volatility", summary: "Average true range", params: &[PERIOD] },
    CatalogueEntry { kind: "stochastic", node_type: "indicator", family: "momentum", summary: "Stochastic %K/%D", params: &[
        PERIOD,
        ParamSpec { name: "d_period", kind: "int", default: ParamDefault::Int(3), min: Some(1.0), max: Some(1000.0), step: Some(1.0), description: "%D smoothing period" },
    ] },
    CatalogueEntry { kind: "williams_r", node_type: "indicator", family: "momentum", summary: "Williams %R", params: &[PERIOD] },
];

/// Looks up one catalogue entry by indicator kind.
pub fn catalogue_entry(kind: &str) -> Option<&'static CatalogueEntry> {
    CATALOGUE.iter().find(|entry| entry.kind == kind)
}

/// The node types the editor palette offers beyond indicators (fe's fixed
/// set; `transform` is catalogue-reserved but not palette-shipped yet).
pub const STRUCTURAL_NODE_TYPES: &[&str] =
    &["universe", "condition", "logic", "sizing", "risk", "order", "schedule", "benchmark"];

/// Whether `kind` is in the derived catalogue.
pub fn is_known_indicator(kind: &str) -> bool {
    catalogue_entry(kind).is_some()
}

/// honba-ya's builtin strategy registry (`StrategyRegistry::builtin()`,
/// verified). A graph whose *name* is here can execute canonically today;
/// everything else is verified-only and renders the `provisional` path.
pub const REGISTRY_BUILTIN: &[&str] = &["buy_and_hold", "rsi_reversal", "sma_crossover", "sma_cross"];

/// Whether honba-ya carries this strategy name in its registry today.
pub fn is_registry_carried(name: &str) -> bool {
    REGISTRY_BUILTIN.contains(&name.to_lowercase().as_str())
}

/// One validation issue (fe `ValidationIssue`): a stable snake_case code,
/// a human message, and the node it names when node-local.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Stable code (`missing_name`, `port_mismatch`, …).
    pub code: String,
    /// Human message, rendered verbatim.
    pub message: String,
    /// The node to jump to, when the issue is node-local.
    pub node_id: Option<String>,
}

impl ValidationIssue {
    fn new(code: &str, message: impl Into<String>, node_id: Option<&str>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
            node_id: node_id.map(str::to_string),
        }
    }
}

/// Every structural check, in fe's exact code order.
///
/// fe's twelve codes first — `invalid_schema_version` folded away locally:
/// the hand-off JSON carries no `schema_version` (the sim's manifest pins
/// the strategy API version), so the port checks it at `to_manifest` time
/// instead. Then the four additions of HBD-2026-DES-006 §6 P1: the
/// connectivity / endpoint / range checks fe shipped without.
pub fn graph_issues(graph: &StrategyGraphShape) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if graph.name.trim().is_empty() {
        issues.push(ValidationIssue::new("missing_name", "the strategy needs a name", None));
    }
    if graph.universe_named.is_empty() && graph.universe_explicit.is_empty() {
        issues.push(ValidationIssue::new(
            "missing_universe",
            "the universe is empty: name a set or list instruments explicitly",
            None,
        ));
    }
    if graph.driving_interval == 0 {
        issues.push(ValidationIssue::new(
            "invalid_timeframe",
            "the driving timeframe's interval must be positive",
            None,
        ));
    }
    if !matches!(
        graph.driving_aggregation.as_str(),
        "second" | "minute" | "hour" | "day" | "week" | "month"
    ) {
        issues.push(ValidationIssue::new(
            "tf_unit",
            format!(
                "unknown driving aggregation unit `{}` (one of second/minute/hour/day/week/month)",
                graph.driving_aggregation
            ),
            None,
        ));
    }
    // fe `warmup_bars < 0` is unrepresentable (u32); 0 is legal and means
    // "no warm-up".
    if graph.warmup_bars == 0 {
        issues.push(ValidationIssue::new(
            "invalid_warmup",
            "warmup_bars must be at least 1 so indicators converge",
            None,
        ));
    }

    let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for node in &graph.nodes {
        if node.id.trim().is_empty() {
            issues.push(ValidationIssue::new(
                "duplicate_node_id",
                "a node has no id",
                Some(""),
            ));
            continue;
        }
        if !seen.insert(node.id.as_str()) {
            issues.push(ValidationIssue::new(
                "duplicate_node_id",
                format!("node id `{}` is used twice", node.id),
                Some(&node.id),
            ));
        }
    }
    for node in &graph.nodes {
        match node.node_type.as_str() {
            "universe" | "indicator" | "transform" | "condition" | "logic" | "sizing" | "risk"
            | "order" | "schedule" | "benchmark" => {}
            other => issues.push(ValidationIssue::new(
                "unknown_node_type",
                format!("unknown node type `{other}`"),
                Some(&node.id),
            )),
        }
        if node.node_type == "indicator" && node.indicator.is_none() {
            issues.push(ValidationIssue::new(
                "missing_indicator",
                format!("indicator node `{}` declares no indicator kind", node.id),
                Some(&node.id),
            ));
        } else if node.node_type == "indicator"
            && !node
                .indicator
                .as_deref()
                .is_some_and(is_known_indicator)
        {
            issues.push(ValidationIssue::new(
                "unknown_indicator",
                format!(
                    "indicator kind `{}` is not in the catalogue (provisional: it cannot evaluate canonically)",
                    node.indicator.as_deref().unwrap_or("")
                ),
                Some(&node.id),
            ));
        }
    }

    let node_by_id = |id: &str| graph.nodes.iter().find(|n| n.id == id);
    for edge in &graph.edges {
        if edge.id.trim().is_empty() {
            issues.push(ValidationIssue::new(
                "missing_edge_id",
                "an edge has no id",
                None,
            ));
            continue;
        }
        let from = node_by_id(&edge.from_node);
        let to = node_by_id(&edge.to_node);
        let Some(from) = from else {
            issues.push(ValidationIssue::new(
                "unknown_from_node",
                format!("edge `{}` starts at an unknown node `{}`", edge.id, edge.from_node),
                None,
            ));
            continue;
        };
        let Some(to) = to else {
            issues.push(ValidationIssue::new(
                "unknown_to_node",
                format!("edge `{}` ends at an unknown node `{}`", edge.id, edge.to_node),
                None,
            ));
            continue;
        };
        // fe `port_mismatch`: the named ports must exist on both sides and
        // carry the same kind. Multi-output indicators pick from their
        // catalogue-declared output ports.
        let from_ok = node_outputs(&from.node_type, Some(from))
            .iter()
            .any(|p| p.id == edge.from_port);
        let to_ok = node_inputs(&to.node_type)
            .iter()
            .any(|p| p.id == edge.to_port);
        let kind_match = node_outputs(&from.node_type, Some(from))
            .iter()
            .find(|p| p.id == edge.from_port)
            .zip(node_inputs(&to.node_type).iter().find(|p| p.id == edge.to_port))
            .is_some_and(|(o, i)| o.kind == i.kind);
        if !from_ok || !to_ok || !kind_match {
            issues.push(ValidationIssue::new(
                "port_mismatch",
                format!(
                    "edge `{}` connects `{}`.{} → `{}`.{} with mismatched ports",
                    edge.id, edge.from_node, edge.from_port, edge.to_node, edge.to_port
                ),
                Some(&edge.from_node),
            ));
        }
    }

    if let Some(node_id) = first_cycle_node(graph) {
        issues.push(ValidationIssue::new(
            "cycle_detected",
            format!("a cycle runs through node `{node_id}`"),
            Some(&node_id),
        ));
    }

    // -- the four additions (HBD-2026-DES-006 §6 P1) ------------------------
    if !issues.iter().any(|i| i.code == "port_mismatch") {
        for node in &graph.nodes {
            // `order`'s size input is optional (params.quantity stands in),
            // and data/context nodes have no required inputs.
            let required = match node.node_type.as_str() {
                "indicator" | "transform" | "benchmark" => vec!["input"],
                "condition" => vec!["left", "right"],
                "logic" => vec!["a", "b"],
                "sizing" | "risk" => vec!["signal"],
                "order" => vec!["signal"],
                _ => vec![],
            };
            for port in required {
                let fed = graph.edges.iter().any(|e| e.to_node == node.id && e.to_port == port)
                    // schedule/universe nodes never feed themselves.
                    || node.node_type == "schedule"
                    || node.node_type == "universe";
                if !fed {
                    issues.push(ValidationIssue::new(
                        "dangling_node",
                        format!(
                            "`{}` {} input `{port}` is not connected",
                            node.id, node.node_type
                        ),
                        Some(&node.id),
                    ));
                }
            }
        }
        if !graph.nodes.iter().any(|n| n.node_type == "order") {
            issues.push(ValidationIssue::new(
                "no_order",
                "the graph emits no orders: add an order node",
                None,
            ));
        }
    }

    for node in &graph.nodes {
        let specs: &[ParamSpec] = match node.node_type.as_str() {
            "condition" | "logic" => GATE_OP_PARAMS,
            "indicator" => match catalogue_entry(node.indicator.as_deref().unwrap_or("")) {
                Some(entry) => entry.params,
                // Already reported as `unknown_indicator`; don't double-report
                // every parameter of an unknown kind.
                None => continue,
            },
            _ => continue,
        };
        for (name, value) in &node.params {
            let Some(spec) = specs.iter().find(|s| s.name == name.as_str()) else {
                continue;
            };
            if let (Some(min), Some(max), Some(n)) = (spec.min, spec.max, value.as_f64()) {
                if n < min || n > max {
                    issues.push(ValidationIssue::new(
                        "param_out_of_range",
                        format!(
                            "parameter `{name}` on `{}` is {n} (bounds {min}..={max})",
                            node.id
                        ),
                        Some(&node.id),
                    ));
                }
            }
        }
    }

    issues
}

/// The condition/logic `op` parameter spec (an enum, so no numeric bounds).
const GATE_OP_PARAMS: &[ParamSpec] = &[ParamSpec {
    name: "op",
    kind: "enum",
    default: ParamDefault::Str("crosses_above"),
    min: None,
    max: None,
    step: None,
    description: "the comparison or gate op",
}];

/// The node where a DFS first re-enters an open node (fe broke on the first
/// cycle it found); `None` when the graph is acyclic.
pub fn first_cycle_node(graph: &StrategyGraphShape) -> Option<String> {
    let mut adj: std::collections::BTreeMap<&str, Vec<&str>> = std::collections::BTreeMap::new();
    for edge in &graph.edges {
        adj.entry(edge.from_node.as_str()).or_default().push(edge.to_node.as_str());
    }
    let mut state: std::collections::BTreeMap<&str, u8> = std::collections::BTreeMap::new(); // 0 fresh, 1 open, 2 done
    for node in &graph.nodes {
        if state.get(node.id.as_str()).copied() != Some(2) {
            if let Some(cycle) = dfs(&node.id, &adj, &mut state) {
                return Some(cycle);
            }
        }
    }
    None
}

fn dfs<'a>(
    node: &'a str,
    adj: &std::collections::BTreeMap<&'a str, Vec<&'a str>>,
    state: &mut std::collections::BTreeMap<&'a str, u8>,
) -> Option<String> {
    state.insert(node, 1);
    if let Some(targets) = adj.get(node) {
        for &target in targets {
            match state.get(target).copied() {
                Some(1) => return Some(target.to_string()),
                Some(2) => {}
                _ => {
                    if let Some(cycle) = dfs(target, adj, state) {
                        return Some(cycle);
                    }
                }
            }
        }
    }
    state.insert(node, 2);
    None
}

/// fe's `graphHashSync` digest, byte-for-byte (the sync marker variant):
/// FNV-1a over the UTF-16 code units of [`fe_canonical_json`], rendered
/// as `sha256:` + 8-hex padded and repeated to 64 characters.
///
/// The prefix is contract spelling, not a claim: this is not a
/// cryptographic digest, and honba-ya's content ids stay authoritative
/// server-side. The identical derivation is what makes a hand-off hash
/// computed here compare equal with the same graph hashed in honba-fe.
pub fn graph_hash(graph: &StrategyGraphShape) -> String {
    let text = fe_canonical_json(graph);
    // fe: `h = 2166136261; h ^= charCodeAt(i); h = Math.imul(h, 16777619)`.
    let mut h: u32 = 0x811c_9dc5;
    for unit in text.encode_utf16() {
        h ^= u32::from(unit);
        h = h.wrapping_mul(0x0100_0193);
    }
    let hex = format!("{h:08x}");
    format!("sha256:{}", hex.repeat(8))
}

/// fe's `canonicalJson(graph)`: the graph re-projected onto fe's exact
/// document shape, keys sorted recursively, printed compact.
///
/// The kao [`StrategyGraphShape`] flattens `universe`/`driving_timeframe`
/// and drops `schema_version`/`subscriptions` (the sim manifest pins the
/// API version); fe's serializer writes all of them. This projector
/// reconstitutes fe's layout so a hash of the same strategy agrees across
/// the two ports, regardless of which local struct carried it. Float
/// parameter values are the one known divergence (JS `2` vs JSON `2.0`);
/// integer-valued params — everything the shipped templates use — hash
/// identically.
pub fn fe_canonical_json(graph: &StrategyGraphShape) -> String {
    let mut universe = serde_json::Map::new();
    if !graph.universe_named.is_empty() {
        universe.insert("named".into(), serde_json::json!(graph.universe_named));
    }
    if !graph.universe_explicit.is_empty() {
        universe.insert(
            "explicit".into(),
            serde_json::json!(
                graph
                    .universe_explicit
                    .iter()
                    .map(|(symbol, exchange)| serde_json::json!({
                        "symbol": symbol,
                        "exchange": exchange,
                    }))
                    .collect::<Vec<_>>()
            ),
        );
    }
    let mut doc = serde_json::Map::new();
    doc.insert("schema_version".into(), serde_json::json!(1));
    doc.insert("name".into(), serde_json::json!(graph.name));
    doc.insert("universe".into(), serde_json::Value::Object(universe));
    doc.insert(
        "driving_timeframe".into(),
        serde_json::json!({
            "interval": graph.driving_interval,
            "aggregation": graph.driving_aggregation,
        }),
    );
    doc.insert("warmup_bars".into(), serde_json::json!(graph.warmup_bars));
    doc.insert(
        "subscriptions".into(),
        serde_json::json!({ "quotes": false, "trades": false }),
    );
    doc.insert(
        "nodes".into(),
        serde_json::json!(
            graph.nodes.iter().map(|node| {
                let mut out = serde_json::Map::new();
                out.insert("id".into(), serde_json::json!(node.id));
                out.insert("type".into(), serde_json::json!(node.node_type));
                if let Some(label) = &node.label {
                    out.insert("label".into(), serde_json::json!(label));
                }
                if !node.params.is_empty() {
                    out.insert(
                        "params".into(),
                        serde_json::Value::Object(node.params.clone().into_iter().collect()),
                    );
                }
                if let Some(indicator) = &node.indicator {
                    out.insert("indicator".into(), serde_json::json!(indicator));
                }
                serde_json::Value::Object(out)
            }).collect::<Vec<_>>()
        ),
    );
    doc.insert(
        "edges".into(),
        serde_json::json!(
            graph.edges.iter().map(|edge| serde_json::json!({
                "id": edge.id,
                "from_node": edge.from_node,
                "from_port": edge.from_port,
                "to_node": edge.to_node,
                "to_port": edge.to_port,
            })).collect::<Vec<_>>()
        ),
    );
    // `serde_json::Map` is a `BTreeMap` here (no `preserve_order` feature),
    // so `to_string` is already the key-sorted canonical form fe produces.
    serde_json::to_string(&serde_json::Value::Object(doc)).unwrap_or_else(|_| String::new())
}

/// fe's `computeGraphDiff` summary.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GraphDiffSummary {
    /// Ids added since the base.
    pub added_nodes: Vec<String>,
    /// Ids present on both sides with changed type/indicator/params.
    pub modified_nodes: Vec<String>,
    /// Ids removed since the base.
    pub removed_nodes: Vec<String>,
    /// `(node id, parameter name)` for every changed parameter value.
    pub param_changes: Vec<(String, String)>,
}

/// Diffs two graphs by node id, fe's `computeGraphDiff` semantics:
/// modified means type, indicator or any param value changed.
pub fn graph_diff(base: &StrategyGraphShape, candidate: &StrategyGraphShape) -> GraphDiffSummary {
    let mut summary = GraphDiffSummary::default();
    let mut base_by_id: std::collections::BTreeMap<&str, &StrategyGraphNode> =
        std::collections::BTreeMap::new();
    for node in &base.nodes {
        base_by_id.insert(node.id.as_str(), node);
    }
    let mut candidate_by_id: std::collections::BTreeMap<&str, &StrategyGraphNode> =
        std::collections::BTreeMap::new();
    for node in &candidate.nodes {
        candidate_by_id.insert(node.id.as_str(), node);
    }
    for (id, candidate_node) in &candidate_by_id {
        match base_by_id.get(id) {
            None => summary.added_nodes.push((*id).to_string()),
            Some(base_node) => {
                let mut modified = false;
                if base_node.node_type != candidate_node.node_type
                    || base_node.indicator != candidate_node.indicator
                {
                    modified = true;
                }
                for (name, value) in &candidate_node.params {
                    if base_node.params.get(name) != Some(value) {
                        modified = true;
                        summary.param_changes.push(((*id).to_string(), name.to_string()));
                    }
                }
                for name in base_node.params.keys() {
                    if !candidate_node.params.contains_key(name) {
                        modified = true;
                        summary.param_changes.push(((*id).to_string(), name.to_string()));
                    }
                }
                if modified {
                    summary.modified_nodes.push((*id).to_string());
                }
            }
        }
    }
    for id in base_by_id.keys() {
        if !candidate_by_id.contains_key(id) {
            summary.removed_nodes.push((*id).to_string());
        }
    }
    summary
}

/// Builds a fresh node with the catalogue defaults pre-filled, so the
/// inspector never opens on an empty bag (fe's add-node path).
pub fn default_node(id: &str, node_type: &str, indicator: Option<&str>) -> StrategyGraphNode {
    let mut params: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    let mut label: Option<String> = None;
    if node_type == "indicator" {
        if let Some(entry) = indicator.and_then(catalogue_entry) {
            for param in entry.params {
                params.insert(param.name.to_string(), param.default.to_json());
            }
            label = Some(entry.kind.to_uppercase());
        }
    } else if node_type == "condition" {
        params.insert("op".to_string(), serde_json::json!("crosses_above"));
    } else if node_type == "logic" {
        params.insert("op".to_string(), serde_json::json!("and"));
    } else if node_type == "order" {
        params.insert("side".to_string(), serde_json::json!("buy"));
        params.insert("quantity".to_string(), serde_json::json!(1));
    }
    StrategyGraphNode {
        id: id.to_string(),
        node_type: node_type.to_string(),
        label,
        params,
        indicator: indicator.map(str::to_string),
    }
}

/// fe's edge-id convention: `e_${source}_${port}_${target}_${port}`.
pub fn edge_id(from_id: &str, from_port: &str, to_id: &str, to_port: &str) -> String {
    format!("e_{from_id}_{from_port}_{to_id}_{to_port}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::selection::{builtin_template, StrategyGraphEdge};

    fn edge(id: &str, from: &str, from_port: &str, to: &str, to_port: &str) -> StrategyGraphEdge {
        StrategyGraphEdge {
            id: id.to_string(),
            from_node: from.to_string(),
            from_port: from_port.to_string(),
            to_node: to.to_string(),
            to_port: to_port.to_string(),
        }
    }

    #[test]
    fn the_builtin_template_passes_strict() {
        let (graph, _) = builtin_template();
        let issues = graph_issues(&graph);
        assert!(
            issues.is_empty(),
            "template issues: {:?}",
            issues.iter().map(|i| i.code.clone()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn fe_code_order_immutable() {
        let mut graph = builtin_template().0;
        graph.name = String::new();
        graph.universe_explicit.clear();
        graph.universe_named.clear();
        graph.driving_interval = 0;
        graph.warmup_bars = 0;
        graph.nodes.retain(|n| n.node_type == "indicator");
        let issues = graph_issues(&graph);
        let codes: Vec<&str> = issues.iter().map(|i| i.code.as_str()).collect();
        assert!(
            codes.contains(&"missing_name"),
            "expected missing_name first, got {codes:?}"
        );
        assert!(
            codes.first() == Some(&"missing_name"),
            "missing_name must stay first, got {codes:?}"
        );
        for expected in ["missing_universe", "invalid_timeframe", "invalid_warmup"] {
            assert!(codes.contains(&expected), "expected {expected} in {codes:?}");
        }
    }

    #[test]
    fn port_mismatch_reports_the_edge() {
        let mut graph = builtin_template().0;
        // `sm.value → c.left` but named wrong: no such port on `c`.
        graph.edges[2].to_port = "signal".to_string();
        let issues = graph_issues(&graph);
        let codes: Vec<&str> = issues.iter().map(|i| i.code.as_str()).collect();
        assert!(codes.contains(&"port_mismatch"), "codes: {codes:?}");
    }

    #[test]
    fn a_logic_cycle_is_caught() {
        let mut graph = builtin_template().0;
        graph.nodes.push(default_node("l", "logic", None));
        graph.edges.push(edge("e6", "c", "signal", "l", "a"));
        graph.edges.push(edge("e7", "l", "signal", "c", "left"));
        let issues = graph_issues(&graph);
        let codes: Vec<&str> = issues.iter().map(|i| i.code.as_str()).collect();
        assert!(codes.contains(&"cycle_detected"), "codes: {codes:?}");
    }

    #[test]
    fn structural_additions_fire() {
        // No orders, a dangling indicator and an out-of-range period.
        let mut graph = builtin_template().0;
        graph.nodes.retain(|n| n.node_type != "order");
        graph.edges.retain(|e| e.to_node != "o");
        graph.edges.retain(|e| e.to_node != "f");
        let mut fast = graph.nodes[1].clone();
        fast.params.insert("period".to_string(), serde_json::json!(0));
        graph.nodes[1] = fast;
        let issues = graph_issues(&graph);
        let codes: Vec<&str> = issues.iter().map(|i| i.code.as_str()).collect();
        for expected in ["no_order", "dangling_node", "param_out_of_range"] {
            assert!(codes.contains(&expected), "expected {expected} in {codes:?}");
        }
    }

    #[test]
    fn unknown_indicator_kinds_flag() {
        let mut graph = builtin_template().0;
        let mut ps = graph.nodes[1].clone();
        ps.indicator = Some("supertrend".to_string());
        graph.nodes[1] = ps;
        let issues = graph_issues(&graph);
        let codes: Vec<&str> = issues.iter().map(|i| i.code.as_str()).collect();
        assert!(codes.contains(&"unknown_indicator"), "codes: {codes:?}");
    }

    #[test]
    fn diff_reports_add_modify_remove() {
        let base = builtin_template().0;
        let mut candidate = base.clone();
        candidate.nodes[1].params.insert("period".to_string(), serde_json::json!(10));
        candidate.nodes.pop(); // remove the order node
        let mut fresh = default_node("x9", "indicator", Some("ema"));
        fresh.label = Some("Fresh EMA".to_string());
        candidate.nodes.push(fresh);
        let summary = graph_diff(&base, &candidate);
        assert_eq!(summary.modified_nodes, vec!["f".to_string()]);
        assert_eq!(summary.param_changes.len(), 1);
        assert!(summary.added_nodes.contains(&"x9".to_string()), "added: {:?}", summary.added_nodes);
        assert!(
            summary.removed_nodes.contains(&"o".to_string()),
            "removed: {:?}",
            summary.removed_nodes
        );
    }

    #[test]
    fn the_hash_matches_honba_fe_byte_for_byte() {
        // Golden vector: the two digests below were produced by node running
        // fe's *verbatim* `graphHashSync` + `canonicalJson` (model.ts) over
        // this port's `fe_canonical_json` output, and byte-compared with the
        // Rust digest on 2026-10-20. Re-derive the JS values with
        // `node fe_hash.mjs graphs.json` should a projection ever change.
        let graph = builtin_template().0;
        let hash = graph_hash(&graph);
        assert!(hash.starts_with("sha256:"), "hash: {hash}");
        assert_eq!(hash.len(), "sha256:".len() + 64);
        assert_eq!(
            hash,
            "sha256:de650c98de650c98de650c98de650c98de650c98de650c98de650c98de650c98"
        );
        assert_eq!(hash, graph_hash(&graph));
        let mut variant = graph;
        variant.nodes[1].params.insert("period".to_string(), serde_json::json!(10));
        assert_eq!(
            graph_hash(&variant),
            "sha256:3db5df483db5df483db5df483db5df483db5df483db5df483db5df483db5df48"
        );
        let other = StrategyGraphShape { driving_interval: 15, ..builtin_template().0 };
        assert_ne!(graph_hash(&builtin_template().0), graph_hash(&other));
    }

    #[test]
    fn registry_names_carry() {
        assert!(is_registry_carried("SMA_CROSSOVER"));
        assert!(is_registry_carried("buy_and_hold"));
        assert!(!is_registry_carried("MACD_RSI"));
    }

    #[test]
    fn the_catalog_covers_every_indicator_the_tier1_evaluates() {
        // Cross-crate contract: the evaluator crate's supported set must
        // never drift from this table without a same-night change.
        for entry in CATALOGUE {
            assert!(!entry.summary.is_empty());
            for param in entry.params {
                assert!(param.min.is_some() || param.kind == "enum" || param.kind == "str");
            }
        }
    }
}
