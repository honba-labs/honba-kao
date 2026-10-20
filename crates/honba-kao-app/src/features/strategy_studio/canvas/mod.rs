//! Canvas module for the Strategy Studio node editor.
//! 
//! Provides the SVG-based viewport, node rendering, edge rendering,
//! and palette integration.
//! 
//! See `editor.rs` for the core implementation details.
//! 
//! # Phases (HBD-2026-DES-006)
//! - P2: Canvas editor with pan/zoom, node dragging, edge creation

pub use editor::CanvasConfig;
pub use editor::Position;
pub mod editor;