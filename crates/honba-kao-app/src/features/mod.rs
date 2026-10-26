//! Domain feature slices. Each slice owns its page, its components, its
//! reactive stores and any data or pure kernel only it uses; code shared by
//! several slices lives in [`crate::shared`].

pub mod account;
pub mod auth;
pub mod dossiers;
pub mod fno;
pub mod landing;
pub mod portfolio;
pub mod screener;
pub mod security_hub;
pub mod simulator;
pub mod strategy_studio;
pub mod tradebook;
pub mod workbench;
