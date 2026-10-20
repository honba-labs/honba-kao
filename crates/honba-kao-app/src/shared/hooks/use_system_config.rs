//! System config access. Mirrors `web/src/hooks/useSystemConfig.ts`.
//!
//! The reference app fetches this over SWR; here it resolves synchronously from
//! the API namespace so routing can branch on it without a loading state.

use crate::shared::api;
use crate::shared::types::SystemConfig;

pub fn use_system_config() -> SystemConfig {
    api::config::get_system_config()
}
