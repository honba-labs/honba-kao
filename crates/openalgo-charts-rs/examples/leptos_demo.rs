//! Leptos 0.7 Example using openalgo-charts-rs
//!
//! Run with:
//! `cargo build --example leptos_demo --target wasm32-unknown-unknown`

use leptos::prelude::*;
use openalgo_charts::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    // Reactive bar signal
    let (bars, _set_bars) = signal(vec![
        Bar::new(1700000000, 100.0, 105.0, 99.5, 104.0, 1500.0),
        Bar::new(1700086400, 104.0, 108.0, 103.0, 107.5, 2300.0),
        Bar::new(1700172800, 107.5, 112.0, 106.0, 110.0, 3100.0),
    ]);

    let on_crosshair = Callback::new(move |ev: CrosshairMoveEvent| {
        if let Some(price) = ev.price {
            web_sys::console::log_1(&format!("Hovered price: {price}").into());
        }
    });

    view! {
        <main style="display: flex; flex-direction: column; width: 100vw; height: 100vh; background: #131722;">
            <header style="padding: 12px; color: #fff; font-family: sans-serif;">
                <h2>"OpenAlgo Charts Leptos Demo"</h2>
            </header>
            <div style="flex: 1; min-height: 0;">
                <ChartContainer
                    data=bars
                    series_kind=SeriesKind::Candlestick
                    on_crosshair=on_crosshair
                />
            </div>
        </main>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
