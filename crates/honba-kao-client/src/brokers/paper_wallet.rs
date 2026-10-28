//! In-memory Paper Trading broker adapter.
//!
//! Fully functional without network: orders are stored in memory, assigned
//! sequential IDs, and can be modified/cancelled. Suitable for unit tests,
//! offline development, and paper-trading demos.

use crate::brokers::{BrokerClient, BrokerError};
use crate::dto::order::{OrderIntent, OrderResponse, OrderStatus};
use crate::dto::portfolio::AccountSummary;

use std::collections::HashMap;

/// Paper-trading broker that simulates order lifecycle in memory.
pub struct PaperWallet {
    name: String,
    connected: bool,
    orders: HashMap<String, OrderResponse>,
    next_id: u64,
    portfolio: AccountSummary,
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
            portfolio: AccountSummary {
                account_id: "PAPER-001".into(),
                currency: "INR".into(),
                total_equity: 1_000_000.0,
                available_margin: 1_000_000.0,
                used_margin: 0.0,
                unrealized_pnl: 0.0,
                realized_pnl: 0.0,
            },
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

    fn next_order_id(&mut self) -> String {
        let id = self.next_id;
        self.next_id += 1;
        format!("PAPER-{id:06}")
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
        let response = OrderResponse {
            order_id: id.clone(),
            symbol: intent.symbol.clone(),
            side: intent.side.clone(),
            status: OrderStatus::Open,
            price: intent.price,
            quantity: intent.quantity,
            filled_quantity: 0.0,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
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
            price: Some(2500.0),
            trigger_price: None,
            quantity: 10.0,
        };
        let resp = wallet.submit_order(intent).unwrap();
        assert_eq!(resp.status, OrderStatus::Open);
        assert!(!resp.order_id.is_empty());
        assert_eq!(resp.symbol, "RELIANCE");

        // Modify price/quantity.
        let updated = wallet
            .modify_order(
                &resp.order_id,
                OrderIntent {
                    symbol: "RELIANCE".into(),
                    side: "BUY".into(),
                    order_type: "LIMIT".into(),
                    price: Some(2490.0),
                    trigger_price: None,
                    quantity: 15.0,
                },
            )
            .unwrap();
        assert!((updated.price.unwrap() - 2490.0).abs() < 1e-9);
        assert!((updated.quantity - 15.0).abs() < 1e-9);

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
                price: Some(2500.0),
                trigger_price: None,
                quantity: 10.0,
            })
            .unwrap_err();
        assert!(matches!(err, BrokerError::Network { .. }));
    }

    #[test]
    fn fetch_portfolio_returns_summary() {
        let mut wallet = PaperWallet::new();
        let summary = wallet.fetch_portfolio().unwrap();
        assert_eq!(summary.account_id, "PAPER-001");
        assert!((summary.total_equity - 1_000_000.0).abs() < 1e-9);
    }
}
