//! The strategy-graph node editor (HBD-2026-DES-006 P2).
//!
//! - [`geometry`]: grid snap, card box model, smoothstep routing,
//!   auto-layout, viewport math (pure).
//! - [`scene`]: the single render pass the editor memoizes (pure).
//! - [`editor`]: the SVG viewport and its gestures.
//! - [`node_card`]: the one generic node card.
//! - [`edge`]: the insertable smoothstep edge and its splice menu.
//! - [`palette`]: the drag-and-drop node palette (`application/honba-node`).
//! - [`inspector`]: the selected node / edge / strategy settings form.
//! - [`dom`]: `DataTransfer` / `DOMRect` / focus shims.

pub mod dom;
pub mod edge;
pub mod editor;
pub mod geometry;
pub mod inspector;
pub mod node_card;
pub mod palette;
pub mod scene;

pub use editor::CanvasEditor;
pub use geometry::{Position, Viewport};
pub use inspector::Inspector;
pub use palette::Palette;
pub use scene::Selection;
