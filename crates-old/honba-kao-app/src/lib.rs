//! 本場・顔 honba-kao — Leptos SPA root.
//!
//! The crate is feature-sliced: each domain screen owns its page, components,
//! stores and private data under `features/<name>/`; the router and persistent
//! shell sit beside it, and anything used by more than one slice lives in
//! `shared/`. Translated from the `web/` app of `NoFxAiOS/nofx`:
//!
//! ```text
//! web/src/            ->  crates/honba-kao-app/src/
//!   App.tsx           ->    app.rs                (provider stack)
//!   main.tsx          ->    lib.rs                (mount_to_body)
//!   router/           ->    router/               (paths + AppRoutes + AppChrome + chrome/)
//!   pages/ + components/<dom>/ + stores/
//!                     ->    features/<dom>/       (page, components/, stores/, data/)
//!   components/ui/    ->    honba-kao-ui-core     (primitives, separate crate)
//!   components/common ->    shared/components/    (cross-feature components)
//!   contexts/         ->    shared/contexts/      (provide_context / expect_context)
//!   lib/api/          ->    shared/api/           (namespaced client modules)
//!   lib/*             ->    shared/utils/         (format, indicators, text)
//!   hooks/, types/, i18n/, constants/, data/
//!                     ->    shared/<same name>/
//!   styles/           ->    ../../style/          (Trunk-served CSS)
//! ```

pub mod app;
pub mod features;
pub mod router;
pub mod shared;

pub use app::App;

use wasm_bindgen::prelude::wasm_bindgen;

/// WASM entry point: wire panic/tracing and mount the app into `<body>`.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    leptos::mount::mount_to_body(App);
}
