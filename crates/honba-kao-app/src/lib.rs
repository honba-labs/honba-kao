//! 本場・顔 honba-kao — Leptos SPA root.
use honba_kao_client::ApiClient;
use leptos::mount::mount_to_body;
use leptos::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;

#[component]
pub fn App() -> impl IntoView {
    // Dev: "/api" is proxied to :3000 by Trunk.
    // Prod: override via a build-time env or a meta tag in index.html.
    let _api = ApiClient::new("/api");

    view! {
        <main style="font-family: system-ui; max-width: 64rem; margin: 4rem auto; padding: 0 1rem;">
            <h1>"本場・顔 honba-kao"</h1>
            <p>"Indian equities algo-trading dashboard."</p>
            <p>
                <a href="/health" target="_blank">"/api/health"</a>
            </p>
        </main>
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    mount_to_body(App);
}
