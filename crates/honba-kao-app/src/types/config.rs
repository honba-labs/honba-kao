//! System configuration types. Mirrors `web/src/types/config.ts`.

/// Server-side system configuration (drives setup vs. login routing).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemConfig {
    pub initialized: bool,
    pub registration_enabled: bool,
    pub paper_trading: bool,
    pub version: &'static str,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            initialized: true,
            registration_enabled: true,
            paper_trading: true,
            version: crate::constants::branding::VERSION,
        }
    }
}
