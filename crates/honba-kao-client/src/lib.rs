//! Typed HTTP client for honba-ya-api.
//!
//! In dev, the base URL is "/api" and Trunk proxies to :3000.
//! In prod, set it to your API origin, e.g. "https://api.honba.in".
use gloo_net::http::Request;
use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("network: {0}")]
    Network(String),
    #[error("decode: {0}")]
    Decode(String),
    #[error("status {0}")]
    Status(u16),
}

#[derive(Clone)]
pub struct ApiClient {
    pub base_url: String,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self { base_url: base_url.into() }
    }

    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, ClientError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = Request::get(&url)
            .send()
            .await
            .map_err(|e| ClientError::Network(e.to_string()))?;
        if !resp.ok() {
            return Err(ClientError::Status(resp.status()));
        }
        resp.json::<T>()
            .await
            .map_err(|e| ClientError::Decode(e.to_string()))
    }
}
