//! Broker-linking API namespace.
//!
//! Real deployments drive a hosted OAuth/API-key handshake per broker (Zerodha
//! Kite Connect, Dhan, Upstox, Angel One SmartAPI, Fyers, Groww). This scaffold
//! keeps the same surface — list, connect, disconnect — over demo data so the
//! portfolio flow is complete before the transport is swapped in.

use crate::data::demo_data;
use crate::types::{BrokerAccount, BrokerStatus};

/// Every broker the app knows about, connected or not.
pub fn list() -> Vec<BrokerAccount> {
    demo_data::demo_brokers()
}

/// Link an account.
///
/// Returns the updated broker (with a demo client id) so the caller can patch
/// its local list. The error arm covers unknown ids; a real implementation
/// would also surface expired API keys and failed authorisations here.
pub fn connect(broker_id: &str) -> Result<BrokerAccount, String> {
    let mut broker = demo_data::demo_brokers()
        .into_iter()
        .find(|broker| broker.id == broker_id)
        .ok_or_else(|| format!("unknown broker: {broker_id}"))?;

    broker.status = BrokerStatus::Connected;
    broker.client_id = "DEMO1042";
    broker.last_sync = "just now";
    Ok(broker)
}

/// Unlink an account. Holdings and funds for it drop out of the aggregate.
pub fn disconnect(broker_id: &str) -> Result<(), String> {
    if demo_data::demo_brokers()
        .iter()
        .any(|broker| broker.id == broker_id)
    {
        Ok(())
    } else {
        Err(format!("unknown broker: {broker_id}"))
    }
}
