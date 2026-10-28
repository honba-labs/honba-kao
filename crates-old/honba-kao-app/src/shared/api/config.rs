//! System config API namespace. Mirrors `web/src/lib/api/config.ts`.

use crate::shared::types::SystemConfig;

pub fn get_system_config() -> SystemConfig {
    SystemConfig::default()
}
