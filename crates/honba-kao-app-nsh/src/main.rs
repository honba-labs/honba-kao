//! Runs the workstation directly on the machine's native backend (Stage 3
//! Phase A.5 of the gallery-rs porting guide): `cargo run -p honba-kao-app`
//! on Linux (GTK 4), macOS (AppKit) and Windows (Win32). The web flow is
//! unchanged — `nsh run --target web` links the *lib's* `cdylib` (the
//! `#[native_module("app/native")]` entry in `lib.rs`), never this bin.

#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(target_os = "linux")]
use nsh_platform_linux as platform;
#[cfg(target_os = "macos")]
use nsh_platform_macos as platform;
#[cfg(target_os = "windows")]
use nsh_platform_windows as platform;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn main() -> impl std::process::Termination {
    env_logger::init();
    // Spec (`honba-kao-assets-spec.md`) §6.3: the assets crate stages the
    // canonical fonts/audio/branding tree next to the executable, and the
    // theme comes from that same crate. The crate-local `theme.json` stays
    // for the `nsh run --target web` flow (its `nsh.json` reads it);
    // honba-kao-assets' `theme_canonical_test` pins the two byte-equal.
    // ui-core contributes no separate UI set (as in the showcase runner).
    let assets_reg = honba_kao_assets::registry();
    let app = honba_kao_app_nsh::registry();
    let assets = asset_directory(&[&assets_reg, &app]).expect("copy the registered assets");
    platform::set_ui_sets(vec![nsh_base::registry(), assets_reg, app]);
    let theme = honba_kao_assets::themes::default_theme();
    // nsh.json's id: the window's and where the app keeps its data.
    let app_id = "dev.honba.kao";
    platform::run_app(
        platform::AppSource::Rust(
            platform::Root::of::<honba_kao_app_nsh::App>()
                .tokens(theme.into_tokens())
                .app_id(app_id),
        ),
        platform::AppOptions {
            title: "Honba Kao".to_owned(),
            asset_base: Some(platform::AssetBase::Directory(assets)),
            app_id: app_id.to_owned(),
            size: (1280, 860),
            ..Default::default()
        },
    )
}

// One deviation from the showcase runner (which fires `compile_error!` for
// every non-desktop target): the workspace's standing wasm gate
// (`cargo check -p honba-kao-app --target wasm32-unknown-unknown`) checks
// the *bin* target too, so wasm32 keeps a compilable stub — the web
// runtime links the cdylib and never runs this bin.
#[cfg(target_arch = "wasm32")]
fn main() {}

// Run this binary on macOS, Windows or Linux; `nsh run` handles iOS,
// Android and the web (which goes through the lib's cdylib as well).
#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "windows",
    target_arch = "wasm32"
)))]
compile_error!(
    "Run this binary on macOS, Windows or Linux; use nsh run for iOS, Android and the web."
);

// Registered asset URIs keep the layout the development hosts serve them in.
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn asset_directory(
    registries: &[&nsh_core::element::Registry],
) -> std::io::Result<std::path::PathBuf> {
    let executable = std::env::current_exe()?;
    let base = executable
        .parent()
        .expect("the executable has a directory")
        .join("honba-kao-assets");
    for registry in registries {
        for file in registry.asset_files() {
            let destination = base.join(file.path());
            std::fs::create_dir_all(
                destination
                    .parent()
                    .expect("an asset has a parent directory"),
            )?;
            std::fs::copy(file.source, destination)?;
        }
    }
    Ok(base)
}
