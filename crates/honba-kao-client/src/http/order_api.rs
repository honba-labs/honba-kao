use crate::dto::order::{OrderIntent, OrderResponse};
use gloo_net::http::Request;
use urlencoding::encode;

pub async fn submit_order(intent: &OrderIntent) -> Result<OrderResponse, String> {
    let resp = Request::post("/api/v1/orders")
        .json(intent)
        .map_err(|e| format!("Serialization error: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.ok() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }
    resp.json::<OrderResponse>()
        .await
        .map_err(|e| format!("JSON decode error: {e}"))
}

pub async fn cancel_order(order_id: &str) -> Result<bool, String> {
    // #68: percent-encode the order id (it may contain characters unsafe in a
    // URL path) and surface non-2xx error responses.
    let url = format!("/api/v1/orders/{}", encode(order_id));
    let resp = Request::delete(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.ok() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }
    Ok(resp.ok())
}
