//! Domain feature slices. Each slice owns its page, its components, its
//! reactive stores and any data or pure kernel only it uses; code shared by
//! several slices lives in [`crate::shared`].

pub mod account;
pub mod auth;
pub mod backtesting;
pub mod chart_workstation;
pub mod compare;
pub mod dom;
pub mod dossiers;
pub mod fno;
pub mod home;
pub mod landing;
pub mod launchpad;
pub mod markets;
pub mod mta;
pub mod model_lab;
pub mod oms;
pub mod ops;
pub mod pair_trading;
pub mod portfolio;
pub mod portfolio_extras;
pub mod portfolio_lab;
#[path = "position-sizer/mod.rs"]
pub mod position_sizer;
pub mod quant_extras;
pub mod research;
pub mod risk;
pub mod risk_ops_extras;
pub mod screener;
pub mod security_hub;
#[path = "shadow-account/mod.rs"]
pub mod shadow_account;
pub mod simulator;
pub mod stat_lab;
pub mod strategy_studio;
pub mod system_extras;
pub mod tape;
pub mod tradebook;
pub mod workbench;
