//! Headless / Framework-agnostic example using ChartController
//!
//! Demonstrates mounting, updating, and cleanup on a raw HTML element (as used in Dioxus, Yew, or Vanilla JS).

use openalgo_charts::prelude::*;

fn main() {
    let mut controller = ChartController::new();

    let bars = vec![
        Bar::new(1700000000, 100.0, 105.0, 99.5, 104.0, 1500.0),
        Bar::new(1700086400, 104.0, 108.0, 103.0, 107.5, 2300.0),
    ];

    // Mount by element id (e.g., inside Dioxus use_effect or Yew rendered())
    if let Ok(handle) = controller.mount_by_id(
        "chart-container",
        None,
        SeriesKind::Candlestick,
        None,
        &bars,
    ) {
        handle.fit_content();
    }

    // Streaming updates
    let live_bar = Bar::new(1700172800, 107.5, 112.0, 106.0, 110.0, 3100.0);
    let _ = controller.update_last(&live_bar);

    // Guaranteed leak-free cleanup on drop
    drop(controller);
}
