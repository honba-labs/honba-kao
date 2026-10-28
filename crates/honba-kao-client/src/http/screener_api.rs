use gloo_net::http::Request;
use serde_json::Value;
use urlencoding::encode;

pub async fn scan_screener(payload: &Value) -> Result<Value, String> {
    let resp = Request::post("/api/v1/screener/scan")
        .json(payload)
        .map_err(|e| format!("Serialization error: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.ok() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }
    resp.json::<Value>()
        .await
        .map_err(|e| format!("JSON decode error: {e}"))
}

/// Build a percent-encoded screener query string from optional parameters.
///
/// #68: symbols and other free-text filter values can contain characters that
/// are unsafe in a query string (spaces, `&`, `:`).
pub fn screener_query(params: &[(&str, &str)]) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}
