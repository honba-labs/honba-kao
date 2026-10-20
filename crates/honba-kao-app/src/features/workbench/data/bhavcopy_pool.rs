//! The NSE bhavcopy pool backing the symbol-search modal — the labs app's
//! `core/bhavcopy-pool.ts`, extracted to `data/bhavcopy-pool.json`.

use std::sync::OnceLock;

use serde::Deserialize;

/// One raw pool row: `[symbol, name, exchange, badge, price, change, changePercent]`.
#[derive(Clone, Debug)]
pub struct BhavcopyRow {
    pub symbol: String,
    pub name: String,
    pub exchange: String,
    pub badge: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
}

impl<'de> Deserialize<'de> for BhavcopyRow {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Array form from the raw pool file.
        struct RowVisitor;
        impl<'de> serde::de::Visitor<'de> for RowVisitor {
            type Value = BhavcopyRow;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("[symbol, name, exchange, badge, price, change, changePercent]")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let symbol = seq.next_element()?.unwrap_or_default();
                let name = seq.next_element()?.unwrap_or_default();
                let exchange = seq.next_element()?.unwrap_or_default();
                let badge = seq.next_element()?.unwrap_or_default();
                let price = seq.next_element()?.unwrap_or_default();
                let change = seq.next_element()?.unwrap_or_default();
                let change_percent = seq.next_element()?.unwrap_or_default();
                Ok(BhavcopyRow {
                    symbol,
                    name,
                    exchange,
                    badge,
                    price,
                    change,
                    change_percent,
                })
            }
        }

        deserializer.deserialize_seq(RowVisitor)
    }
}

static POOL: OnceLock<Vec<BhavcopyRow>> = OnceLock::new();

/// Raw rows loaded from the bundled pool (3,701 NSE rows: EQ, SME, BOND, ETF, MF, F&O, …).
pub fn pool() -> &'static [BhavcopyRow] {
    POOL.get_or_init(|| {
        let raw = include_str!("../../../../data/bhavcopy-pool.json");
        serde_json::from_str(raw).expect("data/bhavcopy-pool.json parses")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_loads_labs_rows() {
        assert_eq!(pool().len(), 3701);
        assert_eq!(pool()[0].symbol, "PCJEWELLER");
    }
}
