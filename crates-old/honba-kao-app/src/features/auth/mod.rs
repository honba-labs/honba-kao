//! Auth — the full-bleed sign-in, registration, password-reset and
//! first-run setup screens. Mirrors `web/src/components/auth/` and
//! `web/src/components/modals/SetupPage.tsx`.

pub mod components;
pub mod login_page;
pub mod register_page;
pub mod reset_password_page;
pub mod setup_page;

pub use login_page::LoginPage;
pub use register_page::RegisterPage;
pub use reset_password_page::ResetPasswordPage;
pub use setup_page::SetupPage;
