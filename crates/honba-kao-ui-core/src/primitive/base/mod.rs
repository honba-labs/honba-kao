//! Tier 1 base shims — one file per wrapped `nsh_base` component.
//!
//! The rest of the tree depends on honba's own `Hf*` wrapper types, never on
//! `nsh_base::*` directly. This keeps primitives swappable if `nsh_base` makes a
//! breaking change (see `nsh-port.md` §3).

pub mod button;
pub mod dialog;
pub mod input;
pub mod link;
pub mod plot;
pub mod progress;
pub mod sheet;
pub mod slider;
pub mod switch;
pub mod tabs;
pub mod toast;

pub use button::{HfButton, HfChip, HfCloseButton, HfIconButton};
pub use dialog::{HfDialog, HfDrawer, HfModal};
pub use input::{HfInput, HfNumberInput};
pub use link::HfLink;
pub use plot::{Curve, HfGrid, HfLine, HfPlot};
pub use progress::{HfProgress, HfSpinner};
pub use sheet::HfSheet;
pub use switch::HfSwitch;
pub use tabs::HfTabs;
pub use toast::{HfAlertBanner, HfAlertBannerStack, HfToastStack};
