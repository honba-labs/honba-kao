//! Shared reactive state (≈ React context). Mirrors `web/src/contexts/`.

pub mod auth_context;
pub mod confirm_dialog;
pub mod language_context;
pub mod symbol_link;

pub use auth_context::{AuthContext, User, use_auth};
pub use confirm_dialog::{ConfirmDialogContext, ConfirmDialogHost, use_confirm_dialog};
pub use language_context::{LanguageContext, use_language};
pub use symbol_link::{
    LinkBook, LinkGroupHandle, SymbolLinkContext, use_link_group, use_symbol_link,
};
