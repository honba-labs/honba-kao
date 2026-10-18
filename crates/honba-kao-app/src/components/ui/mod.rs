//! UI primitives. Mirrors `web/src/components/ui/` (Radix-based in the
//! reference app) and the bfx-hf-ui-core design language already in this repo.

pub mod badge;
pub mod button;
pub mod card;
pub mod kpi;
pub mod modal;
pub mod page_header;
pub mod panel;
pub mod pl_number;
pub mod placeholder;
pub mod status_circle;

pub use badge::*;
pub use button::*;
pub use card::*;
pub use kpi::*;
pub use modal::*;
pub use page_header::*;
pub use panel::*;
pub use pl_number::*;
pub use placeholder::*;
pub use status_circle::*;
