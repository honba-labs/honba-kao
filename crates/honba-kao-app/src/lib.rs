//! 本場・顔 honba-kao — Leptos SPA root.
//!
//! The crate is laid out to mirror the `web/` app of `NoFxAiOS/nofx`
//! (Vite + React + TS), translated to idiomatic Rust/Leptos:
//!
//! ```text
//! web/src/            ->  crates/honba-kao-app/src/
//!   App.tsx           ->    app.rs            (provider stack)
//!   main.tsx          ->    lib.rs            (mount_to_body)
//!   router/           ->    router/           (paths + AppRoutes + AppChrome)
//!   pages/            ->    pages/            (top-level screens)
//!   components/<dom>/ ->    components/<dom>/ (domain components)
//!   components/ui/    ->    honba-kao-ui-core (primitives, separate crate)
//!   contexts/         ->    contexts/         (provide_context / expect_context)
//!   stores/           ->    stores/           (reactive signals per store)
//!   lib/api/          ->    api/              (namespaced client modules)
//!   lib/*             ->    utils/            (format, indicators, text)
//!   hooks/            ->    hooks/            (reusable reactive helpers)
//!   types/            ->    types/
//!   i18n/             ->    i18n/
//!   constants/        ->    constants/
//!   data/             ->    data/
//!   styles/           ->    ../../style/      (Trunk-served CSS)
//! ```

pub mod api;
pub mod app;
pub mod components;
pub mod constants;
pub mod contexts;
pub mod data;
pub mod hooks;
pub mod i18n;
pub mod pages;
pub mod router;
pub mod sim;
pub mod stores;
pub mod studio;
pub mod theme;
pub mod types;
pub mod utils;

pub use app::App;

use wasm_bindgen::prelude::wasm_bindgen;

/// WASM entry point: wire panic/tracing and mount the app into `<body>`.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    leptos::mount::mount_to_body(App);
}
