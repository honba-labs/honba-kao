//! Transport layer. Mirrors `web/src/lib/httpClient.ts`, built on the existing
//! typed `honba-kao-client` crate (`ApiClient::get_json`).

use honba_kao_client::ApiClient;
use serde::de::DeserializeOwned;

pub use honba_kao_client::ClientError as HttpError;

/// Base URL of the honba-ya API. Trunk proxies `/api` to `:3000` in dev; set a
/// real origin for production.
pub fn base_url() -> String {
    crate::constants::branding::API_BASE_URL.to_string()
}

/// The shared typed client.
pub fn client() -> ApiClient {
    ApiClient::new(base_url())
}

/// Typed GET helper so API namespaces can adopt the network one call at a time.
pub async fn get_json<T: DeserializeOwned>(path: &str) -> Result<T, HttpError> {
    client().get_json::<T>(path).await
}
