//! The strategy studio (HBD-2026-DES-006).
//!
//! Layout of the port, one submodule per design-doc layer:
//! - [`graph`]: the pure graph kernel — port map, catalogue, validation
//!   (fe's twelve codes plus four), hash, diff, defaults.
//! - [`codegen`]: deterministic Python rendering of the graph plus the
//!   lossless recognizer (the code⇄graph lockstep).
//! - [`templates`]: the starting strategies, three of which honba-ya's
//!   registry can execute by name today (the honest, shipped path).
//!
//! Later phases land store/canvas/panels/research alongside these.
pub mod graph;
pub mod codegen;
