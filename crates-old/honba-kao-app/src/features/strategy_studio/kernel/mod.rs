//! The strategy studio (HBD-2026-DES-006).
//!
//! Layout of the port, one submodule per design-doc layer:
//! - [`graph`]: the pure graph kernel — port map, catalogue, validation
//!   (fe's twelve codes plus four), hash, diff, defaults.
//! - [`codegen`]: deterministic Python rendering of the graph plus the
//!   lossless recognizer (the code⇄graph lockstep).
//! - [`templates`]: fe's six starting strategies plus the blank graph.
//! - [`edit`]: the pure graph edits every editor gesture lands on (add,
//!   delete, connect, splice, parameter writes).
//!
//! Later phases land store/canvas/panels/research alongside these.
pub mod codegen;
pub mod edit;
pub mod graph;
pub mod templates;
