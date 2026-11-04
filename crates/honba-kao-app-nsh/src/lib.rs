pub mod app;
pub mod features;
pub mod router;
pub mod shared;

use nsh_core::prelude::*;

// Re-exported so the NSH native module entry point (below) can reference `App`.
pub use app::*;

// NSH native module entry point: registers the app's root component `App`
// so the `nsh` CLI (`nsh dev`/`nsh build`) can discover and render it
// (per nsh.json `"root": "App"`).
#[native_module("app/native")]
fn native(registry: &mut Registry) {
    registry.register::<App>();
}

// NSH port (nsh-port.md Phase 5): no `#[wasm_bindgen(start)]` entry point —
// the `nsh` CLI (external tooling, not a Cargo dependency) discovers an
// app's root component from its crate root per `nsh-core`'s own README
// ("the Rust template includes a Cargo project with its root component in
// `src/lib.rs`"); `mount()` is a TSX-only API, irrelevant here. This mirrors
// `honba-kao-ui-core/src/lib.rs`'s own `pub use app::*;` convention.
