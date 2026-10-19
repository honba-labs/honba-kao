//! Demo credentials.
//!
//! honba-kao has no auth backend yet, so the session is created locally. These
//! are the seeded account used for development and demos; the sign-in flow in
//! [`crate::contexts::AuthContext`] validates against them.

/// Seeded demo account email.
pub const DEMO_EMAIL: &str = "test@test.com";

/// Seeded demo account password.
pub const DEMO_PASSWORD: &str = "test123";

/// Display name for the seeded demo account.
pub const DEMO_NAME: &str = "Test Trader";

/// Minimum password length accepted by registration.
pub const MIN_PASSWORD_LEN: usize = 8;
