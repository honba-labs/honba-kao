//! Client-side strategy + multi-signal evaluator (P7, doc §P7).
//!
//! Port of fe `core/simulator/engine/strategy-evaluator.ts`: computes every
//! graph indicator node over the real bars, evaluates condition/logic nodes
//! bar-by-bar, aggregates confluence ("more signals indicating buy/sell the
//! better") and yields order actions for the controller.
//!
//! Param reading is JSON-number based: fe coerces with `Number(…)`, kao
//! rejects non-numeric spellings to the same defaults the catalogue writes
//! anyway (matching the wasm indicators layer's strictness). Panel
//! arithmetic is carried in `f64` with fe's `NaN`-propagation semantics, so
//! warm-up gaps simply read as inactive conditions.
//!
//! Honesty notes: values shown in tooltips are the indicator series on the
//! real bars (rounded to 2dp like fe), never interpolated; an indicator
//! node whose kind the engine has no formula for (e.g. fe's unbundled
//! kinds) falls back to reading the bar close — fe does the same inside
//! `getSeriesValue`, and it is pinned here verbatim.

use std::collections::BTreeMap;

use serde_json::Value;

use super::indicators::{
    bollinger_series, ema_series, macd_from_emas, rsi_series, sma_series, BollingerSeries,
    MacdSeries,
};
use super::{stamp_str, Indication};
use crate::features::simulator::kernel::chart_data::ChartBar;
use crate::features::simulator::kernel::selection::{StrategyGraphEdge, StrategyGraphNode, StrategyGraphShape};

/// The engine's own order side — fe `'buy' | 'sell' | 'hold'`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TradeAction {
    /// `buy`.
    Buy,
    /// `sell`.
    Sell,
    /// `hold` — no order (fe default).
    Hold,
}

impl TradeAction {
    /// fe string member.
    pub fn name(self) -> &'static str {
        match self {
            Self::Buy => "buy",
            Self::Sell => "sell",
            Self::Hold => "hold",
        }
    }
}

/// One evaluated condition row — fe `SignalEvaluation`.
#[derive(Clone, Debug, PartialEq)]
pub struct SignalEvaluation {
    /// `${node.id}-${barIndex}` — fe's id.
    pub id: String,
    /// Source condition node.
    pub node_id: String,
    /// Presentation label (falls back to `Signal {node.id}`).
    pub label: String,
    /// The operator spelling (`crosses_above`, `lt`, …).
    pub condition: String,
    /// Whether it fired on this bar.
    pub active: bool,
    /// Bullish/bearish/neutral tag.
    pub indication: Indication,
    /// Confluence weight (fe hard-codes 1).
    pub weight: f64,
    /// `indicatorValue` — the left series value, 2dp-rounded, when finite.
    pub indicator_value: Option<f64>,
    /// `threshold` — the right series/threshold value, 2dp-rounded, when finite.
    pub threshold_value: Option<f64>,
    /// Human detail line, fe copy verbatim (`Crossed above (12.00 > 10.00)`).
    pub details: String,
}

/// `executedTrade` — the order the snapshot would place at `price`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExecutedTrade {
    /// `buy` or `sell`.
    pub side: TradeAction,
    /// Order quantity.
    pub quantity: f64,
    /// Fill price candidate (the bar close).
    pub price: f64,
}

/// `ConfluenceSnapshot` — one bar's full evaluation.
#[derive(Clone, Debug, PartialEq)]
pub struct ConfluenceSnapshot {
    /// The bar the snapshot belongs to.
    pub bar_index: usize,
    /// Bar timestamp (UTC seconds).
    pub timestamp: i64,
    /// IST stamp (see `engine::stamp_str`).
    pub time_str: String,
    /// The bar close.
    pub price: f64,
    /// Per-condition rows.
    pub signals: Vec<SignalEvaluation>,
    /// Sum of active bullish weights.
    pub bullish_score: f64,
    /// Sum of active bearish weights.
    pub bearish_score: f64,
    /// fe `Math.max(1, signals.length)`.
    pub total_signals: usize,
    /// `max(bullish, bearish) / total`.
    pub confluence_ratio: f64,
    /// Action decided by the confluence rule.
    pub action: TradeAction,
    /// fe reason copy (`Bullish Confluence (2/2 signals): SMA Cross`).
    pub reason: String,
    /// The order to execute, when an order node matches the action.
    pub executed_trade: Option<ExecutedTrade>,
}

/// `EvaluatedIndicators` — per-node series maps. `BTreeMap` so iteration is
/// deterministic; fe's Maps only serve `has`/`get` lookups.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EvaluatedIndicators {
    /// `sma` nodes.
    pub sma: BTreeMap<String, Vec<f64>>,
    /// `ema` nodes.
    pub ema: BTreeMap<String, Vec<f64>>,
    /// `rsi` nodes.
    pub rsi: BTreeMap<String, Vec<f64>>,
    /// `macd` nodes (three lines each).
    pub macd: BTreeMap<String, MacdSeries>,
    /// `bollinger` nodes (three bands each).
    pub bollinger: BTreeMap<String, BollingerSeries>,
}

/// First present numeric param among `keys`, else `default` — fe's
/// `Number(params.a ?? params.b ?? default)`.
fn num_param(params: &BTreeMap<String, Value>, keys: &[&str], default: f64) -> f64 {
    for key in keys {
        if let Some(number) = params
            .get(*key)
            .and_then(Value::as_f64)
            .filter(|n| n.is_finite())
        {
            return number;
        }
    }
    default
}

/// Period-ish param to a `usize` (finite and ≥ 1, truncated) else the
/// kind's default — fe's fractional/NaN periods are not reproduced.
fn period_param(params: &BTreeMap<String, Value>, keys: &[&str], default: usize) -> usize {
    let value = num_param(params, keys, default as f64);
    if value.is_finite() && value >= 1.0 {
        value as usize
    } else {
        default
    }
}

/// fe `computeGraphIndicators` — every indicator node kinded
/// `sma|ema|rsi|macd|bollinger` gets whole-series maps; unknown kinds are
/// skipped (their conditions then read the close, see `get_series_value`).
pub fn compute_graph_indicators(
    graph: &StrategyGraphShape,
    bars: &[ChartBar],
) -> EvaluatedIndicators {
    let mut result = EvaluatedIndicators::default();
    let closes: Vec<f64> = bars.iter().map(|bar| bar.close).collect();
    for node in &graph.nodes {
        let is_indicator = node.node_type == "indicator" || node.indicator.is_some();
        if !is_indicator {
            continue;
        }
        let kind = node.indicator.as_deref().unwrap_or("sma").to_lowercase();
        match kind.as_str() {
            "sma" => {
                let period = period_param(&node.params, &["period", "length"], 10);
                result
                    .sma
                    .insert(node.id.clone(), sma_series(&closes, period));
            }
            "ema" => {
                let period = period_param(&node.params, &["period", "length"], 10);
                result
                    .ema
                    .insert(node.id.clone(), ema_series(&closes, period));
            }
            "rsi" => {
                let period = period_param(&node.params, &["period", "length"], 14);
                result
                    .rsi
                    .insert(node.id.clone(), rsi_series(&closes, period));
            }
            "macd" => {
                let fast = period_param(&node.params, &["fast"], 12);
                let slow = period_param(&node.params, &["slow"], 26);
                let signal = period_param(&node.params, &["signal"], 9);
                let fast_ema = ema_series(&closes, fast);
                let slow_ema = ema_series(&closes, slow);
                result
                    .macd
                    .insert(node.id.clone(), macd_from_emas(&fast_ema, &slow_ema, slow, signal));
            }
            "bollinger" => {
                let length = period_param(&node.params, &["length", "period"], 20);
                let std = num_param(&node.params, &["std"], 2.0);
                result.bollinger.insert(
                    node.id.clone(),
                    bollinger_series(&closes, length, std),
                );
            }
            // fe skips unbundled kinds silently; the conditions referencing
            // them read the close via the fallback in get_series_value.
            _ => {}
        }
    }
    result
}

/// fe `getSeriesValue` — resolve a node/port's value at `idx`, falling back
/// to the bar close when the node has no series at all.
pub fn get_series_value(
    indicators: &EvaluatedIndicators,
    bars: &[ChartBar],
    node_id: &str,
    port: &str,
    idx: usize,
) -> f64 {
    if idx >= bars.len() {
        return f64::NAN;
    }
    if let Some(series) = indicators.sma.get(node_id) {
        return series[idx];
    }
    if let Some(series) = indicators.ema.get(node_id) {
        return series[idx];
    }
    if let Some(series) = indicators.rsi.get(node_id) {
        return series[idx];
    }
    if let Some(series) = indicators.macd.get(node_id) {
        return match port {
            "signal" => series.signal[idx],
            "histogram" => series.histogram[idx],
            _ => series.macd[idx],
        };
    }
    if let Some(series) = indicators.bollinger.get(node_id) {
        return match port {
            "upper" => series.upper[idx],
            "lower" => series.lower[idx],
            _ => series.middle[idx],
        };
    }
    bars[idx].close
}

/// fe `Number(v.toFixed(2))` display form; Rust's `{:.2}` already renders
/// non-finite values as `NaN`.
fn fmt2(value: f64) -> String {
    format!("{value:.2}")
}

/// fe `Number(x.toFixed(2))` rounding for the tooltip numbers.
fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// fe `evaluateBarSignals` — evaluate every condition/logic node on
/// `bar_index` and aggregate the confluence decision.
pub fn evaluate_bar_signals(
    graph: &StrategyGraphShape,
    bars: &[ChartBar],
    bar_index: usize,
    indicators: &EvaluatedIndicators,
) -> ConfluenceSnapshot {
    let current = &bars[bar_index];
    let has_prev = bar_index > 0;

    let mut signals: Vec<SignalEvaluation> = Vec::new();

    for node in graph.nodes.iter().filter(|n| {
        n.node_type == "condition" || n.node_type == "logic"
    }) {
        // fe: the first edge into `left`/`a`, and into `right`/`b`.
        let into = |port: &[&str]| -> Option<&StrategyGraphEdge> {
            graph
                .edges
                .iter()
                .find(|edge| {
                    edge.to_node == node.id && port.contains(&edge.to_port.as_str())
                })
        };
        let left_edge = into(&["left", "a"]);
        let right_edge = into(&["right", "b"]);

        let op = node
            .params
            .get("op")
            .and_then(Value::as_str)
            .unwrap_or("gt")
            .to_lowercase();
        let threshold = node
            .params
            .get("threshold")
            .and_then(Value::as_f64)
            .filter(|v| v.is_finite());

        let left_val = match left_edge {
            Some(edge) => {
                get_series_value(indicators, bars, &edge.from_node, &edge.from_port, bar_index)
            }
            None => current.close,
        };
        let prev_left_val = match (has_prev, left_edge) {
            (true, Some(edge)) => {
                get_series_value(indicators, bars, &edge.from_node, &edge.from_port, bar_index - 1)
            }
            _ => f64::NAN,
        };
        let right_val = match right_edge {
            Some(edge) => {
                get_series_value(indicators, bars, &edge.from_node, &edge.from_port, bar_index)
            }
            None => threshold.unwrap_or(f64::NAN),
        };
        let prev_right_val = match (has_prev, right_edge) {
            (true, Some(edge)) => {
                get_series_value(indicators, bars, &edge.from_node, &edge.from_port, bar_index - 1)
            }
            _ => threshold.unwrap_or(f64::NAN),
        };

        let mut active = false;
        let mut indication = Indication::Neutral;
        let mut details = String::new();

        if op == "crosses_above" {
            active = prev_left_val <= prev_right_val && left_val > right_val;
            indication = Indication::Bullish;
            details = format!("Crossed above ({} > {})", fmt2(left_val), fmt2(right_val));
        } else if op == "crosses_below" {
            active = prev_left_val >= prev_right_val && left_val < right_val;
            indication = Indication::Bearish;
            details = format!("Crossed below ({} < {})", fmt2(left_val), fmt2(right_val));
        } else if op == "lt" || op == "<" {
            active = left_val < right_val;
            // Mean-reversion reads: e.g. RSI < 30 is bullish oversold.
            indication = if threshold.is_some_and(|t| t <= 35.0) {
                Indication::Bullish
            } else {
                Indication::Bearish
            };
            details = format!("{} < {}", fmt2(left_val), fmt2(right_val));
        } else if op == "gt" || op == ">" {
            active = left_val > right_val;
            // Momentum reads: e.g. RSI > 70 is overbought (bearish).
            indication = if threshold.is_some_and(|t| t >= 70.0) {
                Indication::Bearish
            } else {
                Indication::Bullish
            };
            details = format!("{} > {}", fmt2(left_val), fmt2(right_val));
        } else if op == "and" {
            // Logic node.
            active = true;
            indication = Indication::Bullish;
            details = "All conditions met".to_string();
        }

        signals.push(SignalEvaluation {
            id: format!("{}-{}", node.id, bar_index),
            node_id: node.id.clone(),
            label: node
                .label
                .clone()
                .unwrap_or_else(|| format!("Signal {}", node.id)),
            condition: op,
            active,
            indication,
            weight: 1.0,
            indicator_value: left_val
                .is_finite()
                .then(|| round2(left_val)),
            threshold_value: right_val
                .is_finite()
                .then(|| round2(right_val)),
            details,
        });
    }

    // Confluence calculation.
    let bullish: Vec<&SignalEvaluation> = signals
        .iter()
        .filter(|s| s.active && s.indication == Indication::Bullish)
        .collect();
    let bearish: Vec<&SignalEvaluation> = signals
        .iter()
        .filter(|s| s.active && s.indication == Indication::Bearish)
        .collect();
    let bullish_score: f64 = bullish.iter().map(|s| s.weight).sum();
    let bearish_score: f64 = bearish.iter().map(|s| s.weight).sum();
    let total_signals = signals.len().max(1);

    // Strategy order nodes.
    let order_side = |node: &StrategyGraphNode, fallback: bool| -> String {
        let side = node
            .params
            .get("side")
            .and_then(Value::as_str)
            .map(str::to_lowercase);
        match (side, fallback) {
            (Some(side), _) => side,
            // fe buyNode: `String(side || 'buy')` — missing side reads buy.
            (None, true) => "buy".to_string(),
            // fe sellNode: `String(side)` — missing side reads "undefined".
            (None, false) => "undefined".to_string(),
        }
    };
    let buy_node = graph
        .nodes
        .iter()
        .find(|n| n.node_type == "order" && order_side(n, true) == "buy");
    let sell_node = graph
        .nodes
        .iter()
        .find(|n| n.node_type == "order" && order_side(n, false) == "sell");

    let mut action = TradeAction::Hold;
    let mut reason = "Holding".to_string();

    // fe rule: a single condition is enough; with more, a majority-ish
    // count (`Math.min(bullish.length, max(1, ceil(signals/2)))`).
    let required_bullish = bullish.len().min(
        (signals.len() as f64 / 2.0).ceil() as usize
    );
    let required_bullish = required_bullish.max(1);

    if bullish_score >= required_bullish as f64 && bullish_score > bearish_score {
        action = TradeAction::Buy;
        let labels: Vec<&str> = bullish.iter().map(|s| s.label.as_str()).collect();
        reason = format!(
            // fe interpolates the scores bare (`${bullishScore}`), not
            // through toFixed — weight sums render as integers.
            "Bullish Confluence ({}/{total_signals} signals): {}",
            bullish_score,
            labels.join(" + ")
        );
    } else if bearish_score >= 1.0 && bearish_score > bullish_score {
        action = TradeAction::Sell;
        let labels: Vec<&str> = bearish.iter().map(|s| s.label.as_str()).collect();
        reason = format!(
            "Bearish Confluence ({}/{total_signals} signals): {}",
            bearish_score,
            labels.join(" + ")
        );
    }

    let confluence_ratio = if total_signals > 0 {
        bullish_score.max(bearish_score) / total_signals as f64
    } else {
        0.0
    };

    let executed_trade = if action == TradeAction::Buy {
        buy_node.map(|node| ExecutedTrade {
            side: TradeAction::Buy,
            quantity: num_param(&node.params, &["quantity"], super::DEFAULT_QUANTITY),
            price: current.close,
        })
    } else if action == TradeAction::Sell {
        sell_node.map(|node| ExecutedTrade {
            side: TradeAction::Sell,
            quantity: num_param(&node.params, &["quantity"], super::DEFAULT_QUANTITY),
            price: current.close,
        })
    } else {
        None
    };

    ConfluenceSnapshot {
        bar_index,
        timestamp: current.time,
        time_str: stamp_str(current.time),
        price: current.close,
        signals,
        bullish_score,
        bearish_score,
        total_signals,
        confluence_ratio,
        action,
        reason,
        executed_trade,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::features::simulator::kernel::selection::{builtin_template, StrategyGraphEdge, StrategyGraphNode};

    /// fe `engine.test.ts` `mockBars` (same generator, f64 exact).
    fn mock_bars(count: usize) -> Vec<ChartBar> {
        let mut bars = Vec::with_capacity(count);
        let mut price = 100.0;
        let base_time = 1_700_000_000;
        for i in 0..count {
            let delta = (i as f64 / 3.0).sin() * 2.0 + if i.is_multiple_of(2) { 1.0 } else { -0.5 };
            let open = price;
            let close = price + delta;
            let high = open.max(close) + 1.0;
            let low = open.min(close) - 1.0;
            price = close;
            bars.push(ChartBar {
                time: base_time + (i as i64) * 300,
                open,
                high,
                low,
                close,
                volume: 1000.0 + (i as f64) * 10.0,
            });
        }
        bars
    }

    fn node(id: &str, node_type: &str) -> StrategyGraphNode {
        StrategyGraphNode {
            id: id.to_string(),
            node_type: node_type.to_string(),
            ..StrategyGraphNode::default()
        }
    }

    fn param(id: &str, node_type: &str, key: &str, value: Value) -> StrategyGraphNode {
        let mut n = node(id, node_type);
        n.params.insert(key.to_string(), value);
        n
    }

    fn indicator(id: &str, kind: &str, period: f64) -> StrategyGraphNode {
        let mut n = node(id, "indicator");
        n.indicator = Some(kind.to_string());
        n.params.insert("period".to_string(), period.into());
        n
    }

    fn edge(id: &str, from: &str, from_port: &str, to: &str, to_port: &str) -> StrategyGraphEdge {
        StrategyGraphEdge {
            id: id.to_string(),
            from_node: from.to_string(),
            from_port: from_port.to_string(),
            to_node: to.to_string(),
            to_port: to_port.to_string(),
        }
    }

    /// An empty graph shell for tests that attach nodes/edges inline.
    fn bare_graph() -> StrategyGraphShape {
        StrategyGraphShape {
            name: String::new(),
            universe_named: String::new(),
            universe_explicit: Vec::new(),
            driving_interval: 5,
            driving_aggregation: String::new(),
            warmup_bars: 0,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// fe `engine.test.ts` `mockMultiSignalGraph`.
    fn multi_signal_graph() -> StrategyGraphShape {
        StrategyGraphShape {
            name: "multi_signal_strategy".to_string(),
            universe_named: String::new(),
            universe_explicit: vec![("TCS".to_string(), "NSE".to_string())],
            driving_interval: 5,
            driving_aggregation: "minute".to_string(),
            warmup_bars: 10,
            nodes: vec![
                node("n1", "universe"),
                indicator("f", "sma", 3.0),
                indicator("s", "sma", 8.0),
                indicator("r", "rsi", 14.0),
                {
                    let mut c = node("c1", "condition");
                    c.label = Some("SMA Cross".to_string());
                    c.params.insert("op".to_string(), "crosses_above".into());
                    c
                },
                {
                    let mut c = node("c2", "condition");
                    c.label = Some("RSI Oversold".to_string());
                    c.params.insert("op".to_string(), "lt".into());
                    c.params.insert("threshold".to_string(), 40.into());
                    c
                },
                {
                    let mut o = node("o_buy", "order");
                    o.params.insert("side".to_string(), "buy".into());
                    o.params.insert("quantity".to_string(), 10.into());
                    o
                },
                {
                    let mut o = node("o_sell", "order");
                    o.params.insert("side".to_string(), "sell".into());
                    o.params.insert("quantity".to_string(), 10.into());
                    o
                },
            ],
            edges: vec![
                edge("e1", "n1", "bars", "f", "input"),
                edge("e2", "n1", "bars", "s", "input"),
                edge("e3", "n1", "bars", "r", "input"),
                edge("e4", "f", "value", "c1", "left"),
                edge("e5", "s", "value", "c1", "right"),
                edge("e6", "r", "value", "c2", "left"),
            ],
        }
    }

    #[test]
    fn the_multi_signal_graph_computes_each_kind() {
        let bars = mock_bars(25);
        let graph = multi_signal_graph();
        let indicators = compute_graph_indicators(&graph, &bars);
        assert!(indicators.sma.contains_key("f"));
        assert!(indicators.sma.contains_key("s"));
        assert!(indicators.rsi.contains_key("r"));
        assert_eq!(indicators.sma["f"].len(), 25);
    }

    #[test]
    fn confluence_evaluates_two_signal_rows_at_the_bar_price() {
        let bars = mock_bars(25);
        let graph = multi_signal_graph();
        let indicators = compute_graph_indicators(&graph, &bars);
        let confluence = evaluate_bar_signals(&graph, &bars, 15, &indicators);
        assert!(confluence.total_signals >= 2);
        assert_eq!(confluence.signals.len(), 2);
        assert_eq!(confluence.price, bars[15].close);
        // IDs and labels follow fe's shapes.
        assert_eq!(confluence.signals[0].id, "c1-15");
        assert_eq!(confluence.signals[0].label, "SMA Cross");
        assert_eq!(confluence.signals[0].condition, "crosses_above");
        // Rows 15 both read finite series values (warm-up has passed),
        // 2dp-rounded like fe's `Number(x.toFixed(2))`.
        for signal in &confluence.signals {
            assert!(signal.indicator_value.is_some());
        }
    }

    #[test]
    fn an_lt_40_condition_carries_the_bearish_tag_and_stamps_details() {
        // RSI condition lt 40: fe reads threshold > 35 as bearish (only
        // ≤ 35 flips to bullish oversold); details carry the comparison.
        let bars = mock_bars(30);
        let mut graph = bare_graph();
        graph.nodes = vec![
            indicator("r", "rsi", 5.0),
            {
                let mut c = param("c40", "condition", "op", "lt".into());
                c.params.insert("threshold".into(), 40.into());
                c
            },
        ];
        graph.edges = vec![edge("e", "r", "value", "c40", "left")];
        let indicators = compute_graph_indicators(&graph, &bars);
        let confluence = evaluate_bar_signals(&graph, &bars, 20, &indicators);
        let row = &confluence.signals[0];
        assert!(row.details.ends_with(" < 40.00"));
        if row.active {
            assert_eq!(row.indication, Indication::Bearish); // 40 > 35
        }
    }

    #[test]
    fn a_gt_70_condition_reads_bearish_when_it_fires() {
        // All-up closes drive RSI(3) to 100 — the gt 70 row always fires,
        // and fe tags threshold ≥ 70 bearish (overbought).
        let bars: Vec<ChartBar> = (1..=20)
            .map(|i| {
                let close = i as f64 * 2.0;
                ChartBar {
                    time: 1_700_000_000 + (i as i64 - 1) * 300,
                    open: close,
                    high: close,
                    low: close,
                    close,
                    volume: 0.0,
                }
            })
            .collect();
        let mut graph = bare_graph();
        graph.nodes = vec![
            indicator("r", "rsi", 3.0),
            {
                let mut c = param("c", "condition", "op", "gt".into());
                c.params.insert("threshold".into(), 70.into());
                c
            },
        ];
        graph.edges = vec![edge("e", "r", "value", "c", "left")];
        let indicators = compute_graph_indicators(&graph, &bars);
        let confluence = evaluate_bar_signals(&graph, &bars, 10, &indicators);
        let row = &confluence.signals[0];
        assert!(row.active);
        assert_eq!(row.indication, Indication::Bearish);
        assert!(row.details.contains(" > 70.00"));
    }

    #[test]
    fn crosses_above_requires_a_real_cross() {
        let mut graph = bare_graph();
        graph.nodes = vec![
            node("n1", "universe"),
            indicator("f", "sma", 1.0),
            indicator("s", "sma", 2.0),
            param("c", "condition", "op", "crosses_above".into()),
        ];
        graph.edges = vec![
            edge("e1", "n1", "bars", "f", "input"),
            edge("e2", "n1", "bars", "s", "input"),
            edge("e3", "f", "value", "c", "left"),
            edge("e4", "s", "value", "c", "right"),
        ];
        // closes 10, 9, 8, 12: sma1 = close, sma2 slides; at i=3:
        // prev_left(8) <= prev_right(8.5) and left(12) > right(10).
        let bars: Vec<ChartBar> = [10.0, 9.0, 8.0, 12.0]
            .iter()
            .enumerate()
            .map(|(i, &close)| ChartBar {
                time: 1_700_000_000 + (i as i64) * 300,
                open: close,
                high: close,
                low: close,
                close,
                volume: 0.0,
            })
            .collect();
        let indicators = compute_graph_indicators(&graph, &bars);
        let confluence = evaluate_bar_signals(&graph, &bars, 3, &indicators);
        let row = &confluence.signals[0];
        assert!(row.active);
        assert_eq!(row.indication, Indication::Bullish);
        assert_eq!(row.details, "Crossed above (12.00 > 10.00)");
    }

    #[test]
    fn an_unknown_indicator_kind_falls_back_to_the_close_like_fe() {
        let mut graph = bare_graph();
        graph.nodes = vec![
            node("n1", "universe"),
            {
                let mut n = node("w", "indicator");
                n.indicator = Some("wma".into());
                n
            },
            {
                let mut c = param("c", "condition", "op", "gt".into());
                c.params.insert("threshold".into(), 50.into());
                c
            },
        ];
        graph.edges = vec![
            edge("e1", "n1", "bars", "w", "input"),
            edge("e2", "w", "value", "c", "left"),
        ];
        let bars = mock_bars(8);
        let indicators = compute_graph_indicators(&graph, &bars);
        assert!(indicators.sma.is_empty() && indicators.ema.is_empty());
        let confluence = evaluate_bar_signals(&graph, &bars, 5, &indicators);
        let row = &confluence.signals[0];
        // WMA has no series → the left value is the bar close (fe fallback).
        assert_eq!(row.indicator_value, Some(round2(bars[5].close)));
        assert_eq!(row.active, bars[5].close > 50.0);
    }

    #[test]
    fn and_logic_nodes_fire_bullish() {
        let mut graph = bare_graph();
        graph.nodes = vec![{
            let mut n = node("l", "logic");
            n.params.insert("op".into(), "and".into());
            n
        }];
        let indicators = EvaluatedIndicators::default();
        let confluence = evaluate_bar_signals(&graph, &mock_bars(4), 2, &indicators);
        let row = &confluence.signals[0];
        assert!(row.active);
        assert_eq!(row.indication, Indication::Bullish);
        assert_eq!(row.details, "All conditions met");
    }

    #[test]
    fn a_bullish_majority_places_the_buy_order() {
        let mut graph = bare_graph();
        graph.nodes = vec![
            indicator("f", "sma", 1.0),
            {
                let mut c = param("c", "condition", "op", "gt".into());
                c.params.insert("threshold".into(), 50.into());
                c
            },
            {
                let mut o = node("o_buy", "order");
                o.params.insert("side".into(), "buy".into());
                o.params.insert("quantity".into(), 25.into());
                o
            },
        ];
        graph.edges = vec![edge("e1", "f", "value", "c", "left")];
        let bars = mock_bars(6);
        let indicators = compute_graph_indicators(&graph, &bars);
        // Row 5 is up (sine + close > 50 most likely)… find a buy bar.
        for i in 2..bars.len() {
            let confluence = evaluate_bar_signals(&graph, &bars, i, &indicators);
            if confluence.action == TradeAction::Buy {
                let trade = confluence.executed_trade.expect("buy must place an order");
                assert_eq!(trade.side, TradeAction::Buy);
                assert_eq!(trade.quantity, 25.0);
                assert_eq!(trade.price, bars[i].close);
                assert!(confluence.reason.starts_with("Bullish Confluence (1/1 signals):"));
                assert!(confluence.confluence_ratio > 0.0);
                return;
            } else {
                assert!(confluence.executed_trade.is_none());
                assert_eq!(confluence.action, TradeAction::Hold);
                assert_eq!(confluence.reason, "Holding");
            }
        }
        panic!("fixture should produce at least one buy bar");
    }

    #[test]
    fn a_bearish_signal_places_the_sell_order() {
        let mut graph = bare_graph();
        graph.nodes = vec![
            indicator("f", "sma", 1.0),
            {
                let mut c = param("c", "condition", "op", "lt".into());
                c.params.insert("threshold".into(), 200.into());
                c
            },
            {
                let mut o = node("o_sell", "order");
                o.params.insert("side".into(), "sell".into());
                o
            },
        ];
        graph.edges = vec![edge("e1", "f", "value", "c", "left")];
        let bars = mock_bars(6);
        let indicators = compute_graph_indicators(&graph, &bars);
        // Every close sits ≈ 100, so `close < 200` always fires; the
        // threshold 200 > 35 reads bearish in fe (not oversold), so the
        // sell branch fires with the default quantity 10 (fe `?? 10`).
        let confluence = evaluate_bar_signals(&graph, &bars, 5, &indicators);
        assert_eq!(confluence.action, TradeAction::Sell); // bar 5 close ≈ 103 < 200
        let trade = confluence.executed_trade.expect("sell must place an order");
        assert_eq!(trade.side, TradeAction::Sell);
        assert_eq!(trade.quantity, 10.0);
        assert!(confluence
            .reason
            .starts_with("Bearish Confluence (1/1 signals):"));
    }

    #[test]
    fn the_snapshot_stamps_ist_like_the_tick_surface() {
        let bars = mock_bars(25);
        let (graph, _) = builtin_template();
        let indicators = compute_graph_indicators(&graph, &bars);
        let confluence = evaluate_bar_signals(&graph, &bars, 15, &indicators);
        assert_eq!(confluence.time_str, super::super::stamp_str(bars[15].time));
        assert_eq!(confluence.timestamp, bars[15].time);
    }
}
