//! Broker and aggregated-portfolio types.
//!
//! The portfolio page is a *view over many broker accounts*: holdings and funds
//! keep the broker they came from, and the summary is derived from whichever
//! accounts are currently connected.

/// Connection state of one broker account.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BrokerStatus {
    Connected,
    /// Token expires soon — the reference app surfaces this before sync fails.
    Expiring,
    Disconnected,
}

impl BrokerStatus {
    pub const fn label(self) -> &'static str {
        match self {
            BrokerStatus::Connected => "Connected",
            BrokerStatus::Expiring => "Re-auth soon",
            BrokerStatus::Disconnected => "Not connected",
        }
    }

    pub const fn is_connected(self) -> bool {
        matches!(self, BrokerStatus::Connected | BrokerStatus::Expiring)
    }
}

/// A brokerage the app can link. `id` is stable and used as the foreign key on
/// [`Holding`] and [`BrokerFunds`].
#[derive(Clone, Debug, PartialEq)]
pub struct BrokerAccount {
    pub id: &'static str,
    pub name: &'static str,
    pub legal_name: &'static str,
    /// Two-letter mark shown in the card when there is no logo asset.
    pub monogram: &'static str,
    /// CSS token (e.g. `"--broker-zerodha"`) used for the card accent.
    pub tone: &'static str,
    pub status: BrokerStatus,
    pub client_id: &'static str,
    pub segments: &'static [&'static str],
    pub last_sync: &'static str,
}

/// One holding, attributed to the broker it sits with.
#[derive(Clone, Debug, PartialEq)]
pub struct Holding {
    pub symbol: String,
    pub name: String,
    pub broker: &'static str,
    pub sector: &'static str,
    pub qty: i64,
    pub avg_price: f64,
    pub last_price: f64,
    pub day_change_pct: f64,
}

impl Holding {
    pub fn invested(&self) -> f64 {
        self.qty as f64 * self.avg_price
    }

    pub fn value(&self) -> f64 {
        self.qty as f64 * self.last_price
    }

    pub fn pnl(&self) -> f64 {
        self.value() - self.invested()
    }

    pub fn pnl_pct(&self) -> f64 {
        if self.invested() == 0.0 {
            0.0
        } else {
            self.pnl() / self.invested() * 100.0
        }
    }

    /// Today's move in rupees, derived from the day-change percentage.
    pub fn day_pnl(&self) -> f64 {
        let previous = self.value() / (1.0 + self.day_change_pct / 100.0);
        self.value() - previous
    }
}

/// Cash and margin at one broker.
#[derive(Clone, Debug, PartialEq)]
pub struct BrokerFunds {
    pub broker: &'static str,
    pub cash: f64,
    pub collateral: f64,
    pub used_margin: f64,
}

impl BrokerFunds {
    pub fn free(&self) -> f64 {
        self.cash + self.collateral - self.used_margin
    }
}

/// Everything the KPI row and allocation bars need, pre-computed.
#[derive(Clone, Debug, PartialEq)]
pub struct PortfolioSummary {
    pub holdings_value: f64,
    pub invested: f64,
    pub unrealised_pnl: f64,
    pub unrealised_pct: f64,
    pub day_pnl: f64,
    pub cash: f64,
    pub collateral: f64,
    pub used_margin: f64,
}

impl PortfolioSummary {
    /// Holdings plus free cash plus collateral — the number shown as total.
    pub fn total_value(&self) -> f64 {
        self.holdings_value + self.cash + self.collateral
    }

    pub fn free_cash(&self) -> f64 {
        self.cash + self.collateral - self.used_margin
    }
}

/// A consistent slice of the portfolio for one set of connected brokers.
#[derive(Clone, Debug, PartialEq)]
pub struct PortfolioSnapshot {
    pub brokers: Vec<BrokerAccount>,
    pub holdings: Vec<Holding>,
    pub funds: Vec<BrokerFunds>,
    pub summary: PortfolioSummary,
}

impl PortfolioSnapshot {
    pub fn connected_brokers(&self) -> usize {
        self.brokers
            .iter()
            .filter(|broker| broker.status.is_connected())
            .count()
    }

    /// Value held at one broker, including its cash and collateral.
    pub fn value_at(&self, broker_id: &str) -> f64 {
        let holdings: f64 = self
            .holdings
            .iter()
            .filter(|holding| holding.broker == broker_id)
            .map(Holding::value)
            .sum();
        let funds = self
            .funds
            .iter()
            .find(|funds| funds.broker == broker_id)
            .map_or(0.0, |funds| funds.cash + funds.collateral);

        holdings + funds
    }
}
