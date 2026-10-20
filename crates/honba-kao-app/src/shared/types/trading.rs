//! Trading-domain types. Mirrors `web/src/types/trading.ts`.

/// A configured AI trading agent.
#[derive(Clone, Debug, PartialEq)]
pub struct TraderInfo {
    pub trader_id: String,
    pub trader_name: String,
    pub ai_model: String,
    pub exchange: String,
    pub is_running: bool,
    pub pnl: f64,
    pub win_rate: f64,
}

impl TraderInfo {
    /// Slug used in dashboard deep links (`name-idprefix`).
    pub fn slug(&self) -> String {
        let prefix: String = self.trader_id.chars().take(4).collect();
        format!("{}-{}", self.trader_name, prefix)
    }
}

/// An open position.
#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    pub symbol: String,
    pub is_buy: bool,
    pub qty: i64,
    pub avg_price: f64,
    pub mark_price: f64,
    pub unrealized_pnl: f64,
}

/// A working or historical order.
#[derive(Clone, Debug, PartialEq)]
pub struct Order {
    pub order_id: String,
    pub symbol: String,
    pub order_type: String,
    pub is_buy: bool,
    pub qty: i64,
    pub filled_qty: i64,
    pub price: f64,
    pub status: String,
}

/// A single risk-gate decision emitted by the engine.
#[derive(Clone, Debug, PartialEq)]
pub struct DecisionRecord {
    pub time: String,
    pub trader: String,
    pub symbol: String,
    pub action: String,
    pub approved: bool,
    pub reason: String,
}

/// Account-level snapshot for a trader.
#[derive(Clone, Debug, PartialEq)]
pub struct AccountInfo {
    pub total_value: f64,
    pub available_cash: f64,
    pub day_pnl: f64,
    pub buying_power: f64,
}

/// Live market status for a trader's engine.
#[derive(Clone, Debug, PartialEq)]
pub struct SystemStatus {
    pub trader_id: String,
    pub running: bool,
    pub last_update: String,
    pub cpu_pct: f64,
    pub latency_ms: u32,
}

/// A tradeable instrument quote.
#[derive(Clone, Debug, PartialEq)]
pub struct MarketQuote {
    pub symbol: String,
    pub name: String,
    pub last: f64,
    pub change_pct: f64,
}
