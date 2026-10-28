use crate::dto::market::{Bar, Ticker};
use gloo_net::http::Request;

pub async fn get_ticker(symbol: &str) -> Result<Ticker, String> {
    let url = format!("/api/v1/ticker/{}", symbol);
    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    resp.json::<Ticker>()
        .await
        .map_err(|e| format!("JSON decode error: {e}"))
}

pub async fn get_ohlcv(symbol: &str, timeframe: &str, limit: u32) -> Result<Vec<Bar>, String> {
    let url = format!(
        "/api/v1/ohlcv?symbol={}&tf={}&limit={}",
        symbol, timeframe, limit
    );
    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    resp.json::<Vec<Bar>>()
        .await
        .map_err(|e| format!("JSON decode error: {e}"))
}
