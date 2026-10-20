//! Graph ⇄ code: the deterministic Python rendering and the lossless
//! recognizer (HBD-2026-DES-006 P1/P4).
//!
//! The Leptos port of honba-fe algodesigner `nodes/{codegen, code-to-graph}.ts`
//! — `generate_code` mirrors fe's emitter line for line, and `recognize`
//! parses the same builder-chain grammar `"fe writes"`. One honest scope cut
//! versus fe's mini-parser: [`recognize`] accepts the generated shape (any
//! argument order, edited literals, reordered block elements) but not
//! arbitrary Python; anything it cannot parse keeps every un-captured line
//! as a [`CodeSpan`] with the source going `code_only` — the ADR-0029 note
//! that source is never an execution artifact, only a rendering.
//!
//! Constants mirror honba-frontend `core/graph/codegen.ts` spelling:
//! the `# honba:strategy-graph:v1` and `# honba:strategy-graph:generated`
//! markers and `GRAPH_HASH = "…"`.

use serde_json::Value;

use crate::sim::selection::{StrategyGraphEdge, StrategyGraphNode, StrategyGraphShape};

/// The marker line above the builder block (fe grammar).
pub const GRAPH_MARKER: &str = "# honba:strategy-graph:v1";

/// The marker line above the generated strategy class (fe grammar).
pub const GENERATED_MARKER: &str = "# honba:strategy-graph:generated";

/// One not-recognized source region: 1-based inclusive line numbers and a
/// stable reason (`docstring | generated_strategy_class | unparsed`).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CodeSpan {
    /// First line, 1-based.
    pub start: u32,
    /// Last line, 1-based inclusive.
    pub end: u32,
    /// Stable reason code.
    pub reason: String,
}

/// fe's `recognize` result: a graph when the marker block parsed, the
/// `spans` for everything the studio did not capture, and `code_only` when
/// the source must not be recognized back into the canvas at all.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Recognition {
    /// The graph, when the builder block was present and parsed.
    pub graph: Option<StrategyGraphShape>,
    /// `true` when the marker is absent (fe `code_only`).
    pub code_only: bool,
    /// Every source region outside the recognized block, in order.
    pub spans: Vec<CodeSpan>,
}

/// fe's `literal()`: a value printed as a Python literal, dict keys sorted.
pub fn py_literal(value: &Value) -> String {
    match value {
        Value::Null => "None".to_string(),
        Value::Bool(b) => if *b { "True" } else { "False" }.to_string(),
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                if f.is_finite() {
                    // JSON.stringify printing: integral floats print without
                    // the decimal tail, exactly like fe's `String(value)`.
                    if f.fract() == 0.0 && f.abs() < 1e15 {
                        return format!("{}", f as i64);
                    }
                    return format!("{f}");
                }
                return "float(\"nan\")".to_string();
            }
            n.to_string()
        }
        Value::String(s) => json_escape(s),
        Value::Array(items) => format!(
            "[{}]",
            items.iter().map(py_literal).collect::<Vec<_>>().join(", ")
        ),
        Value::Object(dict) => {
            let keys: Vec<&String> = dict.keys().collect();
            format!(
                "{{{}}}",
                keys.iter()
                    .map(|key| format!("{}: {}", py_literal(&Value::from(key.as_str())), py_literal(&dict[*key])))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

/// JSON string escaping, the `JSON.stringify` subset fe relies on.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// fe's `toClassName`: non-word splits camel-cased, digit-leading prefixed.
pub fn to_class_name(name: &str) -> String {
    let ident: String = name
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect();
    let ident = if ident.is_empty() { "StrategyGraph".to_string() } else { ident };
    if ident.starts_with(|ch: char| ch.is_ascii_digit()) {
        format!("S{ident}")
    } else {
        ident
    }
}

/// fe's `toIdent`.
pub fn to_ident(text: &str) -> String {
    let ident: String = text
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() || ch == '_' { ch } else { '_' })
        .collect();
    let ident = if ident.is_empty() { "n".to_string() } else { ident };
    if ident.starts_with(|ch: char| ch.is_ascii_digit()) {
        format!("n_{ident}")
    } else {
        ident
    }
}

/// fe's `build_indicator` parameter layout: shared names get the node-local
/// spelling (`f_period`), unique names stay bare (`period`).
pub fn parameter_layout(graph: &StrategyGraphShape) -> Vec<(String, String)> {
    let mut counts: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
    for node in indicator_nodes(graph) {
        for name in node.params.keys() {
            *counts.entry(name.clone()).or_default() += 1;
        }
    }
    let mut layout = Vec::new();
    for node in indicator_nodes(graph) {
        for name in node.params.keys() {
            let spelling = if counts.get(name).copied().unwrap_or(0) > 1 {
                format!("{}_{}", to_ident(&node.id), name)
            } else {
                name.clone()
            };
            layout.push((format!("{}.{}", node.id, name), spelling));
        }
    }
    layout
}

fn indicator_nodes(graph: &StrategyGraphShape) -> Vec<&StrategyGraphNode> {
    graph
        .nodes
        .iter()
        .filter(|n| n.node_type == "indicator" && n.indicator.is_some())
        .collect()
}

fn incoming<'g>(
    graph: &'g StrategyGraphShape,
    node_id: &str,
    port: &str,
) -> Option<&'g StrategyGraphEdge> {
    graph
        .edges
        .iter()
        .find(|e| e.to_node == node_id && e.to_port == port)
}

fn incoming_ident(graph: &StrategyGraphShape, node_id: &str, port: &str) -> Option<String> {
    incoming(graph, node_id, port).map(|e| to_ident(&e.from_node))
}

/// fe's `conditionLines`.
fn condition_lines(graph: &StrategyGraphShape, node_id: &str, local: &str) -> Vec<String> {
    let node = graph.nodes.iter().find(|n| n.id == node_id);
    let op = node
        .and_then(|n| n.params.get("op"))
        .and_then(Value::as_str)
        .unwrap_or("gt");
    let left = incoming_ident(graph, node_id, "left").unwrap_or_else(|| "bar.close".to_string());
    let right = match incoming_ident(graph, node_id, "right") {
        Some(ident) => ident,
        None => py_literal(
            node.and_then(|n| n.params.get("threshold"))
                .unwrap_or(&Value::from(0)),
        ),
    };
    match op {
        "crosses_above" | "crosses_below" => {
            let symbol = if op == "crosses_above" { ">" } else { "<" };
            vec![
                format!(
                    "        {local}_now = bool({left} is not None and {right} is not None and {left} {symbol} {right})"
                ),
                format!("        {local}_was = getattr(self, '_{local}_was', False)"),
                format!("        {local} = bool({local}_now and not {local}_was)"),
                format!("        self._{local}_was = {local}_now"),
            ]
        }
        other => {
            let symbol = match other {
                "gte" => ">=",
                "lt" => "<",
                "lte" => "<=",
                "eq" => "==",
                _ => ">",
            };
            vec![format!(
                "        {local} = bool({left} is not None and {right} is not None and {left} {symbol} {right})"
            )]
        }
    }
}

/// fe's `logicLines`.
fn logic_lines(graph: &StrategyGraphShape, node_id: &str, local: &str) -> Vec<String> {
    let node = graph.nodes.iter().find(|n| n.id == node_id);
    let op = node
        .and_then(|n| n.params.get("op"))
        .and_then(Value::as_str)
        .unwrap_or("and");
    let a = incoming_ident(graph, node_id, "a").unwrap_or_else(|| "False".to_string());
    let b = incoming_ident(graph, node_id, "b").unwrap_or_else(|| "False".to_string());
    match op {
        "or" => vec![format!("        {local} = bool({a} or {b})")],
        "not" => vec![format!("        {local} = bool(not {a})")],
        _ => vec![format!("        {local} = bool({a} and {b})")],
    }
}

/// fe's `orderLines`.
fn order_lines(graph: &StrategyGraphShape, node_id: &str) -> Vec<String> {
    let node = graph.nodes.iter().find(|n| n.id == node_id);
    let side = node
        .and_then(|n| n.params.get("side"))
        .and_then(Value::as_str)
        .unwrap_or("buy");
    let signal = graph
        .edges
        .iter()
        .find(|e| e.to_node == node_id)
        .map(|e| to_ident(&e.from_node))
        .unwrap_or_else(|| "False".to_string());
    let qty = match graph
        .edges
        .iter()
        .find(|e| e.to_node == node_id && e.to_port == "size")
    {
        Some(edge) => to_ident(&edge.from_node),
        None => py_literal(
            node.and_then(|n| n.params.get("quantity"))
                .unwrap_or(&Value::from(1)),
        ),
    };
    let label = node
        .and_then(|n| n.label.clone())
        .unwrap_or_else(|| if side == "buy" { "Buy" } else { "Sell" }.to_string());
    let method = if side == "buy" { "buy" } else { "sell" };
    vec![
        format!("        if {signal}:"),
        format!("            self.{method}(bar.instrument_id, {qty}, reason={})", json_escape(&label)),
    ]
}

/// fe's `onBarBody`.
fn on_bar_body(graph: &StrategyGraphShape) -> Vec<String> {
    let mut lines = Vec::new();
    for node in &graph.nodes {
        let local = to_ident(&node.id);
        let display = node
            .label
            .clone()
            .or_else(|| node.indicator.clone())
            .unwrap_or_else(|| node.node_type.clone());
        match node.node_type.as_str() {
            "universe" => {}
            "indicator" => lines.push(format!("        {local} = self._{local}.update_bar(bar)")),
            "benchmark" => {
                lines.push(format!("        {local} = None  # benchmark series is not modelled yet"))
            }
            "condition" => lines.extend(condition_lines(graph, &node.id, &local)),
            "logic" => lines.extend(logic_lines(graph, &node.id, &local)),
            "sizing" => lines.push(format!(
                "        {local} = {}",
                py_literal(node.params.get("value").unwrap_or(&Value::from(1)))
            )),
            "risk" => lines.push(format!(
                "        {local} = {}  # risk gate: {}",
                incoming_ident(graph, &node.id, "signal").unwrap_or_else(|| "False".to_string()),
                json_escape(&display)
            )),
            "order" => lines.extend(order_lines(graph, &node.id)),
            "transform" => lines.push(format!(
                "        {local} = {}  # transform: {}",
                incoming_ident(graph, &node.id, "input").unwrap_or_else(|| "None".to_string()),
                json_escape(&display)
            )),
            "schedule" => {
                lines.push(format!("        {local} = True  # schedule: {}", json_escape(&display)))
            }
            _ => {}
        }
    }
    if lines.is_empty() {
        lines.push("        return".to_string());
    }
    lines
}

/// A JSON value built the way fe's graph object looked on the wire, for
/// `literal()`'s benefit: `universe` / `driving_timeframe` re-nested.
fn universe_value(graph: &StrategyGraphShape) -> Value {
    let mut universe = serde_json::Map::new();
    if !graph.universe_named.is_empty() {
        universe.insert("named".into(), Value::from(graph.universe_named.as_str()));
    }
    if !graph.universe_explicit.is_empty() {
        universe.insert(
            "explicit".into(),
            Value::Array(
                graph
                    .universe_explicit
                    .iter()
                    .map(|(symbol, exchange)| {
                        serde_json::json!({"symbol": symbol, "exchange": exchange})
                    })
                    .collect(),
            ),
        );
    }
    Value::Object(universe)
}

fn driving_timeframe_value(graph: &StrategyGraphShape) -> Value {
    serde_json::json!({
        "interval": graph.driving_interval,
        "aggregation": graph.driving_aggregation,
    })
}

/// fe's `generateCode`: the deterministic Python rendering.
pub fn generate_code(graph: &StrategyGraphShape, graph_hash: &str) -> String {
    let klass = to_class_name(&graph.name);
    let layout = parameter_layout(graph);
    let mut out: Vec<String> = Vec::new();
    out.push(format!("\"\"\"{klass}: generated from a Honba StrategyGraph (schema_version 1)."));
    out.push(String::new());
    out.push("DO NOT EDIT by hand: regenerate the strategy from the graph instead.".to_string());
    out.push("\"\"\"".to_string());
    out.push(String::new());
    out.push(GRAPH_MARKER.to_string());
    out.push("from __future__ import annotations".to_string());
    out.push(String::new());
    out.push(
        "from honba.strategies.base import Parameter, Strategy".to_string(),
    );
    out.push(
        "from honba.strategies.indicators import build_indicator".to_string(),
    );
    out.push("from honba.strategies import graph as _graph".to_string());
    out.push(String::new());
    out.push(format!("GRAPH_HASH = {}", json_escape(graph_hash)));
    out.push(String::new());
    out.push("GRAPH = (".to_string());
    out.push("    _graph.StrategyGraph.build(".to_string());
    out.push(format!("        name={},", py_literal(&Value::from(graph.name.as_str()))));
    out.push(format!(
        "        universe={},",
        py_literal(&universe_value(graph))
    ));
    out.push(format!(
        "        driving_timeframe={},",
        py_literal(&driving_timeframe_value(graph))
    ));
    out.push(format!("        warmup_bars={},", py_literal(&Value::from(graph.warmup_bars))));
    out.push("    )".to_string());
    out.push(
        "    .with_subscriptions(quotes=False, trades=False)".to_string(),
    );
    for node in &graph.nodes {
        let mut parts: Vec<String> = vec![
            format!("id={}", py_literal(&Value::from(node.id.as_str()))),
            format!("type={}", py_literal(&Value::from(node.node_type.as_str()))),
        ];
        if let Some(label) = &node.label {
            parts.push(format!("label={}", py_literal(&Value::from(label.as_str()))));
        }
        if let Some(indicator) = &node.indicator {
            parts.push(format!("indicator={}", py_literal(&Value::from(indicator.as_str()))));
        }
        if !node.params.is_empty() {
            let params: serde_json::Map<String, Value> = node.params.clone().into_iter().collect();
            parts.push(format!("params={}", py_literal(&Value::Object(params))));
        }
        out.push(format!("    .node({})", parts.join(", ")));
    }
    for edge in &graph.edges {
        out.push(format!(
            "    .edge(id={}, from_node={}, from_port={}, to_node={}, to_port={})",
            py_literal(&Value::from(edge.id.as_str())),
            py_literal(&Value::from(edge.from_node.as_str())),
            py_literal(&Value::from(edge.from_port.as_str())),
            py_literal(&Value::from(edge.to_node.as_str())),
            py_literal(&Value::from(edge.to_port.as_str())),
        ));
    }
    out.push(")".to_string());
    out.push(String::new());
    out.push(String::new());
    out.push(GENERATED_MARKER.to_string());
    out.push(format!("class {klass}(Strategy):"));
    out.push(format!("    name = {}", py_literal(&Value::from(graph.name.as_str()))));
    out.push(format!("    warmup_bars = {}", py_literal(&Value::from(graph.warmup_bars))));
    let indicators = indicator_nodes(graph);
    for (path, spelling) in &layout {
        let (node_id, pname) = path.split_once('.').expect("layout path");
        let node = indicators.iter().find(|n| n.id == node_id).expect("layout node");
        let value = node.params.get(pname).expect("layout param");
        out.push(format!(
            "    {spelling} = Parameter(default={}, description={})",
            py_literal(value),
            json_escape(pname)
        ));
    }
    if indicators.is_empty() {
        out.push("    pass".to_string());
    }
    out.push(String::new());
    out.push("    def on_start(self) -> None:".to_string());
    if indicators.is_empty() {
        out.push("        pass".to_string());
    } else {
        for node in &indicators {
            let mut kwargs: Vec<String> = node
                .params
                .iter()
                .map(|(name, _)| format!("{name}=self.{}", layout_value(&layout, &node.id, name)))
                .collect();
            kwargs.sort();
            let indicator = node.indicator.as_deref().unwrap_or("");
            out.push(format!(
                "        self._{} = build_indicator({}{})",
                to_ident(&node.id),
                py_literal(&Value::from(indicator)),
                if kwargs.is_empty() {
                    String::new()
                } else {
                    format!(", {}", kwargs.join(", "))
                }
            ));
        }
    }
    out.push(String::new());
    out.push("    def on_bar(self, bar) -> None:".to_string());
    out.extend(on_bar_body(graph));
    out.push(String::new());
    out.join("\n")
}

fn layout_value(layout: &[(String, String)], node_id: &str, param: &str) -> String {
    let path = format!("{node_id}.{param}");
    layout
        .iter()
        .find(|(key, _)| key == &path)
        .map(|(_, value)| value.clone())
        .unwrap_or_else(|| param.to_string())
}

// -- the recognizer ---------------------------------------------------------

// -- literal parsing (the recognizer's shared front end) ---------------------

/// Parses one Python literal starting at `text`, returning `(value, rest)`.
///
/// Covers exactly the `literal()` spellings `generate_code` can emit:
/// `None / True / False`, JSON strings, numbers, and `[...]`/`{...}`/`(...)`
/// containers. Anything else returns `None` (the caller degrades to
/// `code_only`, never to a wrong graph).
fn parse_literal(text: &str) -> Option<(Value, &str)> {
    let text = text.trim_start();
    if let Some(rest) = text.strip_prefix("None") {
        return Some((Value::Null, rest));
    }
    if let Some(rest) = text.strip_prefix("True") {
        return Some((Value::Bool(true), rest));
    }
    if let Some(rest) = text.strip_prefix("False") {
        return Some((Value::Bool(false), rest));
    }
    if let Some(rest) = text.strip_prefix('"') {
        let mut out = String::new();
        let mut chars = rest.chars();
        while let Some(ch) = chars.next() {
            match ch {
                '"' => break,
                '\\' => match chars.next() {
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some('/') => out.push('/'),
                    Some('n') => out.push('\n'),
                    Some('r') => out.push('\r'),
                    Some('t') => out.push('\t'),
                    Some('u') => {
                        let hex: String = chars.by_ref().take(4).collect();
                        let code = u32::from_str_radix(&hex, 16).ok()?;
                        out.push(char::from_u32(code).unwrap_or('\u{fffd}'));
                    }
                    _ => return None,
                },
                other => out.push(other),
            }
        }
        // `chars.as_str()` is everything after the closing quote.
        return Some((Value::from(out.as_str()), chars.as_str()));
    }
    if text.starts_with('[') || text.starts_with('{') || text.starts_with('(') {
        return parse_container(text);
    }
    // A number: consume until a delimiter.
    let end = text
        .find(|ch: char| ch == ',' || ch == ']' || ch == '}' || ch == ')' || ch.is_whitespace())
        .unwrap_or(text.len());
    let raw = &text[..end];
    let value: Value = if raw.contains('.') || raw.contains('e') || raw.contains('E') {
        let f: f64 = raw.parse().ok()?;
        serde_json::json!(f)
    } else {
        let n: i64 = raw.parse().ok()?;
        serde_json::json!(n)
    };
    Some((value, &text[end..]))
}

/// Parses `[...]`, `{...}`, and `(...)` containers recursively.
fn parse_container(text: &str) -> Option<(Value, &str)> {
    let open = text.chars().next()?;
    let closing = match open {
        '[' => ']',
        '{' => '}',
        _ => ')', // '(' and anything else
    };
    let mut rest = &text[1..];
    let mut items: Vec<Value> = Vec::new();
    let mut dict: serde_json::Map<String, Value> = serde_json::Map::new();
    loop {
        rest = rest.trim_start();
        if let Some(after) = rest.strip_prefix(closing) {
            let value = match open {
                '{' => Value::Object(dict),
                // Arrays and Python tuples round-trip to the same JSON array.
                _ => Value::Array(items),
            };
            return Some((value, after));
        }
        let (value, after) = parse_literal(rest)?;
        rest = after.trim_start();
        if open == '{' {
            rest = rest.strip_prefix(':')?.trim_start();
            let (item, after) = parse_literal(rest)?;
            let key = value.as_str()?.to_string();
            dict.insert(key, item);
            rest = after.trim_start();
        } else {
            items.push(value);
        }
        if let Some(after) = rest.strip_prefix(',') {
            rest = after;
            continue;
        }
        if rest.starts_with(closing) {
            continue;
        }
        // Neither a separator nor the close: a malformed container.
        return None;
    }
}

/// Splits `k=v, k2=v2` argument text at top level (fe relied on the same
/// generated shape: no commas inside un-bracketed strings). Each value is
/// the parsed literal, not its spelling.
fn split_args(text: &str) -> Option<Vec<(String, Value)>> {
    let mut args = Vec::new();
    let mut rest = text.trim();
    while !rest.is_empty() {
        let (key, after) = parse_literal(rest)?;
        let key = key.as_str()?.to_string();
        let after = after.trim_start().strip_prefix('=')?;
        let (value, tail) = parse_literal(after)?;
        args.push((key, value));
        rest = tail.trim_start().trim_start_matches(',').trim_start();
    }
    Some(args)
}

/// Applies a parsed `universe={...}` literal to the graph.
fn apply_universe(graph: &mut StrategyGraphShape, value: &Value) {
    if let Value::Object(map) = value {
        if let Some(named) = map.get("named").and_then(Value::as_str) {
            graph.universe_named = named.to_string();
        }
        if let Some(Value::Array(items)) = map.get("explicit") {
            for item in items {
                let symbol = item.get("symbol").and_then(Value::as_str).unwrap_or("");
                let exchange = item.get("exchange").and_then(Value::as_str).unwrap_or("");
                graph.universe_explicit.push((symbol.to_string(), exchange.to_string()));
            }
        }
    }
}

/// Applies a parsed `driving_timeframe={...}` literal to the graph.
fn apply_timeframe(graph: &mut StrategyGraphShape, value: &Value) {
    if let Value::Object(map) = value {
        if let Some(interval) = map.get("interval").and_then(Value::as_u64) {
            graph.driving_interval = interval.min(u32::MAX as u64) as u32;
        }
        if let Some(aggregation) = map.get("aggregation").and_then(Value::as_str) {
            graph.driving_aggregation = aggregation.to_string();
        }
    }
}

/// Builds a node from one parsed `.node(...)` argument list.
fn node_from_args(args: &[(String, Value)]) -> StrategyGraphNode {
    let mut node = StrategyGraphNode::default();
    let mut params: serde_json::Map<String, Value> = serde_json::Map::new();
    for (key, value) in args {
        match key.as_str() {
            "id" => node.id = value_to_string(value),
            "type" => node.node_type = value_to_string(value),
            "label" => node.label = Some(value_to_string(value)),
            "indicator" => node.indicator = Some(value_to_string(value)),
            "params" => {
                if let Value::Object(map) = value {
                    params = map.clone();
                }
            }
            // A hoisted scalar (possible in a hand edit): land it in the
            // params bag so nothing the author wrote is silently dropped.
            other => {
                params.insert(other.to_string(), value.clone());
            }
        }
    }
    node.params = params.into_iter().collect();
    node
}

fn value_to_string(value: &Value) -> String {
    value.as_str().unwrap_or("").to_string()
}

/// fe's `codeToGraph` recognition, scoped to the emitted grammar.
///
/// Returns `code_only` when the `# honba:strategy-graph:v1` marker is
/// absent; otherwise every line outside the builder block lands in `spans`
/// with a stable reason, and a parse failure inside the block degrades to
/// one `unparsed` span and no graph rather than a wrong graph.
pub fn recognize(source: &str) -> Recognition {
    let lines: Vec<&str> = source.lines().collect();
    let Some(marker_index) = lines.iter().position(|l| l.trim() == GRAPH_MARKER) else {
        return Recognition {
            spans: vec![CodeSpan {
                start: 1,
                end: lines.len() as u32,
                reason: "generated_strategy_class".to_string(),
            }],
            code_only: true,
            graph: None,
        };
    };

    let mut graph = StrategyGraphShape::default();
    let mut index = marker_index + 1;
    let mut builder_ok = true;
    let mut closed = false;

    let arg = |args: &[(String, Value)], name: &str| -> Option<Value> {
        args.iter().find(|(key, _)| key == name).map(|(_, value)| value.clone())
    };

    while index < lines.len() {
        let trimmed = lines[index].trim();
        if trimmed == ")" {
            closed = true;
            break;
        }
        if trimmed.is_empty() || trimmed.starts_with("GRAPH_HASH = ") {
            // The hash echo is informational; the caller hashes graphs,
            // not code.
        } else {
            let (maybe_args, kind): (Option<Vec<(String, Value)>>, &'static str) =
                if let Some(inner) = trimmed
                    .strip_suffix(',')
                    .unwrap_or(trimmed)
                    .strip_prefix(".node(")
                {
                    (split_args(inner.trim_end_matches(')')), "node")
                } else if let Some(inner) = trimmed
                    .strip_suffix(',')
                    .unwrap_or(trimmed)
                    .strip_prefix(".edge(")
                {
                    (split_args(inner.trim_end_matches(')')), "edge")
                } else {
                    (split_args(trimmed.trim_end_matches(',').trim_end_matches(')')), "kw")
                };
            if let Some(args) = maybe_args {
                match kind {
                    "node" => graph.nodes.push(node_from_args(&args)),
                    "edge" => {
                        let get = |name: &str| -> String {
                            arg(&args, name)
                                .and_then(|v| v.as_str().map(str::to_string))
                                .unwrap_or_default()
                        };
                        graph.edges.push(StrategyGraphEdge {
                            id: get("id"),
                            from_node: get("from_node"),
                            from_port: get("from_port"),
                            to_node: get("to_node"),
                            to_port: get("to_port"),
                        });
                    }
                    _ => {
                        for (key, value) in &args {
                            match key.as_str() {
                                "name" => graph.name = value_to_string(value),
                                "warmup_bars" => {
                                    graph.warmup_bars =
                                        value.as_u64().unwrap_or(0).min(u32::MAX as u64) as u32;
                                }
                                "universe" => apply_universe(&mut graph, value),
                                "driving_timeframe" => apply_timeframe(&mut graph, value),
                                _ => {}
                            }
                        }
                    }
                }
            } else if trimmed == "GRAPH = ("
                || trimmed.starts_with("from ")
                || trimmed.starts_with("_graph.StrategyGraph.build(")
                || trimmed.starts_with(".with_subscriptions(")
            {
                // Structural: the builder opener, the import block, and the
                // subscriptions line (fixed False/False, pinned by the
                // manifest) carry no graph data.
            } else {
                builder_ok = false;
            }
        }
        index += 1;
    }

    if !closed || !builder_ok {
        return Recognition {
            spans: vec![CodeSpan {
                start: marker_index as u32 + 1,
                end: lines.len() as u32,
                reason: "unparsed".to_string(),
            }],
            code_only: true,
            graph: None,
        };
    }

    let mut spans = Vec::new();
    if marker_index > 0 {
        spans.push(CodeSpan {
            start: 1,
            end: marker_index as u32,
            reason: "docstring".to_string(),
        });
    }
    let after_block = index as u32 + 1; // the line after the closing `)`
    if after_block <= lines.len() as u32 {
        spans.push(CodeSpan {
            start: after_block,
            end: lines.len() as u32,
            reason: "generated_strategy_class".to_string(),
        });
    }
    Recognition {
        graph: Some(graph),
        code_only: false,
        spans,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::selection::builtin_template;
    use crate::studio::graph::graph_hash;

    #[test]
    fn generated_code_round_trips_losslessly() {
        let graph = builtin_template().0;
        let code = generate_code(&graph, &graph_hash(&graph));
        let recognition = recognize(&code);
        assert!(!recognition.code_only, "the studio's own code recognizes");
        let recovered = recognition.graph.expect("round trip");
        assert_eq!(recovered, builtin_template().0, "lossless");
        // Every line is accounted for: the docstring span, the marker block
        // itself, and the generated class span — no overlap, no gap.
        let lines = code.lines().count() as u32;
        let block = marker_block_lines(&code);
        let spans_total: u32 = recognition.spans.iter().map(|s| s.end - s.start + 1).sum();
        assert_eq!(spans_total + block, lines, "spans: {:?}", recognition.spans);
    }

    fn marker_block_lines(code: &str) -> u32 {
        let lines: Vec<&str> = code.lines().collect();
        let start = lines
            .iter()
            .position(|l| l.trim() == GRAPH_MARKER)
            .expect("marker");
        1 + lines[start..]
            .iter()
            .take_while(|l| l.trim() != ")" && !(l.trim()).starts_with(GENERATED_MARKER))
            .count() as u32
    }

    #[test]
    fn a_hand_edit_recovers_an_edited_param() {
        let graph = builtin_template().0;
        let code = generate_code(&builtin_template().0, "sha256:x");
        // A hand edit: fast period 5 → 7, exactly the panel's edit shape.
        let edited = code.replacen(
            "params={\"period\": 5}",
            "params={\"period\": 7}",
            1,
        );
        assert_ne!(edited, code, "the edit must change the source");
        let recognition = recognize(&edited);
        let recovered = recognition.graph.expect("edited code still recognizes");
        let fast = recovered.nodes.iter().find(|n| n.id == "f").expect("node");
        assert_eq!(fast.params["period"], serde_json::json!(7));
    }

    #[test]
    fn code_only_for_a_plain_python_file() {
        let code = "class Momentum(Strategy):\n    pass\n";
        let recognition = recognize(code);
        assert!(recognition.code_only);
        assert!(recognition.graph.is_none());
        assert_eq!(recognition.spans.len(), 1);
        assert_eq!(recognition.spans[0].reason, "generated_strategy_class");
    }

    #[test]
    fn generated_markers_are_stable_spellings() {
        let graph = builtin_template().0;
        let code = generate_code(&graph, "sha256:test");
        assert!(code.contains(GRAPH_MARKER));
        assert!(code.contains(GENERATED_MARKER));
        assert!(code.contains("GRAPH_HASH = \"sha256:test\""));
        assert!(code.contains("_graph.StrategyGraph.build("));
        assert!(code.contains("class SMACross(Strategy):"));
    }

    #[test]
    fn py_literal_matches_fe_spelling() {
        assert_eq!(py_literal(&Value::Null), "None");
        assert_eq!(py_literal(&Value::Bool(true)), "True");
        assert_eq!(py_literal(&serde_json::json!(5)), "5");
        assert_eq!(py_literal(&serde_json::json!(2.0)), "2");
        assert_eq!(py_literal(&serde_json::json!(0.5)), "0.5");
        assert_eq!(py_literal(&serde_json::json!("a\"b")), "\"a\\\"b\"");
        let dict: serde_json::Map<String, Value> =
            serde_json::json!({"period": 5, "kind": "sma"}).as_object().expect("obj").clone();
        // Dict keys print sorted, exactly like fe's literal().
        assert_eq!(py_literal(&Value::Object(dict)), "{\"kind\": \"sma\", \"period\": 5}");
    }

    #[test]
    fn parameter_layout_dedupes_shared_names() {
        let graph = builtin_template().0;
        let layout: std::collections::BTreeMap<String, String> =
            parameter_layout(&graph).into_iter().collect();
        // Both SMAs carry `period` → node-local spelling.
        assert_eq!(layout["f.period"], "f_period");
        assert_eq!(layout["s.period"], "s_period");
    }

    #[test]
    fn class_name_and_ident_scopes_survive_odd_names() {
        // fe capitalizes each part's first rune only; existing caps stay.
        assert_eq!(to_class_name("sma cross"), "SmaCross");
        assert_eq!(to_class_name("SMA Cross"), "SMACross");
        assert_eq!(to_class_name("1m breakout"), "S1mBreakout");
        assert_eq!(to_class_name("  "), "StrategyGraph");
        assert_eq!(to_ident("c-1"), "c_1");
        assert_eq!(to_ident("9lives"), "n_9lives");
    }
}
