//! `primitive::icons` — re-exports the procedural outline icons and [`HfIcon`].
//!
//! The implementations live at `crate::icons` (`icons/outline.rs`); this module
//! exists so `honba_kao_ui_core::primitive::icons::*` and `crate::primitive::*`
//! match the layout in `plan.md` §3.2.

pub use crate::icons::outline::{HfIcon, SvgIcon};
pub use crate::icons::*;
