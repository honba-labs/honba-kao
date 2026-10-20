//! Primitive UI components — unstyled, accessible, Radix/Shadcn parity.
//! All components compose from `honba-kao-ui-core` design tokens.

pub mod button;
pub mod input;
pub mod select;
pub mod checkbox;
pub mod radio;
pub mod switch;
pub mod label;
pub mod separator;
pub mod tabs;
pub mod panel;
pub mod page_header;
pub mod placeholder;
pub mod card;
pub mod kpi;
pub mod badge;
pub mod segmented;

// Icons are at the crate root level (icons/), re-exported via lib.rs