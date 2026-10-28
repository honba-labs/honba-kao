use crate::dto::order::{OrderIntent, OrderResponse};
use gloo_net::http::Request;

pub async fn submit_order(intent: &OrderIntent) -> Result<OrderResponse, String> {
    let resp = Request::post("/api/v1/orders")
        .json(intent)
        .map_err(|e| format!("Serialization error: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    resp.json::<OrderResponse>()
        .await
        .map_err(|e| format!("JSON decode error: {e}"))
}

pub async fn cancel_order(order_id: &str) -> Result<bool, String> {
    let url = format!("/api/v1/orders/{}", order_id);
    let resp = Request::delete(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    Ok(resp.ok())
}
