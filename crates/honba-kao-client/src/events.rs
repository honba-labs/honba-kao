use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DomainEvent {
    TickerUpdate(crate::dto::market::Ticker),
    OrderBookUpdate(crate::dto::market::Level2Book),
    TradePrint(crate::dto::market::TradeTick),
    OrderStateChange(crate::dto::order::OrderResponse),
    LiquidationAlert(crate::dto::risk::LiquidationWarning),
    AgentThought(crate::stream::AgentStep),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EventEnvelope {
    pub event_id: String,
    pub timestamp: i64,
    pub payload: DomainEvent,
}
