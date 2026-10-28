use gloo_net::http::Request;
use serde_json::Value;

pub async fn scan_screener(payload: &Value) -> Result<Value, String> {
    let resp = Request::post("/api/v1/screener/scan")
        .json(payload)
        .map_err(|e| format!("Serialization error: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    resp.json::<Value>()
        .await
        .map_err(|e| format!("JSON decode error: {e}"))
}
