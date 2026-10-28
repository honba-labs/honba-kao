use crate::dto::market::{Bar, Ticker};
use gloo_net::http::Request;
use urlencoding::encode;

/// Fetch the latest ticker for a symbol.
///
/// #68: the symbol is percent-encoded (Indian symbols like `NSE:NIFTY 50`,
/// `M&M`, `NFO:NIFTY24DEC24000CE` contain spaces/`&`) and non-2xx responses
/// surface the server's error body instead of an obscure JSON decode failure.
pub async fn get_ticker(symbol: &str) -> Result<Ticker, String> {
    let url = format!("/api/v1/ticker/{}", encode(symbol));
    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.ok() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }
    resp.json::<Ticker>()
        .await
        .map_err(|e| format!("JSON decode error: {e}"))
}

pub async fn get_ohlcv(symbol: &str, timeframe: &str, limit: u32) -> Result<Vec<Bar>, String> {
    let url = format!(
        "/api/v1/ohlcv?symbol={}&tf={}&limit={}",
        encode(symbol),
        encode(timeframe),
        limit
    );
    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.ok() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }
    resp.json::<Vec<Bar>>()
        .await
        .map_err(|e| format!("JSON decode error: {e}"))
}
