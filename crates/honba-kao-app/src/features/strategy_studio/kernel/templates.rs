//! The studio's starting strategies (HBD-2026-DES-006 P2 acceptance).
//!
//! The Leptos port of honba-fe algodesigner `strategy-templates.ts`: the
//! six `STRATEGY_TEMPLATES` graphs, node-for-node and edge-for-edge
//! (including fe's parameter spellings such as the Bollinger template's
//! `length`/`std`, which the validator leaves alone because they are not
//! catalogue names). Pure data — no DOM, no signals.
//!
//! Beyond fe: [`blank_graph`] (the "+ New strategy" starting point) and
//! [`library_template`], which maps the demo library rows onto the template
//! graph each one is authored from, so the library's `Open` lands on a real
//! graph instead of a placeholder.

use std::collections::BTreeMap;

use crate::features::simulator::kernel::selection::{
    StrategyGraphEdge, StrategyGraphNode, StrategyGraphShape,
};

/// One starting strategy (fe `StrategyTemplate`).
#[derive(Clone, Debug, PartialEq)]
pub struct StrategyTemplate {
    /// Stable template name (fe `name`, also the graph's `name`).
    pub name: &'static str,
    /// Display label.
    pub label: &'static str,
    /// fe category tag (`trend | mean_reversion | breakout | volatility`).
    pub category: &'static str,
    /// One-line description.
    pub description: &'static str,
    /// The graph itself.
    pub graph: StrategyGraphShape,
}

fn node(
    id: &str,
    node_type: &str,
    label: &str,
    indicator: Option<&str>,
    params: &[(&str, serde_json::Value)],
) -> StrategyGraphNode {
    let params: BTreeMap<String, serde_json::Value> = params
        .iter()
        .map(|(name, value)| ((*name).to_string(), value.clone()))
        .collect();
    StrategyGraphNode {
        id: id.to_string(),
        node_type: node_type.to_string(),
        label: Some(label.to_string()),
        params,
        indicator: indicator.map(str::to_string),
    }
}

fn edge(id: &str, from_node: &str, from_port: &str, to_node: &str, to_port: &str) -> StrategyGraphEdge {
    StrategyGraphEdge {
        id: id.to_string(),
        from_node: from_node.to_string(),
        from_port: from_port.to_string(),
        to_node: to_node.to_string(),
        to_port: to_port.to_string(),
    }
}

fn shape(
    name: &str,
    symbol: &str,
    interval: u32,
    warmup_bars: u32,
    nodes: Vec<StrategyGraphNode>,
    edges: Vec<StrategyGraphEdge>,
) -> StrategyGraphShape {
    StrategyGraphShape {
        name: name.to_string(),
        universe_named: String::new(),
        universe_explicit: vec![(symbol.to_string(), "NSE".to_string())],
        driving_interval: interval,
        driving_aggregation: "minute".to_string(),
        warmup_bars,
        nodes,
        edges,
    }
}

fn order(id: &str, label: &str, side: &str) -> StrategyGraphNode {
    node(id, "order", label, None, &[("side", side.into()), ("quantity", 1.into())])
}

fn sma_cross() -> StrategyGraphShape {
    shape(
        "sma_cross",
        "TCS",
        1,
        2,
        vec![
            node("n1", "universe", "Universe", None, &[]),
            node("f", "indicator", "Fast SMA", Some("sma"), &[("period", 5.into())]),
            node("s", "indicator", "Slow SMA", Some("sma"), &[("period", 20.into())]),
            node("c", "condition", "Cross", None, &[("op", "crosses_above".into())]),
            order("o", "Entry Long", "buy"),
        ],
        vec![
            edge("e1", "n1", "bars", "f", "input"),
            edge("e2", "n1", "bars", "s", "input"),
            edge("e3", "f", "value", "c", "left"),
            edge("e4", "s", "value", "c", "right"),
            edge("e5", "c", "signal", "o", "signal"),
        ],
    )
}

fn ema_trend() -> StrategyGraphShape {
    shape(
        "ema_trend",
        "RELIANCE",
        5,
        5,
        vec![
            node("n1", "universe", "Universe", None, &[]),
            node("fast", "indicator", "Fast EMA", Some("ema"), &[("period", 9.into())]),
            node("slow", "indicator", "Slow EMA", Some("ema"), &[("period", 21.into())]),
            node("c", "condition", "Trend Up", None, &[("op", "gt".into())]),
            order("buy", "Buy", "buy"),
            order("sell", "Sell", "sell"),
            node("c2", "condition", "Trend Down", None, &[("op", "lt".into())]),
        ],
        vec![
            edge("e1", "n1", "bars", "fast", "input"),
            edge("e2", "n1", "bars", "slow", "input"),
            edge("e3", "fast", "value", "c", "left"),
            edge("e4", "slow", "value", "c", "right"),
            edge("e5", "c", "signal", "buy", "signal"),
            edge("e6", "fast", "value", "c2", "left"),
            edge("e7", "slow", "value", "c2", "right"),
            edge("e8", "c2", "signal", "sell", "signal"),
        ],
    )
}

fn rsi_mr() -> StrategyGraphShape {
    shape(
        "rsi_mr",
        "INFY",
        15,
        14,
        vec![
            node("n1", "universe", "Universe", None, &[]),
            node("rsi", "indicator", "RSI 14", Some("rsi"), &[("period", 14.into())]),
            node(
                "oversold",
                "condition",
                "Oversold (< 30)",
                None,
                &[("op", "lt".into()), ("threshold", 30.into())],
            ),
            node(
                "overbought",
                "condition",
                "Overbought (> 70)",
                None,
                &[("op", "gt".into()), ("threshold", 70.into())],
            ),
            order("buy", "Buy (Oversold)", "buy"),
            order("sell", "Sell (Overbought)", "sell"),
        ],
        vec![
            edge("e1", "n1", "bars", "rsi", "input"),
            edge("e2", "rsi", "value", "oversold", "left"),
            edge("e3", "rsi", "value", "overbought", "left"),
            edge("e4", "oversold", "signal", "buy", "signal"),
            edge("e5", "overbought", "signal", "sell", "signal"),
        ],
    )
}

fn macd_rsi() -> StrategyGraphShape {
    shape(
        "macd_rsi",
        "HDFCBANK",
        30,
        26,
        vec![
            node("n1", "universe", "Universe", None, &[]),
            node(
                "macd",
                "indicator",
                "MACD",
                Some("macd"),
                &[("fast", 12.into()), ("slow", 26.into()), ("signal", 9.into())],
            ),
            node("rsi", "indicator", "RSI 14", Some("rsi"), &[("period", 14.into())]),
            node("macd_cross", "condition", "MACD Cross Up", None, &[("op", "crosses_above".into())]),
            node(
                "rsi_filter",
                "condition",
                "RSI < 70",
                None,
                &[("op", "lt".into()), ("threshold", 70.into())],
            ),
            node("and", "logic", "AND", None, &[("op", "and".into())]),
            order("buy", "Buy", "buy"),
        ],
        vec![
            edge("e1", "n1", "bars", "macd", "input"),
            edge("e2", "n1", "bars", "rsi", "input"),
            edge("e3", "macd", "macd", "macd_cross", "left"),
            edge("e4", "macd", "signal", "macd_cross", "right"),
            edge("e5", "rsi", "value", "rsi_filter", "left"),
            edge("e6", "macd_cross", "signal", "and", "a"),
            edge("e7", "rsi_filter", "signal", "and", "b"),
            edge("e8", "and", "signal", "buy", "signal"),
        ],
    )
}

fn bb_breakout() -> StrategyGraphShape {
    shape(
        "bb_breakout",
        "ICICIBANK",
        15,
        20,
        vec![
            node("n1", "universe", "Universe", None, &[]),
            node(
                "bb",
                "indicator",
                "Bollinger Bands",
                Some("bollinger"),
                &[("length", 20.into()), ("std", 2.0.into())],
            ),
            node("upper_break", "condition", "Break Upper", None, &[("op", "crosses_above".into())]),
            node("lower_break", "condition", "Break Lower", None, &[("op", "crosses_below".into())]),
            order("buy", "Buy Breakout", "buy"),
            order("sell", "Sell Breakdown", "sell"),
        ],
        vec![
            edge("e1", "n1", "bars", "bb", "input"),
            edge("e2", "bb", "upper", "upper_break", "left"),
            edge("e3", "n1", "bars", "upper_break", "right"),
            edge("e4", "bb", "lower", "lower_break", "right"),
            edge("e5", "n1", "bars", "lower_break", "left"),
            edge("e6", "upper_break", "signal", "buy", "signal"),
            edge("e7", "lower_break", "signal", "sell", "signal"),
        ],
    )
}

fn atr_trailing() -> StrategyGraphShape {
    shape(
        "atr_trailing",
        "BAJFINANCE",
        60,
        14,
        vec![
            node("n1", "universe", "Universe", None, &[]),
            node("ema", "indicator", "EMA 200", Some("ema"), &[("period", 200.into())]),
            node("atr", "indicator", "ATR 14", Some("atr"), &[("period", 14.into())]),
            node("trend_up", "condition", "Price > EMA", None, &[("op", "gt".into())]),
            node("trailing", "risk", "ATR Trailing Stop", None, &[("max_drawdown", 0.03.into())]),
            order("buy", "Buy", "buy"),
            node(
                "sz",
                "sizing",
                "Vol Sizing",
                None,
                &[("mode", "percent_equity".into()), ("value", 10.into())],
            ),
        ],
        vec![
            edge("e1", "n1", "bars", "ema", "input"),
            edge("e2", "n1", "bars", "atr", "input"),
            edge("e3", "n1", "bars", "trend_up", "left"),
            edge("e4", "ema", "value", "trend_up", "right"),
            edge("e5", "trend_up", "signal", "trailing", "signal"),
            edge("e6", "trailing", "signal", "sz", "signal"),
            edge("e7", "sz", "size", "buy", "size"),
            edge("e8", "trailing", "signal", "buy", "signal"),
        ],
    )
}

/// fe's `STRATEGY_TEMPLATES`, in fe's order.
pub fn strategy_templates() -> Vec<StrategyTemplate> {
    vec![
        StrategyTemplate {
            name: "sma_cross",
            label: "SMA Cross (Classic)",
            category: "trend",
            description: "Classic dual-SMA crossover. Fast crosses above slow = buy. Good for trending markets.",
            graph: sma_cross(),
        },
        StrategyTemplate {
            name: "ema_trend",
            label: "EMA Trend Follow",
            category: "trend",
            description: "Dual-EMA trend following with both long and short signals. Faster than SMA.",
            graph: ema_trend(),
        },
        StrategyTemplate {
            name: "rsi_mr",
            label: "RSI Mean Reversion",
            category: "mean_reversion",
            description: "RSI-based mean reversion. Buys oversold (<30), sells overbought (>70). Best in ranging markets.",
            graph: rsi_mr(),
        },
        StrategyTemplate {
            name: "macd_rsi",
            label: "MACD + RSI Filter",
            category: "trend",
            description: "MACD crossover confirmed by RSI filter. Reduces whipsaws in choppy conditions.",
            graph: macd_rsi(),
        },
        StrategyTemplate {
            name: "bb_breakout",
            label: "Bollinger Band Breakout",
            category: "breakout",
            description: "Price breaking Bollinger bands. Captures volatility expansion moves.",
            graph: bb_breakout(),
        },
        StrategyTemplate {
            name: "atr_trailing",
            label: "ATR Trailing Stop Trend",
            category: "volatility",
            description: "Long-term EMA trend filter with ATR-based trailing stop and volatility position sizing.",
            graph: atr_trailing(),
        },
    ]
}

/// Looks one template up by its `name`.
pub fn template_by_name(name: &str) -> Option<StrategyTemplate> {
    strategy_templates().into_iter().find(|t| t.name == name)
}

/// The "+ New strategy" starting point: a named strategy with a universe
/// node and nothing else — the palette adds the rest. Validation reports
/// `no_order` until an order node lands, which is the honest state.
pub fn blank_graph() -> StrategyGraphShape {
    shape(
        "untitled_strategy",
        "TCS",
        5,
        2,
        vec![node("n1", "universe", "Universe", None, &[])],
        Vec::new(),
    )
}

/// The template graph a demo library row is authored from, by library id.
/// `None` means the row has no graph yet (rendered as such, never faked).
pub fn library_template(library_id: &str) -> Option<&'static str> {
    match library_id {
        "trend-ema" => Some("ema_trend"),
        "bollinger-squeeze" => Some("bb_breakout"),
        _ => None,
    }
}
