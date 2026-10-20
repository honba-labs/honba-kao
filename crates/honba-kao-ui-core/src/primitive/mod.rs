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

// Icons are at the crate root level (icons/), re-exported via lib.rs