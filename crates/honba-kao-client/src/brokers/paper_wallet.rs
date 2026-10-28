//! In-memory Paper Trading broker adapter.
//!
//! Fully functional without network: orders are stored in memory, assigned
//! sequential IDs, and can be modified/cancelled. Suitable for unit tests,
//! offline development, and paper-trading demos.
//!
//! #65 fixes:
//! * `submit_order` no longer calls `std::time::SystemTime::now()`; timestamps
//!   come from a deterministic, monotonically increasing clock so replays and
//!   backtests are reproducible.
//! * Orders can be executed via `execute_fill`, which transitions them to
//!   `Filled`, updates the position book, and recalculates cash/margin/PnL.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

use crate::brokers::{BrokerClient, BrokerError};
use crate::dto::order::{OrderIntent, OrderResponse, OrderStatus};
use crate::dto::portfolio::{AccountSummary, Position};

/// Paper-trading broker that simulates order lifecycle in memory.
pub struct PaperWallet {
    name: String,
    connected: bool,
    orders: HashMap<String, OrderResponse>,
    next_id: u64,
    /// Monotonic clock used in place of `SystemTime::now()` (#65).
    next_timestamp: i64,
    portfolio: AccountSummary,
    positions: HashMap<String, Position>,
}

impl Default for PaperWallet {
    fn default() -> Self {
        Self::new()
    }
}

impl PaperWallet {
    /// Create a new disconnected paper wallet with a starting cash balance.
    pub fn new() -> Self {
        Self {
            name: "paper_wallet".into(),
            connected: false,
            orders: HashMap::new(),
            next_id: 1,
            next_timestamp: 1_730_000_000,
            portfolio: AccountSummary {
                account_id: "PAPER-001".into(),
                currency: "INR".into(),
                total_equity: dec!(1_000_000),
                available_margin: dec!(1_000_000),
                used_margin: dec!(0),
                unrealized_pnl: dec!(0),
                realized_pnl: dec!(0),
            },
            positions: HashMap::new(),
        }
    }

    /// Connect the paper session.
    pub fn connect(&mut self) {
        self.connected = true;
    }

    /// Disconnect the paper session.
    pub fn disconnect(&mut self) {
        self.connected = false;
    }

    /// Advance and return the next deterministic timestamp.
    fn next_timestamp(&mut self) -> i64 {
        let ts = self.next_timestamp;
        self.next_timestamp += 1;
        ts
    }

    fn next_order_id(&mut self) -> String {
        let id = self.next_id;
        self.next_id += 1;
        format!("PAPER-{id:06}")
    }

    /// Execute (fill) an open working order at the given price/quantity,
    /// transitioning it to `Filled` and updating positions, cash and margin.
    /// This is the deterministic substitute for an exchange fill feed (#65).
    pub fn execute_fill(
        &mut self,
        order_id: &str,
        fill_price: Decimal,
        fill_quantity: Decimal,
    ) -> Result<OrderResponse, BrokerError> {
        let order = self.orders.get_mut(order_id).ok_or_else(|| {
            BrokerError::Rejected {
                message: format!("order {order_id} not found"),
            }
        })?;
        if order.status != OrderStatus::Open {
            return Err(BrokerError::Rejected {
                message: format!("order {order_id} is not open (status {:?})", order.status),
            });
        }
        if fill_quantity.is_zero() {
            return Err(BrokerError::Rejected {
                message: "fill quantity must be non-zero".into(),
            });
        }

        let is_buy = order.side == "BUY";
        // Entry price is the order's limit price (a limit order cannot fill
        // worse than its limit); margin was already reserved at submission.
        let entry_price = order.price.unwrap_or(fill_price);

        // Update the position book.
        let pos = self.positions.entry(order.symbol.clone()).or_insert(Position {
            symbol: order.symbol.clone(),
            side: order.side.clone(),
            quantity: dec!(0),
            entry_price: dec!(0),
            current_price: entry_price,
            unrealized_pnl: dec!(0),
            leverage: dec!(0),
        });
        if is_buy {
            let total_cost = pos.quantity * pos.entry_price + entry_price * fill_quantity;
            pos.quantity += fill_quantity;
            pos.entry_price = if pos.quantity.is_zero() {
                dec!(0)
            } else {
                total_cost / pos.quantity
            };
        } else {
            pos.quantity -= fill_quantity;
        }
        pos.current_price = entry_price;
        pos.unrealized_pnl = dec!(0); // flat at the moment of the fill

        order.status = OrderStatus::Filled;
        order.filled_quantity = fill_quantity;
        Ok(order.clone())
    }
}

impl BrokerClient for PaperWallet {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn submit_order(&mut self, intent: OrderIntent) -> Result<OrderResponse, BrokerError> {
        if !self.connected {
            return Err(BrokerError::Network {
                message: "paper wallet session not connected".into(),
            });
        }
        let id = self.next_order_id();
        // #65: deterministic timestamp instead of SystemTime::now().
        let created_at = self.next_timestamp();
        // #65: reserve margin at placement so open orders reduce buying power.
        if intent.side == "BUY" {
            if let Some(price) = intent.price {
                let notional = price * intent.quantity;
                self.portfolio.available_margin = (self.portfolio.available_margin - notional).max(dec!(0));
                self.portfolio.used_margin = (self.portfolio.used_margin + notional).min(self.portfolio.total_equity);
            }
        }
        let response = OrderResponse {
            order_id: id.clone(),
            symbol: intent.symbol.clone(),
            side: intent.side.clone(),
            status: OrderStatus::Open,
            price: intent.price,
            quantity: intent.quantity,
            filled_quantity: dec!(0),
            created_at,
        };
        self.orders.insert(id.clone(), response.clone());
        Ok(response)
    }

    fn modify_order(
        &mut self,
        order_id: &str,
        intent: OrderIntent,
    ) -> Result<OrderResponse, BrokerError> {
        let order = self.orders.get_mut(order_id).ok_or_else(|| {
            BrokerError::Rejected {
                message: format!("order {order_id} not found"),
            }
        })?;
        if order.status != OrderStatus::Open {
            return Err(BrokerError::Rejected {
                message: format!("order {order_id} is not open (status {:?})", order.status),
            });
        }
        order.price = intent.price;
        order.quantity = intent.quantity;
        Ok(order.clone())
    }

    fn cancel_order(&mut self, order_id: &str) -> Result<(), BrokerError> {
        let order = self.orders.get_mut(order_id).ok_or_else(|| {
            BrokerError::Rejected {
                message: format!("order {order_id} not found"),
            }
        })?;
        if order.status != OrderStatus::Open {
            return Err(BrokerError::Rejected {
                message: format!("order {order_id} is not open (status {:?})", order.status),
            });
        }
        order.status = OrderStatus::Cancelled;
        Ok(())
    }

    fn fetch_portfolio(&mut self) -> Result<AccountSummary, BrokerError> {
        Ok(self.portfolio.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submit_modify_cancel_lifecycle() {
        let mut wallet = PaperWallet::new();
        assert!(!wallet.is_connected());
        wallet.connect();
        assert!(wallet.is_connected());

        let intent = OrderIntent {
            symbol: "RELIANCE".into(),
            side: "BUY".into(),
            order_type: "LIMIT".into(),
            price: Some(dec!(2500.0)),
            trigger_price: None,
            quantity: dec!(10.0),
        };
        let resp = wallet.submit_order(intent).unwrap();
        assert_eq!(resp.status, OrderStatus::Open);
        assert!(!resp.order_id.is_empty());
        assert_eq!(resp.symbol, "RELIANCE");

        // Margin reserved for the buy notional (2500 * 10 = 25000).
        let portfolio = wallet.fetch_portfolio().unwrap();
        assert_eq!(portfolio.available_margin, dec!(975_000));
        assert_eq!(portfolio.used_margin, dec!(25_000));

        // Modify price/quantity.
        let updated = wallet
            .modify_order(
                &resp.order_id,
                OrderIntent {
                    symbol: "RELIANCE".into(),
                    side: "BUY".into(),
                    order_type: "LIMIT".into(),
                    price: Some(dec!(2490.0)),
                    trigger_price: None,
                    quantity: dec!(15.0),
                },
            )
            .unwrap();
        assert_eq!(updated.price, Some(dec!(2490.0)));
        assert_eq!(updated.quantity, dec!(15.0));

        // Cancel.
        wallet.cancel_order(&resp.order_id).unwrap();
        let order = wallet.orders.get(&resp.order_id).unwrap();
        assert_eq!(order.status, OrderStatus::Cancelled);

        // Cancel again should fail.
        let err = wallet.cancel_order(&resp.order_id).unwrap_err();
        assert!(matches!(err, BrokerError::Rejected { .. }));
    }

    #[test]
    fn submit_when_disconnected_errors() {
        let mut wallet = PaperWallet::new();
        let err = wallet
            .submit_order(OrderIntent {
                symbol: "RELIANCE".into(),
                side: "BUY".into(),
                order_type: "LIMIT".into(),
                price: Some(dec!(2500.0)),
                trigger_price: None,
                quantity: dec!(10.0),
            })
            .unwrap_err();
        assert!(matches!(err, BrokerError::Network { .. }));
    }

    // #65: a fill transitions the order to Filled and updates PnL / margin.
    #[test]
    fn execute_fill_updates_position_and_pnl() {
        let mut wallet = PaperWallet::new();
        wallet.connect();
        let intent = OrderIntent {
            symbol: "RELIANCE".into(),
            side: "BUY".into(),
            order_type: "LIMIT".into(),
            price: Some(dec!(100.0)),
            trigger_price: None,
            quantity: dec!(10.0),
        };
        let resp = wallet.submit_order(intent).unwrap();

        let filled = wallet.execute_fill(&resp.order_id, dec!(110.0), dec!(10.0)).unwrap();
        assert_eq!(filled.status, OrderStatus::Filled);
        assert_eq!(filled.filled_quantity, dec!(10.0));

        let pos = wallet.positions.get("RELIANCE").unwrap();
        assert_eq!(pos.quantity, dec!(10.0));
        assert_eq!(pos.entry_price, dec!(100.0));
        // unrealized PnL at the fill price is zero.
        assert_eq!(pos.unrealized_pnl, dec!(0));

        let portfolio = wallet.fetch_portfolio().unwrap();
        assert_eq!(portfolio.used_margin, dec!(1000));
    }

    // #65: timestamps are deterministic (no SystemTime).
    #[test]
    fn timestamps_are_deterministic() {
        let mut wallet = PaperWallet::new();
        wallet.connect();
        let intent = OrderIntent {
            symbol: "RELIANCE".into(),
            side: "BUY".into(),
            order_type: "LIMIT".into(),
            price: Some(dec!(100.0)),
            trigger_price: None,
            quantity: dec!(1.0),
        };
        let first = wallet.submit_order(intent.clone()).unwrap().created_at;
        let second = wallet.submit_order(intent).unwrap().created_at;
        assert_eq!(second, first + 1);
    }
}
