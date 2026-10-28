use crate::dto::portfolio::{AccountSummary, BrokerConnection, Holding, Position};
use gloo_net::http::Request;

pub async fn get_portfolio_snapshot() -> Result<AccountSummary, String> {
    let resp = Request::get("/api/v1/portfolio/snapshot")
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    resp.json::<AccountSummary>()
        .await
        .map_err(|e| format!("JSON decode error: {e}"))
}

pub async fn get_positions() -> Result<Vec<Position>, String> {
    let resp = Request::get("/api/v1/portfolio/positions")
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    resp.json::<Vec<Position>>()
        .await
        .map_err(|e| format!("JSON decode error: {e}"))
}

pub async fn get_holdings() -> Result<Vec<Holding>, String> {
    let resp = Request::get("/api/v1/portfolio/holdings")
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    resp.json::<Vec<Holding>>()
        .await
        .map_err(|e| format!("JSON decode error: {e}"))
}

pub async fn get_brokers() -> Result<Vec<BrokerConnection>, String> {
    let resp = Request::get("/api/v1/brokers")
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    resp.json::<Vec<BrokerConnection>>()
        .await
        .map_err(|e| format!("JSON decode error: {e}"))
}
